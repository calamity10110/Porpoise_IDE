use async_trait::async_trait;

use crate::error::Result;
use crate::types::event::SystemEvent;

/// Reacts to system events published on the `EventBus`.
///
/// Implementations should handle a single responsibility — e.g. logging,
/// state updates, or forwarding to WebSocket clients. The `interested_in()`
/// method can filter events to reduce unnecessary wake-ups.
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle a single system event. Return `Ok(())` on success.
    async fn handle(&self, event: &SystemEvent) -> Result<()>;

    /// Returns the list of event type names this handler is interested in.
    ///
    /// Return an empty vec (the default) to receive all events.
    fn interested_in(&self) -> Vec<String> {
        vec![]
    }
}
