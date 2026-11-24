//! Translation of a Porpoise agent run into an AG-UI event stream.
//!
//! [`AgUiSession`] is the state machine that turns the structured output a
//! Porpoise agent produces ([`AgUiInput`]) into spec-shaped AG-UI events:
//!
//! ```text
//! stdout text   -> TEXT_MESSAGE_START / CONTENT / END
//! thinking      -> REASONING_MESSAGE_START / CONTENT / END
//! tool call     -> TOOL_CALL_START / ARGS / END
//! status        -> STEP_STARTED / STEP_FINISHED
//! stderr/error  -> CUSTOM porpoise.* (the run continues)
//! waiting       -> RUN_FINISHED outcome: interrupt  (run paused)
//! exit 0        -> RUN_FINISHED                      (run succeeded)
//! exit != 0     -> RUN_ERROR                         (run failed)
//! stop/cancel   -> RUN_FINISHED outcome: cancelled
//! ```
//!
//! Protocol rules the session enforces: exactly one `RUN_STARTED` at the
//! beginning, exactly one terminal event (`RUN_FINISHED` or `RUN_ERROR`)
//! at the end, open message streams closed before any terminal event, and
//! no events at all after the run is terminal.

use uuid::Uuid;

use crate::events::{
    BaseEvent, CustomEvent, Event, Interrupt, PROTOCOL_VERSION, ReasoningMessageContentEvent, ReasoningMessageEndEvent,
    ReasoningMessageStartEvent, RunErrorEvent, RunFinishedEvent, RunFinishedOutcome, RunStartedEvent,
    StepFinishedEvent, StepStartedEvent, TextMessageContentEvent, TextMessageEndEvent, TextMessageStartEvent,
    ToolCallArgsEvent, ToolCallEndEvent, ToolCallStartEvent,
};

/// What one chunk of Porpoise agent output means to the AG-UI translator.
///
/// `porpoise-agent` converts its `AgentOutput` / `ParsedEvent` values into
/// this enum; keeping the protocol crate free of agent-crate types leaves
/// it a leaf dependency.
#[derive(Debug, Clone, PartialEq)]
pub enum AgUiInput {
    /// A fragment of the assistant's reply.
    Text(String),
    /// A fragment of the agent's private reasoning.
    Thinking(String),
    /// The agent requested a tool call; `arguments` arrive whole.
    ToolCall {
        name: String,
        arguments: serde_json::Value,
        id: Option<String>,
    },
    /// A tool returned its result.
    ToolResult { id: String, content: String },
    /// A short status line ("Applied edit to src/main.rs").
    Status(String),
    /// A non-fatal error line (typically stderr); the run continues.
    Error(String),
    /// The agent produced a file diff.
    Diff {
        file_path: String,
        old_content: Option<String>,
        new_content: String,
        language: Option<String>,
    },
    /// The agent paused and needs the user before it can continue.
    WaitingForInput { id: Option<String>, reason: String },
    /// Unrecognised output, passed through as text.
    Raw(String),
    /// The agent process exited.
    Done { exit_code: Option<i32> },
}

/// Translates one agent run into AG-UI events, tracking which message
/// streams are open and whether the run has ended.
#[derive(Debug, Clone)]
pub struct AgUiSession {
    thread_id: String,
    run_id: String,
    begun: bool,
    finished: bool,
    open_text: Option<String>,
    open_reasoning: Option<String>,
    tool_counter: u64,
}

impl AgUiSession {
    /// A session with fresh thread and run ids.
    pub fn new() -> Self {
        Self::with_ids(Uuid::now_v7().to_string(), Uuid::now_v7().to_string())
    }

    /// A session for a caller-chosen thread (and optional run) id, so a
    /// client can continue an existing conversation.
    pub fn with_ids(thread_id: impl Into<String>, run_id: impl Into<String>) -> Self {
        Self {
            thread_id: thread_id.into(),
            run_id: run_id.into(),
            begun: false,
            finished: false,
            open_text: None,
            open_reasoning: None,
            tool_counter: 0,
        }
    }

