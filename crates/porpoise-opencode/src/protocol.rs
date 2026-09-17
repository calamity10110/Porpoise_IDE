use serde::Deserialize;

/// A single JSON event emitted by `opencode run --format json`.
///
/// OpenCode streams newline-delimited JSON events. Each line is one of:
/// - `{"type":"message", ...}` — assistant/user/tool messages
/// - `{"type":"tool_use", ...}` — tool invocation
/// - `{"type":"tool_result", ...}` — tool result
/// - `{"type":"error", ...}` — error event
/// - `{"type":"session", ...}` — session metadata
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenCodeEvent {
    Message(MessageEvent),
    ToolUse(ToolUseEvent),
    ToolResult(ToolResultEvent),
    Error(ErrorEvent),
    Session(SessionEvent),
    /// Catch-all for unknown event types.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageEvent {
    /// Role: "assistant", "user", "system".
    pub role: String,
    /// Message content (may be text or structured).
    pub content: serde_json::Value,
    /// Model used for this message.
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolUseEvent {
    /// Tool name.
    pub name: String,
    /// Tool input as JSON value.
    pub input: serde_json::Value,
    /// Unique tool invocation ID.
    #[serde(default)]
    pub id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolResultEvent {
    /// Tool invocation ID this result corresponds to.
    #[serde(default)]
    pub id: Option<String>,
    /// Tool output content.
    pub content: serde_json::Value,
    /// Whether the tool call errored.
    #[serde(default)]
    pub is_error: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorEvent {
    /// Error message.
    pub message: String,
    /// Error code or type.
    #[serde(default)]
    pub code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionEvent {
    /// Session ID.
    pub id: String,
    /// Session title.
    #[serde(default)]
    pub title: Option<String>,
}

/// Parsed summary of an OpenCode run result.
#[derive(Debug, Clone)]
pub struct OpenCodeResult {
    /// The assistant's final text response.
    pub text: String,
    /// Session ID if available.
    pub session_id: Option<String>,
    /// Any tool calls that were made.
    pub tool_calls: Vec<ToolCall>,
    /// Whether the run completed successfully.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub input: serde_json::Value,
    pub result: Option<serde_json::Value>,
}

impl Default for OpenCodeResult {
    fn default() -> Self {
        Self {
            text: String::new(),
            session_id: None,
            tool_calls: vec![],
            success: true,
            error: None,
        }
    }
}

/// Parse a JSON event line from OpenCode output.
pub fn parse_event_line(line: &str) -> Option<OpenCodeEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    serde_json::from_str(trimmed).ok()
}

/// Accumulate events into a final result.
pub fn accumulate_events(events: &[OpenCodeEvent]) -> OpenCodeResult {
    let mut result = OpenCodeResult::default();
    let mut current_tool: Option<(String, serde_json::Value)> = None;

    for event in events {
        match event {
            OpenCodeEvent::Message(msg) if msg.role == "assistant" => {
                // Extract text content
                if let Some(text) = msg.content.as_str() {
                    result.text.push_str(text);
                } else if let Some(arr) = msg.content.as_array() {
                    for item in arr {
                        if let Some(obj) = item.as_object()
                            && obj.get("type").and_then(|t| t.as_str()) == Some("text")
                            && let Some(text) = obj.get("text").and_then(|t| t.as_str())
                        {
                            result.text.push_str(text);
                        }
                    }
                }
            }
            OpenCodeEvent::ToolUse(tool) => {
                current_tool = Some((tool.name.clone(), tool.input.clone()));
            }
            OpenCodeEvent::ToolResult(tool_result) => {
                if let Some((name, input)) = current_tool.take() {
                    result.tool_calls.push(ToolCall {
                        name,
                        input,
                        result: Some(tool_result.content.clone()),
                    });
                }
            }
            OpenCodeEvent::Error(err) => {
                result.success = false;
                result.error = Some(err.message.clone());
            }
            OpenCodeEvent::Session(session) => {
                result.session_id = Some(session.id.clone());
            }
            _ => {}
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_message_event() {
        let json = r#"{"type":"message","role":"assistant","content":"Hello world"}"#;
        let event = parse_event_line(json).unwrap();
        match event {
            OpenCodeEvent::Message(msg) => {
                assert_eq!(msg.role, "assistant");
                assert_eq!(msg.content.as_str().unwrap(), "Hello world");
            }
            _ => panic!("expected message event"),
        }
    }

    #[test]
    fn test_parse_tool_use_event() {
        let json = r#"{"type":"tool_use","name":"read_file","input":{"path":"/tmp/test.rs"}}"#;
        let event = parse_event_line(json).unwrap();
        match event {
            OpenCodeEvent::ToolUse(tool) => {
                assert_eq!(tool.name, "read_file");
            }
            _ => panic!("expected tool_use event"),
        }
    }

    #[test]
    fn test_parse_error_event() {
        let json = r#"{"type":"error","message":"something went wrong"}"#;
        let event = parse_event_line(json).unwrap();
        match event {
            OpenCodeEvent::Error(err) => {
                assert_eq!(err.message, "something went wrong");
            }
            _ => panic!("expected error event"),
        }
    }

    #[test]
    fn test_accumulate_events() {
        let events = vec![
            OpenCodeEvent::Session(SessionEvent {
                id: "sess_123".into(),
                title: None,
            }),
            OpenCodeEvent::ToolUse(ToolUseEvent {
                name: "read_file".into(),
                input: serde_json::json!({"path": "/tmp/test.rs"}),
                id: None,
            }),
            OpenCodeEvent::ToolResult(ToolResultEvent {
                id: None,
                content: serde_json::json!("file contents here"),
                is_error: false,
            }),
            OpenCodeEvent::Message(MessageEvent {
                role: "assistant".into(),
                content: serde_json::json!("I read the file."),
                model: None,
            }),
        ];

        let result = accumulate_events(&events);
        assert_eq!(result.text, "I read the file.");
        assert_eq!(result.session_id.as_deref(), Some("sess_123"));
        assert_eq!(result.tool_calls.len(), 1);
        assert_eq!(result.tool_calls[0].name, "read_file");
        assert!(result.success);
    }

    #[test]
    fn test_empty_line_returns_none() {
        assert!(parse_event_line("").is_none());
        assert!(parse_event_line("   ").is_none());
    }

    #[test]
    fn test_unknown_event_type() {
        let json = r#"{"type":"something_new","data":"value"}"#;
        let event = parse_event_line(json).unwrap();
        assert!(matches!(event, OpenCodeEvent::Unknown));
    }
}
