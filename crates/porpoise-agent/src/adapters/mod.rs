//! Agent Adapter Layer
//!
//! Provides agent-specific adapters that understand each agent's CLI interface,
//! output format, and capabilities. The `AgentAdapter` trait extends the base
//! `Agent` trait with protocol-specific behavior.

pub mod aider;
pub mod claude;
pub mod codex;
pub mod opencode;
pub mod registry;
pub mod traits;

pub use registry::AdapterRegistry;
pub use traits::{AdapterCapabilities, AgentAdapter, OutputParser};
