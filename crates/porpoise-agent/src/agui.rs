//! Conversions from Porpoise agent output into AG-UI translator inputs.
//!
//! [`porpoise_agui::AgUiInput`] is the boundary type: this module maps the
//! agent crate's raw output ([`AgentOutput`]) and structured parse results
//! ([`ParsedEvent`]) onto it, so the protocol crate stays a leaf
//! dependency with no knowledge of agent internals.

use porpoise_agui::AgUiInput;

use crate::{
    adapters::traits::ParsedEvent,
    types::{AgentOutput, OutputType},
};

impl From<&AgentOutput> for AgUiInput {
    fn from(output: &AgentOutput) -> Self {
        match output.output_type {
            OutputType::Stdout => AgUiInput::Text(output.text.clone()),
            OutputType::Stderr => AgUiInput::Error(output.text.clone()),
            OutputType::Status => AgUiInput::Status(output.text.clone()),
            OutputType::Thinking => AgUiInput::Thinking(output.text.clone()),
            OutputType::Exit(code) => AgUiInput::Done { exit_code: Some(code) },
        }
    }
}

impl From<&ParsedEvent> for AgUiInput {
    fn from(event: &ParsedEvent) -> Self {
        match event {
            ParsedEvent::Text(text) | ParsedEvent::Raw(text) => AgUiInput::Text(text.clone()),
            ParsedEvent::Thinking(text) => AgUiInput::Thinking(text.clone()),
            ParsedEvent::ToolCall(call) => AgUiInput::ToolCall {
                name: call.tool_name.clone(),
                arguments: serde_json::json!(call.arguments),
                id: call.call_id.clone(),
            },
            ParsedEvent::Diff(diff) => AgUiInput::Diff {
                file_path: diff.file_path.clone(),
                old_content: diff.old_content.clone(),
                new_content: diff.new_content.clone(),
                language: diff.language.clone(),
            },
            ParsedEvent::Error(message) => AgUiInput::Error(message.clone()),
            ParsedEvent::Done { exit_code } => AgUiInput::Done { exit_code: *exit_code },
            ParsedEvent::StatusChange(status) => AgUiInput::Status(status.clone()),
            ParsedEvent::WaitingForInput => AgUiInput::WaitingForInput {
                id: None,
                reason: "waiting_for_input".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::adapters::traits::ToolCall;

    #[test]
    fn agent_output_maps_to_translator_inputs() {
        let cases = [
            (
                AgentOutput {
                    text: "hello".into(),
                    output_type: OutputType::Stdout,
                },
                AgUiInput::Text("hello".into()),
            ),
            (
                AgentOutput {
                    text: "boom".into(),
                    output_type: OutputType::Stderr,
                },
                AgUiInput::Error("boom".into()),
            ),
            (
                AgentOutput {
                    text: "editing".into(),
                    output_type: OutputType::Status,
                },
                AgUiInput::Status("editing".into()),
            ),
            (
                AgentOutput {
                    text: "hmm".into(),
                    output_type: OutputType::Thinking,
                },
                AgUiInput::Thinking("hmm".into()),
            ),
            (
                AgentOutput {
                    text: String::new(),
                    output_type: OutputType::Exit(0),
                },
                AgUiInput::Done { exit_code: Some(0) },
            ),
        ];
        for (output, expected) in cases {
            assert_eq!(AgUiInput::from(&output), expected);
        }
    }

    #[test]
    fn parsed_events_map_to_translator_inputs() {
        let mut arguments = HashMap::new();
        arguments.insert("cmd".to_string(), serde_json::json!("ls"));

        let cases = [
            (ParsedEvent::Text("hi".into()), AgUiInput::Text("hi".into())),
            (ParsedEvent::Thinking("deep".into()), AgUiInput::Thinking("deep".into())),
            (
                ParsedEvent::ToolCall(ToolCall {
                    tool_name: "shell".into(),
                    arguments,
                    call_id: Some("call_9".into()),
                }),
                AgUiInput::ToolCall {
                    name: "shell".into(),
                    arguments: serde_json::json!({"cmd": "ls"}),
                    id: Some("call_9".into()),
                },
            ),
            (
                ParsedEvent::Done { exit_code: Some(2) },
                AgUiInput::Done { exit_code: Some(2) },
            ),
            (
                ParsedEvent::StatusChange("Applied edit".into()),
                AgUiInput::Status("Applied edit".into()),
            ),
            (
                ParsedEvent::WaitingForInput,
                AgUiInput::WaitingForInput {
                    id: None,
                    reason: "waiting_for_input".into(),
                },
            ),
            (ParsedEvent::Error("warn".into()), AgUiInput::Error("warn".into())),
            (ParsedEvent::Raw("as-is".into()), AgUiInput::Text("as-is".into())),
        ];
        for (event, expected) in cases {
            assert_eq!(AgUiInput::from(&event), expected);
        }
    }

    /// The end-to-end mapping a run performs: raw output → input →
    /// spec-shaped events.
    #[test]
    fn output_to_event_pipeline() {
        let mut session = porpoise_agui::AgUiSession::with_ids("t", "r");
        let mut events = session.begin();
        for output in [
            AgentOutput {
                text: "Hello".into(),
                output_type: OutputType::Stdout,
            },
            AgentOutput {
                text: String::new(),
                output_type: OutputType::Exit(0),
            },
        ] {
            events.extend(session.feed(AgUiInput::from(&output)));
        }
        assert_eq!(events.len(), 5, "run start, msg start, content, msg end, finish");
        assert!(events.last().unwrap().is_terminal());
        assert!(session.is_finished());
        // The exit closes the open message before the terminal event.
        assert_eq!(events[3].event_type(), porpoise_agui::EventType::TextMessageEnd);
    }
}