    /// The conversation this run belongs to.
    pub fn thread_id(&self) -> &str {
        &self.thread_id
    }

    /// This run's id.
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// Whether a terminal event (`RUN_FINISHED` / `RUN_ERROR`) has been
    /// emitted; nothing may follow it.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Emit `RUN_STARTED`. Idempotent: a second call emits nothing.
    pub fn begin(&mut self) -> Vec<Event> {
        if self.begun || self.finished {
            return Vec::new();
        }
        self.begun = true;
        vec![Event::RunStarted(RunStartedEvent {
            base: BaseEvent::now(),
            thread_id: self.thread_id.clone(),
            run_id: self.run_id.clone(),
            protocol_version: Some(PROTOCOL_VERSION.into()),
            parent_run_id: None,
        })]
    }

    /// Translate one chunk of agent output. Returns an empty vector once
    /// the run is terminal, so a late chunk can never violate the protocol.
    pub fn feed(&mut self, input: AgUiInput) -> Vec<Event> {
        if self.finished {
            return Vec::new();
        }
        match input {
            AgUiInput::Text(text) | AgUiInput::Raw(text) => {
                if text.is_empty() && self.open_text.is_none() {
                    return Vec::new();
                }
                self.text_delta(text)
            }
            AgUiInput::Thinking(text) => {
                if text.is_empty() && self.open_reasoning.is_none() {
                    return Vec::new();
                }
                let mut events = self.close_text();
                if self.open_reasoning.is_none() {
                    let message_id = Uuid::now_v7().to_string();
                    self.open_reasoning = Some(message_id.clone());
                    events.push(Event::ReasoningMessageStart(ReasoningMessageStartEvent {
                        base: BaseEvent::now(),
                        message_id,
                        role: crate::events::Role::Reasoning,
                    }));
                }
                events.push(Event::ReasoningMessageContent(ReasoningMessageContentEvent {
                    base: BaseEvent::now(),
                    message_id: self.open_reasoning.clone().unwrap_or_default(),
                    delta: text,
                }));
                events
            }
            AgUiInput::ToolCall { name, arguments, id } => {
                // The call hangs off the message that was open when it was
                // requested; that message closes before the call starts.
                let parent_message_id = self.open_text.clone();
                let mut events = self.close_open_streams();
                self.tool_counter += 1;
                let tool_call_id = id.unwrap_or_else(|| format!("call_{}", self.tool_counter));
                events.push(Event::ToolCallStart(ToolCallStartEvent {
                    base: BaseEvent::now(),
                    tool_call_id: tool_call_id.clone(),
                    tool_call_name: name,
                    parent_message_id,
                }));
                let delta = serde_json::to_string(&arguments).unwrap_or_else(|_| "{}".into());
                events.push(Event::ToolCallArgs(ToolCallArgsEvent {
                    base: BaseEvent::now(),
                    tool_call_id: tool_call_id.clone(),
                    delta,
                }));
                events.push(Event::ToolCallEnd(ToolCallEndEvent {
                    base: BaseEvent::now(),
                    tool_call_id,
                }));
                events
            }
            AgUiInput::ToolResult { id, content } => {
                let mut events = self.close_open_streams();
                events.push(Event::ToolCallResult(crate::events::ToolCallResultEvent {
                    base: BaseEvent::now(),
                    message_id: Uuid::now_v7().to_string(),
                    tool_call_id: id,
                    content,
                    role: crate::events::Role::Tool,
                }));
                events
            }
            AgUiInput::Status(status) => vec![
                Event::StepStarted(StepStartedEvent {
                    base: BaseEvent::now(),
                    step_name: status.clone(),
                }),
                Event::StepFinished(StepFinishedEvent {
                    base: BaseEvent::now(),
                    step_name: status,
                }),
            ],
            AgUiInput::Error(message) => vec![Event::Custom(CustomEvent {
                base: BaseEvent::now(),
                name: "porpoise.error".into(),
                value: serde_json::json!({ "message": message }),
            })],
            AgUiInput::Diff {
                file_path,
                old_content,
                new_content,
                language,
            } => vec![Event::Custom(CustomEvent {
                base: BaseEvent::now(),
                name: "porpoise.diff".into(),
                value: serde_json::json!({
                    "file_path": file_path,
                    "old_content": old_content,
                    "new_content": new_content,
                    "language": language,
                }),
            })],
            AgUiInput::WaitingForInput { id, reason } => {
                let mut events = self.close_open_streams();
                let interrupt_id = id.unwrap_or_else(|| format!("int_{}", Uuid::now_v7()));
                events.push(Event::RunFinished(RunFinishedEvent {
                    base: BaseEvent::now(),
                    thread_id: self.thread_id.clone(),
                    run_id: self.run_id.clone(),
                    result: None,
                    outcome: Some(RunFinishedOutcome::interrupt(vec![Interrupt {
                        id: interrupt_id,
                        reason,
                        message: None,
                    }])),
                }));
                self.finished = true;
                events
            }
            AgUiInput::Done { exit_code } => {
                let mut events = self.close_open_streams();
                match exit_code {
                    None | Some(0) => {
                        events.push(Event::RunFinished(RunFinishedEvent {
                            base: BaseEvent::now(),
                            thread_id: self.thread_id.clone(),
                            run_id: self.run_id.clone(),
                            result: None,
                            outcome: Some(RunFinishedOutcome::success()),
                        }));
                    }
                    Some(code) => {
                        events.push(Event::RunError(RunErrorEvent {
                            base: BaseEvent::now(),
                            message: format!("agent exited with code {code}"),
                            code: Some("exit_code".into()),
                        }));
                    }
                }
                self.finished = true;
                events
            }
        }
    }

