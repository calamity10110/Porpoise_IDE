//! OpenCode adapter – understands `opencode` CLI protocol.

use std::path::Path;

use async_trait::async_trait;

use super::traits::{AdapterCapabilities, AgentAdapter, InputMode, OutputParser, ParsedEvent, RawOutput, ToolCall};
use crate::types::AgentKind;

pub struct OpenCodeAdapter;

impl OpenCodeAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OpenCodeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

struct OpenCodeOutputParser;

impl OutputParser for OpenCodeOutputParser {
    fn parse_line(&self, raw: &RawOutput) -> Vec<ParsedEvent> {
        let text = match raw {
            RawOutput::Stdout(s) => s,
            RawOutput::Stderr(s) => return vec![ParsedEvent::Error(s.clone())],
        };
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return vec![];
        }
        // OpenCode may output JSON tool calls
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed)
            && let Some(action) = val.get("action").and_then(|a| a.as_str())
        {
            let args: std::collections::HashMap<String, serde_json::Value> = val
                .get("params")
                .and_then(|p| serde_json::from_value(p.clone()).ok())
                .unwrap_or_default();
            return vec![ParsedEvent::ToolCall(ToolCall {
                tool_name: action.to_string(),
                arguments: args,
                call_id: val.get("id").and_then(|i| i.as_str()).map(String::from),
            })];
        }
        vec![ParsedEvent::Text(text.clone())]
    }
}

#[async_trait]
impl AgentAdapter for OpenCodeAdapter {
    fn kind(&self) -> AgentKind {
        AgentKind::OpenCode
    }

    fn binary_name(&self) -> &str {
        "opencode"
    }

    fn display_name(&self) -> &str {
        "OpenCode"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            streaming: true,
            session_resume: false,
            image_input: false,
            tool_calls: true,
            system_prompt: true,
            interruptible: true,
            max_context_tokens: 200_000,
            input_modes: vec![InputMode::Text, InputMode::File],
            description: "OpenCode terminal AI assistant".into(),
        }
    }

    fn build_args(&self, worktree: &Path, prompt: Option<&str>, extra_args: &[String]) -> Vec<String> {
        let mut args = vec![];
        if let Some(p) = prompt {
            args.push("--prompt".to_string());
            args.push(p.to_string());
        }
        args.push("--dir".to_string());
        args.push(worktree.to_string_lossy().to_string());
        args.extend(extra_args.iter().cloned());
        args
    }

    fn output_parser(&self) -> Box<dyn OutputParser> {
        Box::new(OpenCodeOutputParser)
    }
}
