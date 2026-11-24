//! AG-UI 1.0 events — the typed vocabulary a Porpoise agent run streams.
//!
//! Vendor note: these types mirror the frozen specification
//! `spec/1.0/schema.json` of [ag-ui-protocol/ag-ui](https://github.com/ag-ui-protocol/ag-ui)
//! (MIT), field for field, so anything that speaks AG-UI — the TypeScript
//! client, the Python SDK, CopilotKit — can consume a Porpoise run without
//! translation. This module carries the subset a *producer* emits; the
//! canonical source of truth for the wire format remains the spec.

use serde::{Deserialize, Serialize};

/// The protocol version this crate emits, taken from the canonical
/// schema's `$id` (`https://ag-ui.com/spec/1.0/schema.json`).
pub const PROTOCOL_VERSION: &str = "1.0";

/// The discriminator carried by every event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    TextMessageStart,
    TextMessageContent,
    TextMessageEnd,
    ToolCallStart,
    ToolCallArgs,
    ToolCallEnd,
    ToolCallResult,
    StepStarted,
    StepFinished,
    RunStarted,
    RunFinished,
    RunError,
    ReasoningMessageStart,
    ReasoningMessageContent,
    ReasoningMessageEnd,
    Custom,
}

impl EventType {
    /// How many discriminators this producer knows.
    pub const COUNT: usize = 16;
}

/// The fields every event carries, whatever its type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BaseEvent {
    /// When the event was created: milliseconds since the Unix epoch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    /// The provider-native event this one was translated from, carried
    /// verbatim for debugging.
    #[serde(rename = "rawEvent", skip_serializing_if = "Option::is_none")]
    pub raw_event: Option<serde_json::Value>,
    /// Extra information attached to this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl BaseEvent {
    /// A base event stamped with the current time in epoch milliseconds.
    pub fn now() -> Self {
        let millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        Self {
            timestamp: Some(millis),
            raw_event: None,
            metadata: None,
        }
    }
}

/// Every role a materialised message may have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Developer,
    System,
    Assistant,
    User,
    Tool,
    Activity,
    Reasoning,
}

/// Opens a streamed text message. The content arrives as
/// [`TextMessageContentEvent`]s and the message closes with
/// [`TextMessageEndEvent`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextMessageStartEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    /// Identifies the message this stream builds.
    #[serde(rename = "messageId")]
    pub message_id: String,
    /// Who the message is from. An absent role means assistant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    /// An optional display name for the author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Appends a fragment to a streamed text message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextMessageContentEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    /// The message this fragment belongs to.
    #[serde(rename = "messageId")]
    pub message_id: String,
    /// The fragment to append. May be the empty string: providers emit
    /// empty deltas as keep-alives.
    pub delta: String,
}

/// Closes a streamed text message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextMessageEndEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    /// The message this stream closes.
    #[serde(rename = "messageId")]
    pub message_id: String,
}

/// Opens a tool call: which tool, and inside which assistant message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallStartEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    /// Identifies this call; the answering tool message carries it as
    /// `toolCallId`.
    #[serde(rename = "toolCallId")]
    pub tool_call_id: String,
    /// Which tool is being called.
    #[serde(rename = "toolCallName")]
    pub tool_call_name: String,
    /// The assistant message that holds this call.
    #[serde(rename = "parentMessageId", skip_serializing_if = "Option::is_none")]
    pub parent_message_id: Option<String>,
}

/// Appends a fragment of tool-call arguments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallArgsEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    /// The call these arguments belong to.
    #[serde(rename = "toolCallId")]
    pub tool_call_id: String,
    /// The fragment to append. May be the empty string.
    pub delta: String,
}

/// Closes the argument stream of a tool call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallEndEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    /// The call whose arguments are complete.
    #[serde(rename = "toolCallId")]
    pub tool_call_id: String,
}

/// Delivers the result of a completed tool call as a tool message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallResultEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    /// Identifies the tool message this result materialises as.
    #[serde(rename = "messageId")]
    pub message_id: String,
    /// The call being answered.
    #[serde(rename = "toolCallId")]
    pub tool_call_id: String,
    /// What the tool returned, as text.
    pub content: String,
    /// Always `tool`.
    #[serde(default = "default_tool_role")]
    pub role: Role,
}

fn default_tool_role() -> Role {
    Role::Tool
}

/// Signals the start of a named step within a run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepStartedEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    /// The step's name.
    #[serde(rename = "stepName")]
    pub step_name: String,
}

/// Signals the completion of a named step within a run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepFinishedEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    /// The step's name.
    #[serde(rename = "stepName")]
    pub step_name: String,
}

