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

    /// Handle every event in `events` for which `predicate(event)` is `true`.
    ///
    /// Returns the number of events handled. Stops on the first error from
    /// `handle()`, propagating it.
    async fn handle_map<F>(&self, events: &[SystemEvent], predicate: F) -> Result<usize>
    where
        F: Fn(&SystemEvent) -> bool + Send + Sync,
    {
        let mut handled = 0;
        for event in events {
            if predicate(event) {
                self.handle(event).await?;
                handled += 1;
            }
        }
        Ok(handled)
    }

    /// Handle every event matching `predicate`, then run `next_handler` on
    /// each handled event, in order.
    ///
    /// Returns the number of events processed. Stops on the first error from
    /// either `handle()` or `next_handler`, propagating it.
    async fn then_handle<F, G>(&self, events: &[SystemEvent], predicate: F, next_handler: G) -> Result<usize>
    where
        F: Fn(&SystemEvent) -> bool + Send + Sync,
        G: Fn(&SystemEvent) -> Result<()> + Send + Sync,
    {
        let mut handled = 0;
        for event in events {
            if predicate(event) {
                self.handle(event).await?;
                next_handler(event)?;
                handled += 1;
            }
        }
        Ok(handled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::event::{AgentEvent, SystemEventKind};

    struct RecordingHandler {
        handled: std::sync::Mutex<Vec<String>>,
        fail_on: Option<String>,
    }

    impl RecordingHandler {
        fn new() -> Self {
            Self {
                handled: std::sync::Mutex::new(Vec::new()),
                fail_on: None,
            }
        }

        fn failing_on(marker: &str) -> Self {
            Self {
                handled: std::sync::Mutex::new(Vec::new()),
                fail_on: Some(marker.to_string()),
            }
        }

        fn handled(&self) -> Vec<String> {
            self.handled.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl EventHandler for RecordingHandler {
        async fn handle(&self, event: &SystemEvent) -> Result<()> {
            let name = describe(event);
            if self.fail_on.as_deref() == Some(name.as_str()) {
                return Err(crate::error::PorpoiseError::Internal(format!("fail on {name}")));
            }
            self.handled.lock().unwrap().push(name);
            Ok(())
        }
    }

    fn describe(event: &SystemEvent) -> String {
        match event {
            SystemEvent::System(SystemEventKind::Startup) => "startup".into(),
            SystemEvent::System(SystemEventKind::Shutdown) => "shutdown".into(),
            SystemEvent::Agent(_) => "agent".into(),
            _ => "other".into(),
        }
    }

    fn sample_events() -> Vec<SystemEvent> {
        vec![
            SystemEvent::System(SystemEventKind::Startup),
            SystemEvent::Agent(AgentEvent::Exited {
                id: crate::types::id::AgentId::new(),
                exit_code: 0,
            }),
            SystemEvent::System(SystemEventKind::Shutdown),
        ]
    }

    fn is_system(event: &SystemEvent) -> bool {
        matches!(event, SystemEvent::System(_))
    }

    #[tokio::test]
    async fn handle_map_filters_and_counts() {
        let handler = RecordingHandler::new();
        let events = sample_events();

        let n = handler.handle_map(&events, is_system).await.unwrap();

        assert_eq!(n, 2);
        assert_eq!(handler.handled(), vec!["startup".to_string(), "shutdown".to_string()]);
    }

    #[tokio::test]
    async fn handle_map_no_matches() {
        let handler = RecordingHandler::new();
        let events = sample_events();

        let n = handler.handle_map(&events, |_| false).await.unwrap();

        assert_eq!(n, 0);
        assert!(handler.handled().is_empty());
    }

    #[tokio::test]
    async fn handle_map_propagates_error() {
        let handler = RecordingHandler::failing_on("startup");
        let events = sample_events();

        let result = handler.handle_map(&events, is_system).await;

        assert!(result.is_err());
        assert!(handler.handled().is_empty());
    }

    #[tokio::test]
    async fn then_handle_runs_handle_then_next() {
        let handler = RecordingHandler::new();
        let events = sample_events();
        let order: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

        let n = handler
            .then_handle(&events, is_system, |event| {
                order.lock().unwrap().push(format!("next:{}", describe(event)));
                Ok(())
            })
            .await
            .unwrap();

        assert_eq!(n, 2);
        assert_eq!(
            order.lock().unwrap().clone(),
            vec!["next:startup".to_string(), "next:shutdown".to_string()]
        );
        assert_eq!(handler.handled(), vec!["startup".to_string(), "shutdown".to_string()]);
    }

    #[tokio::test]
    async fn then_handle_propagates_next_handler_error() {
        let handler = RecordingHandler::new();
        let events = sample_events();

        let result: Result<usize> = handler
            .then_handle(&events, is_system, |_| {
                Err(crate::error::PorpoiseError::Internal("next failed".into()))
            })
            .await;

        assert!(result.is_err());
        assert_eq!(handler.handled(), vec!["startup".to_string()]);
    }
}
