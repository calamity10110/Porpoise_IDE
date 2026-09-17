use std::{future::Future, pin::Pin};

use async_trait::async_trait;

use crate::{error::Result, types::event::SystemEvent};

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

    /// Filter events matching `predicate`, then handle each matching event.
    ///
    /// Default implementation: receives all events and calls `handle()` for events
    /// where `predicate(event)` returns `true`.
    async fn handle_map<F>(&self, _predicate: F) -> Pin<Box<dyn Future<Output = Result<()>> + Send + Sync>>
    where
        F: Fn(&SystemEvent) -> bool + Send + Sync + 'static,
    {
        Box::pin(async move { Ok(()) })
    }

    /// Handle events matching `predicate`, then run `next_handler` on each matching event.
    ///
    /// Default implementation: receives all events, filters via `predicate`,
    /// calls `handle()` for matching events, then calls `next_handler()`.
    async fn then_handle<F, G>(
        &self,
        _predicate: F,
        _next_handler: G,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + Sync>>
    where
        F: Fn(&SystemEvent) -> bool + Send + Sync + 'static,
        G: Fn(&SystemEvent) -> Result<()> + Send + Sync + 'static,
    {
        Box::pin(async move { Ok(()) })
    }
}
