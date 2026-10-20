use async_trait::async_trait;
use serde::Serialize;

use crate::error::Result;

/// A typed CLI command that can be routed to a handler.
///
/// Implement this trait for each CLI command. The associated `Output` type
/// controls serialization for the response. Commands receive a reference to
/// `AppState` for access to shared state and the event bus.
#[async_trait]
#[allow(clippy::double_must_use)] // async_trait generates a bare #[must_use] on the boxed future
pub trait Command: Send + Sync {
    /// The serializable output type produced by this command.
    type Output: Serialize;

    /// Returns the command name, used for routing (e.g. "worktree.create").
    fn name(&self) -> &'static str;

    /// Executes the command against the given application state.
    async fn execute(self, state: &crate::state::AppState) -> Result<Self::Output>;
}
