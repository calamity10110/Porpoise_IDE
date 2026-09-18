//! Claude Code adapter – understands `claude` CLI protocol.

use std::path::Path;

use async_trait::async_trait;

use super::traits::{AdapterCapabilities, AgentAdapter, InputMode, OutputParser, ParsedEvent, RawOutput, ToolCall};
use crate::types::AgentKind;

pub struct ClaudeAdapter;

impl ClaudeAdapter {
    pub fn new() -> Self {
        Self
    }
}

struct ClaudeOutputParser;

impl OutputParser for ClaudeOutputParser {
    fn parse_line(&self, raw: &RawOutput) -> Vec<ParsedEvent> {
        let text = match raw {
            RawOutput::Stdout(s) => s,
            RawOutput::Stderr(s) => {
                return vec![ParsedEvent::Error(s.clone())];
            }
        };
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return vec![];
        }
        // Claude Code outputs JSON lines for tool calls
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(tool) = val.get("tool").and_then(|t| t.as_str()) {
                return vec![ParsedEvent::ToolCall(ToolCall {
                    tool_name: tool.to_string(),
                    arguments: val
                        .get("arguments")
                        .cloned()
                        .and_then(|a| serde_json::from_value(a).ok())
                        .unwrap_or_default(),
                    call_id: val.get("id").and_then(|i| i.as_str()).map(String::from),
                })];
            }
        }
        // Thinking markers
        if trimmed.starts_with("<thinking>") || trimmed.starts_with("[thinking]") {
            return vec![ParsedEvent::Thinking(trimmed.to_string())];
        }
        vec![ParsedEvent::Text(text.clone())]
    }
}

#[async_trait]
impl AgentAdapter for ClaudeAdapter {
    fn kind(&self) -> AgentKind {
        AgentKind::ClaudeCode
    }

    fn binary_name(&self) -> &str {
        "claude"
    }

    fn display_name(&self) -> &str {
        "Claude Code"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            streaming: true,
            session_resume: true,
            image_input: true,
            tool_calls: true,
            system_prompt: true,
            interruptible: true,
            max_context_tokens: 200_000,
            input_modes: vec![InputMode::Text, InputMode::Image, InputMode::File],
            description: "Anthropic Claude Code CLI with tool use and streaming".into(),
        }
    }

    fn build_args(&self, worktree: &Path, prompt: Option<&str>, extra_args: &[String]) -> Vec<String> {
        let mut args = vec!["--print".to_string()];
        if let Some(p) = prompt {
            args.push("--prompt".to_string());
            args.push(p.to_string());
        }
        args.push("--cwd".to_string());
        args.push(worktree.to_string_lossy().to_string());
        args.extend(extra_args.iter().cloned());
        args
    }

    fn output_parser(&self) -> Box<dyn OutputParser> {
        Box::new(ClaudeOutputParser)
    }

    fn version(&self) -> Option<String> {
        std::process::Command::new("claude")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    }
}
