use tokio::sync::broadcast;

use crate::types::event::SystemEvent;

#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<SystemEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.tx.subscribe()
    }

    pub fn publish(&self, event: impl Into<SystemEvent>) {
        let _ = self.tx.send(event.into());
    }

    pub fn sender(&self) -> broadcast::Sender<SystemEvent> {
        self.tx.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::event::*;
    use crate::types::id::WorktreeId;

    #[test]
    fn test_publish_subscribe() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();

        let id = WorktreeId::new();
        bus.publish(SystemEvent::Worktree(WorktreeEvent::Created {
            id,
            path: "/tmp/test".into(),
            name: "test".into(),
        }));

        match rx.try_recv() {
            Ok(SystemEvent::Worktree(WorktreeEvent::Created { name, .. })) => {
                assert_eq!(name, "test");
            }
            _ => panic!("expected WorktreeEvent::Created"),
        }
    }

    #[test]
    fn test_multiple_subscribers() {
        let bus = EventBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        bus.publish(SystemEvent::System(SystemEventKind::Startup));

        assert!(rx1.try_recv().is_ok());
        assert!(rx2.try_recv().is_ok());
    }

    #[test]
    fn test_event_bus_clone() {
        let bus1 = EventBus::new(16);
        let bus2 = bus1.clone();
        let mut rx = bus1.subscribe();

        bus2.publish(SystemEvent::System(SystemEventKind::Shutdown));
        assert!(rx.try_recv().is_ok());
    }
}
