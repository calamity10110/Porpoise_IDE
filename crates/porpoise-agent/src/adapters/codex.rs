//! OpenAI Codex adapter – understands `codex` CLI protocol.

use std::path::Path;

use async_trait::async_trait;

use super::traits::{AdapterCapabilities, AgentAdapter, InputMode, OutputParser, ParsedEvent, RawOutput, ToolCall};
use crate::types::AgentKind;

pub struct CodexAdapter;

impl CodexAdapter {
    pub fn new() -> Self {
        Self
    }
}

struct CodexOutputParser;

impl OutputParser for CodexOutputParser {
    fn parse_line(&self, raw: &RawOutput) -> Vec<ParsedEvent> {
        let text = match raw {
            RawOutput::Stdout(s) => s,
            RawOutput::Stderr(s) => return vec![ParsedEvent::Error(s.clone())],
        };
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return vec![];
        }
        // Codex outputs JSON for tool calls
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(call) = val.get("function_call") {
                let name = call
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let args: std::collections::HashMap<String, serde_json::Value> = call
                    .get("arguments")
                    .and_then(|a| serde_json::from_value(a.clone()).ok())
                    .unwrap_or_default();
                return vec![ParsedEvent::ToolCall(ToolCall {
                    tool_name: name,
                    arguments: args,
                    call_id: val.get("id").and_then(|i| i.as_str()).map(String::from),
                })];
            }
        }
        vec![ParsedEvent::Text(text.clone())]
    }
}

#[async_trait]
impl AgentAdapter for CodexAdapter {
    fn kind(&self) -> AgentKind {
        AgentKind::Codex
    }

    fn binary_name(&self) -> &str {
        "codex"
    }

    fn display_name(&self) -> &str {
        "OpenAI Codex"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            streaming: true,
            session_resume: false,
            image_input: false,
            tool_calls: true,
            system_prompt: true,
            interruptible: true,
            max_context_tokens: 128_000,
            input_modes: vec![InputMode::Text, InputMode::File],
            description: "OpenAI Codex CLI with function calling".into(),
        }
    }

    fn build_args(&self, worktree: &Path, prompt: Option<&str>, extra_args: &[String]) -> Vec<String> {
        let mut args = vec![];
        if let Some(p) = prompt {
            args.push(p.to_string());
        }
        args.push("--quiet".to_string());
        args.push("--directory".to_string());
        args.push(worktree.to_string_lossy().to_string());
        args.extend(extra_args.iter().cloned());
        args
    }

    fn output_parser(&self) -> Box<dyn OutputParser> {
        Box::new(CodexOutputParser)
    }
}