/// Opens a streamed reasoning message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningMessageStartEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    /// Identifies the reasoning message this stream builds.
    #[serde(rename = "messageId")]
    pub message_id: String,
    /// Always `reasoning`.
    #[serde(default = "default_reasoning_role")]
    pub role: Role,
}

fn default_reasoning_role() -> Role {
    Role::Reasoning
}

/// Appends a fragment to a streamed reasoning message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningMessageContentEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    /// The reasoning message this fragment belongs to.
    #[serde(rename = "messageId")]
    pub message_id: String,
    /// The fragment to append. May be the empty string.
    pub delta: String,
}

/// Closes a streamed reasoning message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningMessageEndEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    /// The reasoning message this stream closes.
    #[serde(rename = "messageId")]
    pub message_id: String,
}

/// Signals that a run has started within a specific thread.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunStartedEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    /// The conversation this run belongs to.
    #[serde(rename = "threadId")]
    pub thread_id: String,
    /// The run being started.
    #[serde(rename = "runId")]
    pub run_id: String,
    /// The protocol version this producer speaks, such as `"1.0"`.
    #[serde(rename = "protocolVersion", skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
    /// The run that spawned this one, when an agent invokes another agent
    /// as a separate run.
    #[serde(rename = "parentRunId", skip_serializing_if = "Option::is_none")]
    pub parent_run_id: Option<String>,
}

/// Signals that a run completed, paused, or was cancelled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunFinishedEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    /// The conversation this run belongs to.
    #[serde(rename = "threadId")]
    pub thread_id: String,
    /// The run being closed.
    #[serde(rename = "runId")]
    pub run_id: String,
    /// The run's return value, if it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Why the run ended. Absent means success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<RunFinishedOutcome>,
}

/// Signals that a run failed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunErrorEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    /// What went wrong, for a person to read.
    pub message: String,
    /// A machine-readable error code. An open string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// An application-specific event with arbitrary data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    /// Identifies the custom event. Porpoise reserves the `porpoise.`
    /// prefix for its own signals (diffs, stderr, status).
    pub name: String,
    /// The payload. Any JSON value, including null.
    pub value: serde_json::Value,
}

/// Something a run needs from outside before it can continue, such as an
/// approval or a missing value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interrupt {
    /// Identifies the interrupt. A resume entry answers it by this id.
    pub id: String,
    /// Why the run stopped. An open string.
    pub reason: String,
    /// A human-readable prompt for whoever answers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Why a run ended.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RunFinishedOutcome {
    /// The run completed. Equivalent to an absent outcome.
    Success {
        /// Tool calls the run started and left unanswered, in the order
        /// they were made.
        #[serde(rename = "pendingToolCallIds", skip_serializing_if = "Option::is_none")]
        pending_tool_call_ids: Option<Vec<String>>,
    },
    /// The run is paused, waiting for something outside it. Resuming means
    /// starting a new run whose resume entries answer these interrupts.
    Interrupt {
        /// What the run is waiting for. At least one.
        interrupts: Vec<Interrupt>,
    },
    /// The run was stopped before it completed, by whoever was running it,
    /// and did not fail.
    Cancelled,
}

impl RunFinishedOutcome {
    /// A successful completion, optionally naming unanswered tool calls.
    pub fn success() -> Self {
        Self::Success {
            pending_tool_call_ids: None,
        }
    }

    /// A paused run waiting on `interrupts`.
    pub fn interrupt(interrupts: Vec<Interrupt>) -> Self {
        Self::Interrupt { interrupts }
    }
}

/// Union of the AG-UI events Porpoise emits, discriminated on the wire by
/// `type`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Event {
    TextMessageStart(TextMessageStartEvent),
    TextMessageContent(TextMessageContentEvent),
    TextMessageEnd(TextMessageEndEvent),
    ToolCallStart(ToolCallStartEvent),
    ToolCallArgs(ToolCallArgsEvent),
    ToolCallEnd(ToolCallEndEvent),
    ToolCallResult(ToolCallResultEvent),
    StepStarted(StepStartedEvent),
    StepFinished(StepFinishedEvent),
    RunStarted(RunStartedEvent),
    RunFinished(RunFinishedEvent),
    RunError(RunErrorEvent),
    ReasoningMessageStart(ReasoningMessageStartEvent),
    ReasoningMessageContent(ReasoningMessageContentEvent),
    ReasoningMessageEnd(ReasoningMessageEndEvent),
    Custom(CustomEvent),
}

