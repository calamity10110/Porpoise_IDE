//! Core adapter traits that extend the base Agent interface with
//! agent-specific protocol knowledge.

use std::collections::HashMap;
use std::path::Path;

use async_trait::async_trait;
use porpoise_core::error::Result;
use serde::{Deserialize, Serialize};

use crate::types::{AgentKind, AgentOutput};

// ---------------------------------------------------------------------------
// AdapterCapabilities – what a given agent backend can do
// ---------------------------------------------------------------------------

/// Describes the capabilities of a specific agent backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterCapabilities {
    /// Agent supports streaming partial output (token-by-token or chunked).
    pub streaming: bool,
    /// Agent supports resuming a previous session/conversation.
    pub session_resume: bool,
    /// Agent can accept image inputs (screenshots, diagrams).
    pub image_input: bool,
    /// Agent can produce structured tool calls (file edits, shell commands).
    pub tool_calls: bool,
    /// Agent supports system-level prompt injection / system prompt.
    pub system_prompt: bool,
    /// Agent supports interrupting a running generation.
    pub interruptible: bool,
    /// Maximum context window in tokens (0 = unknown/unlimited).
    pub max_context_tokens: u32,
    /// Supported input modalities.
    pub input_modes: Vec<InputMode>,
    /// Human-readable description.
    pub description: String,
}

/// Input modalities an agent can accept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputMode {
    Text,
    Image,
    File,
    Diff,
    Url,
}

impl Default for AdapterCapabilities {
    fn default() -> Self {
        Self {
            streaming: false,
            session_resume: false,
            image_input: false,
            tool_calls: false,
            system_prompt: false,
            interruptible: true,
            max_context_tokens: 0,
            input_modes: vec![InputMode::Text],
            description: String::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// OutputParser – protocol-specific output parsing
// ---------------------------------------------------------------------------

/// Raw bytes or text received from an agent process before parsing.
#[derive(Debug, Clone)]
pub enum RawOutput {
    Stdout(String),
    Stderr(String),
}

/// Structured event extracted from raw agent output.
#[derive(Debug, Clone)]
pub enum ParsedEvent {
    /// A text chunk to display to the user.
    Text(String),
    /// Agent is thinking / reasoning (not shown to user directly).
    Thinking(String),
    /// Agent wants to execute a tool call.
    ToolCall(ToolCall),
    /// Agent produced a diff / file change.
    Diff(DiffPatch),
    /// Agent reported an error.
    Error(String),
    /// Agent signaled it is done.
    Done { exit_code: Option<i32> },
    /// Agent status changed (e.g. "Applied edit to ...").
    StatusChange(String),
    /// Agent is waiting for user input.
    WaitingForInput,
    /// Unrecognised output, pass through as text.
    Raw(String),
}

/// A structured tool call requested by the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub arguments: HashMap<String, serde_json::Value>,
    pub call_id: Option<String>,
}

/// A diff/patch produced by the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffPatch {
    pub file_path: String,
    pub old_content: Option<String>,
    pub new_content: String,
    pub language: Option<String>,
}

/// Trait for parsing agent-specific output formats into structured events.
pub trait OutputParser: Send + Sync {
    /// Parse a raw output line into one or more structured events.
    fn parse_line(&self, raw: &RawOutput) -> Vec<ParsedEvent>;

    /// Called when the agent process exits. Returns any final events.
    fn on_exit(&self, exit_code: i32) -> Vec<ParsedEvent> {
        vec![ParsedEvent::Done {
            exit_code: Some(exit_code),
        }]
    }
}

// ---------------------------------------------------------------------------
// AgentAdapter – the main adapter trait
// ---------------------------------------------------------------------------

/// Extension trait that adds agent-specific protocol knowledge on top of
/// the base `Agent` trait.
///
/// Each agent backend (Claude Code, Codex, Aider, OpenCode) implements this
/// trait to provide:
/// - CLI argument construction
/// - Output parsing
/// - Capability advertisement
/// - Environment setup
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    /// The agent kind this adapter handles.
    fn kind(&self) -> AgentKind;

    /// The binary name to look up in PATH.
    fn binary_name(&self) -> &str;

    /// Human-readable display name.
    fn display_name(&self) -> &str;

    /// What this agent can do.
    fn capabilities(&self) -> AdapterCapabilities;

    /// Build the CLI arguments for spawning this agent.
    ///
    /// `worktree` is the working directory. `prompt` is the initial prompt
    /// to send. `extra_args` are user-supplied overrides.
    fn build_args(
        &self,
        worktree: &Path,
        prompt: Option<&str>,
        extra_args: &[String],
    ) -> Vec<String>;

    /// Build environment variables needed for this agent.
    /// Returns (key, value) pairs to add to the process environment.
    fn build_env(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// Get the output parser for this agent.
    fn output_parser(&self) -> Box<dyn OutputParser>;

    /// Format a prompt for this agent's protocol.
    /// Some agents need special framing (e.g. system prompt prefix).
    fn format_prompt(&self, prompt: &str, system: Option<&str>) -> String {
        match system {
            Some(sys) => format!("{sys}\n\n{prompt}"),
            None => prompt.to_string(),
        }
    }

    /// Check if the agent binary is available on this system.
    fn is_available(&self) -> bool {
        if let Ok(path) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path) {
                let candidate = dir.join(self.binary_name());
                if candidate.exists() {
                    return true;
                }
                #[cfg(windows)]
                {
                    let with_exe = dir.join(format!("{}.exe", self.binary_name()));
                    if with_exe.exists() {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Get the version string, if detectable.
    fn version(&self) -> Option<String> {
        None
    }
}
