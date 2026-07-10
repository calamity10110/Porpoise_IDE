use std::path::Path;

use async_trait::async_trait;
use porpoise_core::error::Result;

use crate::types::{AgentKind, AgentOutput};

#[async_trait]
pub trait Agent: Send + Sync {
    fn kind(&self) -> AgentKind;
    fn binary_name(&self) -> &str;
    async fn spawn(&self, worktree: &Path) -> Result<Box<dyn AgentHandle>>;
}

#[async_trait]
pub trait AgentHandle: Send + Sync {
    fn pid(&self) -> Option<u32>;
    async fn read_output(&mut self) -> Result<Option<AgentOutput>>;
    async fn send_input(&mut self, text: &str) -> Result<()>;
    async fn interrupt(&mut self) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
    fn is_running(&self) -> bool;
}
