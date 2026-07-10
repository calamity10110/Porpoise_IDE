use std::path::PathBuf;

use porpoise_core::types::id::AgentId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentKind {
    ClaudeCode,
    Codex,
    Gemini,
    Custom(String),
}

impl std::fmt::Display for AgentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentKind::ClaudeCode => write!(f, "claude"),
            AgentKind::Codex => write!(f, "codex"),
            AgentKind::Gemini => write!(f, "gemini"),
            AgentKind::Custom(name) => write!(f, "{name}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub id: AgentId,
    pub kind: AgentKind,
    pub pid: Option<u32>,
    pub status: AgentStatus,
    pub worktree_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    Detected,
    Spawning,
    Running,
    Thinking,
    Waiting,
    Error(String),
    Completed(i32),
    Killed,
}

#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub text: String,
    pub output_type: OutputType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputType {
    Stdout,
    Stderr,
    Status,
    Thinking,
}

#[derive(Debug, Clone)]
pub struct AgentManifest {
    pub name: String,
    pub binary: String,
    pub kind: AgentKind,
    pub detected: bool,
    pub version: Option<String>,
}