impl Event {
    /// The event's discriminator.
    pub fn event_type(&self) -> EventType {
        match self {
            Event::TextMessageStart(_) => EventType::TextMessageStart,
            Event::TextMessageContent(_) => EventType::TextMessageContent,
            Event::TextMessageEnd(_) => EventType::TextMessageEnd,
            Event::ToolCallStart(_) => EventType::ToolCallStart,
            Event::ToolCallArgs(_) => EventType::ToolCallArgs,
            Event::ToolCallEnd(_) => EventType::ToolCallEnd,
            Event::ToolCallResult(_) => EventType::ToolCallResult,
            Event::StepStarted(_) => EventType::StepStarted,
            Event::StepFinished(_) => EventType::StepFinished,
            Event::RunStarted(_) => EventType::RunStarted,
            Event::RunFinished(_) => EventType::RunFinished,
            Event::RunError(_) => EventType::RunError,
            Event::ReasoningMessageStart(_) => EventType::ReasoningMessageStart,
            Event::ReasoningMessageContent(_) => EventType::ReasoningMessageContent,
            Event::ReasoningMessageEnd(_) => EventType::ReasoningMessageEnd,
            Event::Custom(_) => EventType::Custom,
        }
    }

    /// The event's creation timestamp, if set.
    pub fn timestamp(&self) -> Option<i64> {
        match self {
            Event::TextMessageStart(e) => e.base.timestamp,
            Event::TextMessageContent(e) => e.base.timestamp,
            Event::TextMessageEnd(e) => e.base.timestamp,
            Event::ToolCallStart(e) => e.base.timestamp,
            Event::ToolCallArgs(e) => e.base.timestamp,
            Event::ToolCallEnd(e) => e.base.timestamp,
            Event::ToolCallResult(e) => e.base.timestamp,
            Event::StepStarted(e) => e.base.timestamp,
            Event::StepFinished(e) => e.base.timestamp,
            Event::RunStarted(e) => e.base.timestamp,
            Event::RunFinished(e) => e.base.timestamp,
            Event::RunError(e) => e.base.timestamp,
            Event::ReasoningMessageStart(e) => e.base.timestamp,
            Event::ReasoningMessageContent(e) => e.base.timestamp,
            Event::ReasoningMessageEnd(e) => e.base.timestamp,
            Event::Custom(e) => e.base.timestamp,
        }
    }