    /// Close the run successfully, if it has not ended yet. Used when the
    /// output stream ends without an explicit exit event.
    pub fn finish_ok(&mut self) -> Vec<Event> {
        if self.finished {
            return Vec::new();
        }
        let mut events = self.close_open_streams();
        events.push(Event::RunFinished(RunFinishedEvent {
            base: BaseEvent::now(),
            thread_id: self.thread_id.clone(),
            run_id: self.run_id.clone(),
            result: None,
            outcome: Some(RunFinishedOutcome::success()),
        }));
        self.finished = true;
        events
    }

    /// Fail the run with `message`, if it has not ended yet.
    pub fn finish_error(&mut self, message: impl Into<String>) -> Vec<Event> {
        if self.finished {
            return Vec::new();
        }
        let mut events = self.close_open_streams();
        events.push(Event::RunError(RunErrorEvent {
            base: BaseEvent::now(),
            message: message.into(),
            code: Some("agent_error".into()),
        }));
        self.finished = true;
        events
    }

    /// End the run because the user stopped it: `RUN_FINISHED` with the
    /// `cancelled` outcome, if it has not ended yet.
    pub fn cancel(&mut self) -> Vec<Event> {
        if self.finished {
            return Vec::new();
        }
        let mut events = self.close_open_streams();
        events.push(Event::RunFinished(RunFinishedEvent {
            base: BaseEvent::now(),
            thread_id: self.thread_id.clone(),
            run_id: self.run_id.clone(),
            result: None,
            outcome: Some(RunFinishedOutcome::Cancelled),
        }));
        self.finished = true;
        events
    }

    /// Append `text` to the open text message, opening it first if needed.
    fn text_delta(&mut self, text: String) -> Vec<Event> {
        let mut events = Vec::new();
        if self.open_text.is_none() {
            let message_id = Uuid::now_v7().to_string();
            self.open_text = Some(message_id.clone());
            events.push(Event::TextMessageStart(TextMessageStartEvent {
                base: BaseEvent::now(),
                message_id,
                role: Some(crate::events::Role::Assistant),
                name: None,
            }));
        }
        events.push(Event::TextMessageContent(TextMessageContentEvent {
            base: BaseEvent::now(),
            message_id: self.open_text.clone().unwrap_or_default(),
            delta: text,
        }));
        events
    }

