//! OpenCode agent integration for Porpoise.
//!
//! This crate provides a first-class [`Agent`] implementation for OpenCode,
//! wrapping the `opencode` CLI with proper JSON output parsing, session
//! management, and configuration support.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use porpoise_opencode::{OpenCodeAgent, Agent};
//!
//! # async fn example() -> porpoise_core::Result<()> {
//! let agent = OpenCodeAgent::new();
//! let worktree = std::path::Path::new(".");
//! let mut handle = agent.spawn(worktree).await?;
//!
//! let output = handle.read_output().await?;
//! if let Some(out) = output {
//!     println!("{}", out.text);
//! }
//! # Ok(())
//! # }
//! ```

pub mod agent;
pub mod config;
pub mod protocol;

pub use agent::{OpenCodeAgent, OpenCodeHandle};
pub use config::{OpenCodeConfig, OpenCodeOutputFormat, OpenCodeServerConfig};
// Re-export agent traits for convenience
pub use porpoise_agent::traits::{Agent, AgentHandle};
pub use porpoise_agent::types::AgentKind;
pub use protocol::{
    MessageEvent, OpenCodeEvent, OpenCodeResult, ToolCall, ToolResultEvent, ToolUseEvent, accumulate_events,
    parse_event_line,
};