    /// Whether this event closes the run: a run ends with exactly one
    /// `RUN_FINISHED` or `RUN_ERROR`, and nothing may follow it.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Event::RunFinished(_) | Event::RunError(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every emitted event must validate against `spec/1.0/schema.json`:
    /// correct discriminator tag, camelCase fields, optional fields omitted
    /// rather than emitted as null.
    #[test]
    fn events_serialize_with_spec_wire_format() {
        let cases: Vec<(Event, &str)> = vec![
            (
                Event::TextMessageStart(TextMessageStartEvent {
                    base: BaseEvent::default(),
                    message_id: "m1".into(),
                    role: Some(Role::Assistant),
                    name: None,
                }),
                "TEXT_MESSAGE_START",
            ),
            (
                Event::TextMessageContent(TextMessageContentEvent {
                    base: BaseEvent::default(),
                    message_id: "m1".into(),
                    delta: "hi".into(),
                }),
                "TEXT_MESSAGE_CONTENT",
            ),
            (
                Event::TextMessageEnd(TextMessageEndEvent {
                    base: BaseEvent::default(),
                    message_id: "m1".into(),
                }),
                "TEXT_MESSAGE_END",
            ),
            (
                Event::ToolCallStart(ToolCallStartEvent {
                    base: BaseEvent::default(),
                    tool_call_id: "call_1".into(),
                    tool_call_name: "shell".into(),
                    parent_message_id: None,
                }),
                "TOOL_CALL_START",
            ),
            (
                Event::ToolCallArgs(ToolCallArgsEvent {
                    base: BaseEvent::default(),
                    tool_call_id: "call_1".into(),
                    delta: "{}".into(),
                }),
                "TOOL_CALL_ARGS",
            ),
            (
                Event::ToolCallEnd(ToolCallEndEvent {
                    base: BaseEvent::default(),
                    tool_call_id: "call_1".into(),
                }),
                "TOOL_CALL_END",
            ),
            (
                Event::ToolCallResult(ToolCallResultEvent {
                    base: BaseEvent::default(),
                    message_id: "m2".into(),
                    tool_call_id: "call_1".into(),
                    content: "ok".into(),
                    role: Role::Tool,
                }),
                "TOOL_CALL_RESULT",
            ),
            (
                Event::StepStarted(StepStartedEvent {
                    base: BaseEvent::default(),
                    step_name: "edit".into(),
                }),
                "STEP_STARTED",
            ),
            (
                Event::StepFinished(StepFinishedEvent {
                    base: BaseEvent::default(),
                    step_name: "edit".into(),
                }),
                "STEP_FINISHED",
            ),
            (
                Event::RunStarted(RunStartedEvent {
                    base: BaseEvent::default(),
                    thread_id: "t1".into(),
                    run_id: "r1".into(),
                    protocol_version: Some(PROTOCOL_VERSION.into()),
                    parent_run_id: None,
                }),
                "RUN_STARTED",
            ),
            (
                Event::RunFinished(RunFinishedEvent {
                    base: BaseEvent::default(),
                    thread_id: "t1".into(),
                    run_id: "r1".into(),
                    result: None,
                    outcome: None,
                }),
                "RUN_FINISHED",
            ),
            (
                Event::RunError(RunErrorEvent {
                    base: BaseEvent::default(),
                    message: "boom".into(),
                    code: Some("exit_code".into()),
                }),
                "RUN_ERROR",
            ),
            (
                Event::ReasoningMessageStart(ReasoningMessageStartEvent {
                    base: BaseEvent::default(),
                    message_id: "m3".into(),
                    role: Role::Reasoning,
                }),
                "REASONING_MESSAGE_START",
            ),
            (
                Event::ReasoningMessageContent(ReasoningMessageContentEvent {
                    base: BaseEvent::default(),
                    message_id: "m3".into(),
                    delta: "hmm".into(),
                }),
                "REASONING_MESSAGE_CONTENT",
            ),
            (
                Event::ReasoningMessageEnd(ReasoningMessageEndEvent {
                    base: BaseEvent::default(),
                    message_id: "m3".into(),
                }),
                "REASONING_MESSAGE_END",
            ),
            (
                Event::Custom(CustomEvent {
                    base: BaseEvent::default(),
                    name: "porpoise.diff".into(),
                    value: serde_json::json!({"file_path": "src/main.rs"}),
                }),
                "CUSTOM",
            ),
        ];

        assert_eq!(cases.len(), EventType::COUNT);
        for (event, expected_tag) in cases {
            let wire = serde_json::to_value(&event).unwrap();
            assert_eq!(wire["type"], expected_tag, "tag for {expected_tag}");
            // Optional fields are omitted, never emitted as null.
            for key in ["timestamp", "rawEvent", "metadata"] {
                assert!(
                    wire.get(key).is_none(),
                    "{expected_tag} must omit absent {key}, got {wire}"
                );
            }
            // Round-trip: the wire form parses back to an equal event.
            let back: Event = serde_json::from_value(wire).unwrap();
            assert_eq!(back, event, "round-trip for {expected_tag}");
        }

        // When set, the timestamp is an integer (spec: bounded JSON
        // integer, deliberately not a float).
        let stamped = serde_json::to_value(Event::TextMessageStart(TextMessageStartEvent {
            base: BaseEvent::now(),
            message_id: "m1".into(),
            role: None,
            name: None,
        }))
        .unwrap();
        assert!(stamped["timestamp"].is_i64(), "got {stamped}");
    }

    #[test]
    fn run_finished_outcomes_use_spec_discriminators() {
        let cases = [
            (RunFinishedOutcome::success(), "success"),
            (
                RunFinishedOutcome::interrupt(vec![Interrupt {
                    id: "i1".into(),
                    reason: "waiting_for_input".into(),
                    message: None,
                }]),
                "interrupt",
            ),
            (RunFinishedOutcome::Cancelled, "cancelled"),
        ];
        for (outcome, tag) in cases {
            let wire = serde_json::to_value(&outcome).unwrap();
            assert_eq!(wire["type"], tag);
            let back: RunFinishedOutcome = serde_json::from_value(wire).unwrap();
            assert_eq!(back, outcome);
        }
    }

    #[test]
    fn terminal_detection() {
        let terminal = Event::RunFinished(RunFinishedEvent {
            base: BaseEvent::default(),
            thread_id: "t".into(),
            run_id: "r".into(),
            result: None,
            outcome: None,
        });
        let streaming = Event::TextMessageContent(TextMessageContentEvent {
            base: BaseEvent::default(),
            message_id: "m".into(),
            delta: "x".into(),
        });
        assert!(terminal.is_terminal());
        assert!(!streaming.is_terminal());
    }
}