    /// Close the open text message, if any.
    fn close_text(&mut self) -> Vec<Event> {
        match self.open_text.take() {
            Some(message_id) => vec![Event::TextMessageEnd(TextMessageEndEvent {
                base: BaseEvent::now(),
                message_id,
            })],
            None => Vec::new(),
        }
    }

    /// Close every open message stream (text and reasoning), so a tool
    /// call or terminal event never lands inside an open message.
    fn close_open_streams(&mut self) -> Vec<Event> {
        let mut events = self.close_text();
        if let Some(message_id) = self.open_reasoning.take() {
            events.push(Event::ReasoningMessageEnd(ReasoningMessageEndEvent {
                base: BaseEvent::now(),
                message_id,
            }));
        }
        events
    }
}

impl Default for AgUiSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventType;

    fn types(events: &[Event]) -> Vec<EventType> {
        events.iter().map(|e| e.event_type()).collect()
    }

    #[test]
    fn full_run_produces_spec_ordered_stream() {
        let mut session = AgUiSession::with_ids("thread-1", "run-1");
        let mut all = session.begin();
        all.extend(session.feed(AgUiInput::Text("Hello ".into())));
        all.extend(session.feed(AgUiInput::Text("world".into())));
        all.extend(session.feed(AgUiInput::Thinking("should I edit?".into())));
        all.extend(session.feed(AgUiInput::ToolCall {
            name: "shell".into(),
            arguments: serde_json::json!({"cmd": "ls"}),
            id: Some("call_1".into()),
        }));
        all.extend(session.feed(AgUiInput::Status("Applied edit to main.rs".into())));
        all.extend(session.feed(AgUiInput::Text("Done!".into())));
        all.extend(session.feed(AgUiInput::Done { exit_code: Some(0) }));

        assert_eq!(
            types(&all),
            vec![
                EventType::RunStarted,
                EventType::TextMessageStart,
                EventType::TextMessageContent,
                EventType::TextMessageContent,
                // thinking closes the text message first
                EventType::TextMessageEnd,
                EventType::ReasoningMessageStart,
                EventType::ReasoningMessageContent,
                // a tool call closes the reasoning stream
                EventType::ReasoningMessageEnd,
                EventType::ToolCallStart,
                EventType::ToolCallArgs,
                EventType::ToolCallEnd,
                EventType::StepStarted,
                EventType::StepFinished,
                // text resumes after the status step
                EventType::TextMessageStart,
                EventType::TextMessageContent,
                // the exit closes that message before the terminal event
                EventType::TextMessageEnd,
                EventType::RunFinished,
            ]
        );

        // RUN_STARTED carries the ids and protocol version.
        match &all[0] {
            Event::RunStarted(e) => {
                assert_eq!(e.thread_id, "thread-1");
                assert_eq!(e.run_id, "run-1");
                assert_eq!(e.protocol_version.as_deref(), Some("1.0"));
            }
            other => panic!("expected RUN_STARTED, got {other:?}"),
        }

        // Exactly one terminal event, last; nothing after it.
        assert!(all.last().unwrap().is_terminal());
        assert_eq!(
            all.iter().filter(|e| e.is_terminal()).count(),
            1,
            "run must end with exactly one terminal event"
        );
    }

    #[test]
    fn non_zero_exit_becomes_run_error() {
        let mut session = AgUiSession::new();
        let mut all = session.begin();
        all.extend(session.feed(AgUiInput::Text("hi".into())));
        all.extend(session.feed(AgUiInput::Done { exit_code: Some(3) }));
        assert!(session.is_finished());
        assert_eq!(
            types(&all),
            vec![
                EventType::RunStarted,
                EventType::TextMessageStart,
                EventType::TextMessageContent,
                EventType::TextMessageEnd,
                EventType::RunError,
            ]
        );
        match all.last().unwrap() {
            Event::RunError(e) => {
                assert!(e.message.contains('3'));
                assert_eq!(e.code.as_deref(), Some("exit_code"));
            }
            other => panic!("expected RUN_ERROR, got {other:?}"),
        }
    }

    #[test]
    fn waiting_for_input_pauses_with_interrupt_outcome() {
        let mut session = AgUiSession::new();
        let mut all = session.begin();
        all.extend(session.feed(AgUiInput::Text("approve?".into())));
        all.extend(session.feed(AgUiInput::WaitingForInput {
            id: Some("int_1".into()),
            reason: "approval".into(),
        }));
        assert!(session.is_finished());
        match all.last().unwrap() {
            Event::RunFinished(e) => match e.outcome.as_ref() {
                Some(RunFinishedOutcome::Interrupt { interrupts }) => {
                    assert_eq!(interrupts[0].id, "int_1");
                    assert_eq!(interrupts[0].reason, "approval");
                }
                other => panic!("expected interrupt outcome, got {other:?}"),
            },
            other => panic!("expected RUN_FINISHED, got {other:?}"),
        }
        // A late chunk after the pause emits nothing.
        assert!(session.feed(AgUiInput::Text("late".into())).is_empty());
    }

    #[test]
    fn cancel_emits_cancelled_outcome_once() {
        let mut session = AgUiSession::new();
        let all = session.begin();
        let cancelled = session.cancel();
        assert!(session.is_finished());
        match cancelled.last().unwrap() {
            Event::RunFinished(e) => {
                assert_eq!(e.outcome, Some(RunFinishedOutcome::Cancelled));
            }
            other => panic!("expected RUN_FINISHED, got {other:?}"),
        }
        assert!(session.cancel().is_empty(), "cancel is idempotent");
        assert!(session.finish_ok().is_empty());
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn finish_ok_closes_open_messages_first() {
        let mut session = AgUiSession::new();
        let mut all = session.begin();
        all.extend(session.feed(AgUiInput::Text("unfinished thought".into())));
        all.extend(session.finish_ok());
        assert_eq!(
            types(&all),
            vec![
                EventType::RunStarted,
                EventType::TextMessageStart,
                EventType::TextMessageContent,
                EventType::TextMessageEnd,
                EventType::RunFinished,
            ]
        );
        // The terminal event closes with the success outcome.
        match all.last().unwrap() {
            Event::RunFinished(e) => {
                assert_eq!(e.outcome, Some(RunFinishedOutcome::success()));
            }
            other => panic!("expected RUN_FINISHED, got {other:?}"),
        }
    }

    #[test]
    fn stderr_and_diff_are_custom_events_and_do_not_end_the_run() {
        let mut session = AgUiSession::new();
        let mut all = session.begin();
        all.extend(session.feed(AgUiInput::Error("warning: deprecated".into())));
        all.extend(session.feed(AgUiInput::Diff {
            file_path: "src/lib.rs".into(),
            old_content: None,
            new_content: "fn main() {}".into(),
            language: Some("rust".into()),
        }));
        assert!(!session.is_finished(), "non-fatal signals keep the run open");
        assert_eq!(types(&all)[1], EventType::Custom);
        assert_eq!(types(&all)[2], EventType::Custom);
        match &all[1] {
            Event::Custom(e) => {
                assert_eq!(e.name, "porpoise.error");
                assert_eq!(e.value["message"], "warning: deprecated");
            }
            other => panic!("expected CUSTOM, got {other:?}"),
        }
        match &all[2] {
            Event::Custom(e) => {
                assert_eq!(e.name, "porpoise.diff");
                assert_eq!(e.value["file_path"], "src/lib.rs");
            }
            other => panic!("expected CUSTOM, got {other:?}"),
        }
    }

    #[test]
    fn begin_is_idempotent() {
        let mut session = AgUiSession::new();
        assert_eq!(session.begin().len(), 1);
        assert!(session.begin().is_empty());
    }
}
