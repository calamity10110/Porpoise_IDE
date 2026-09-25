//! Aider adapter – understands `aider` CLI protocol.

use std::path::Path;

use async_trait::async_trait;

use super::traits::{AdapterCapabilities, AgentAdapter, DiffPatch, InputMode, OutputParser, ParsedEvent, RawOutput};
use crate::types::AgentKind;

pub struct AiderAdapter;

impl AiderAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AiderAdapter {
    fn default() -> Self {
        Self::new()
    }
}

struct AiderOutputParser;

impl OutputParser for AiderOutputParser {
    fn parse_line(&self, raw: &RawOutput) -> Vec<ParsedEvent> {
        let text = match raw {
            RawOutput::Stdout(s) => s,
            RawOutput::Stderr(s) => return vec![ParsedEvent::Error(s.clone())],
        };
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return vec![];
        }
        // Aider outputs unified diffs for file changes
        if trimmed.starts_with("--- a/") || trimmed.starts_with("+++ b/") || trimmed.starts_with("@@") {
            return vec![ParsedEvent::Diff(DiffPatch {
                file_path: String::new(),
                old_content: None,
                new_content: text.clone(),
                language: None,
            })];
        }
        // Aider status lines
        if trimmed.starts_with("Applied edit to") || trimmed.starts_with("Added") {
            return vec![ParsedEvent::StatusChange(trimmed.to_string())];
        }
        vec![ParsedEvent::Text(text.clone())]
    }
}

#[async_trait]
impl AgentAdapter for AiderAdapter {
    fn kind(&self) -> AgentKind {
        AgentKind::Aider
    }

    fn binary_name(&self) -> &str {
        "aider"
    }

    fn display_name(&self) -> &str {
        "Aider"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            streaming: true,
            session_resume: true,
            image_input: false,
            tool_calls: false,
            system_prompt: true,
            interruptible: true,
            max_context_tokens: 128_000,
            input_modes: vec![InputMode::Text, InputMode::File],
            description: "Aider AI pair programming with git integration".into(),
        }
    }

    fn build_args(&self, worktree: &Path, prompt: Option<&str>, extra_args: &[String]) -> Vec<String> {
        let mut args = vec!["--yes".to_string()];
        args.push("--no-auto-commits".to_string());
        args.push("--no-git".to_string());
        args.push("--no-pretty".to_string());
        args.push("--no-stream".to_string());
        if let Some(p) = prompt {
            args.push("--message".to_string());
            args.push(p.to_string());
        }
        args.push("--cwd".to_string());
        args.push(worktree.to_string_lossy().to_string());
        args.extend(extra_args.iter().cloned());
        args
    }

    fn output_parser(&self) -> Box<dyn OutputParser> {
        Box::new(AiderOutputParser)
    }
}
