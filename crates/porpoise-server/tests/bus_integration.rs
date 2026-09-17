//! Cross-crate integration tests for EventBus (porpoise-core).

use porpoise_core::{
    bus::EventBus,
    types::{
        event::{
            AgentEvent, AgentStatusKind, NotificationSeverity, OutputKind, SystemEvent,
            SystemEventKind, WorktreeEvent,
        },
        id::{AgentId, WorktreeId},
    },
};

#[test]
fn test_bus_cross_crate_subscribe_publish() {
    let bus = EventBus::new(64);
    let mut rx = bus.subscribe();

    let wt_id = WorktreeId::new();
    bus.publish(SystemEvent::Worktree(WorktreeEvent::Created {
        id: wt_id,
        path: "/tmp/test".into(),
        name: "test".into(),
    }));

    match rx.try_recv() {
        Ok(SystemEvent::Worktree(WorktreeEvent::Created { name, .. })) => {
            assert_eq!(name, "test");
        }
        other => panic!("expected WorktreeEvent::Created, got {other:?}"),
    }
}

#[test]
fn test_bus_agent_event_roundtrip() {
    let bus = EventBus::new(64);
    let mut rx = bus.subscribe();

    let agent_id = AgentId::new();
    bus.publish(SystemEvent::Agent(AgentEvent::Spawned {
        id: agent_id,
        kind: "claude".into(),
        pid: 42,
    }));

    match rx.try_recv() {
        Ok(SystemEvent::Agent(AgentEvent::Spawned { kind, pid, .. })) => {
            assert_eq!(kind, "claude");
            assert_eq!(pid, 42);
        }
        other => panic!("expected AgentEvent::Spawned, got {other:?}"),
    }
}

#[test]
fn test_bus_multiple_event_types() {
    let bus = EventBus::new(64);
    let mut rx = bus.subscribe();

    // Publish different event types
    bus.publish(SystemEvent::System(SystemEventKind::Startup));
    bus.publish(SystemEvent::Agent(AgentEvent::StatusChanged {
        id: AgentId::new(),
        status: AgentStatusKind::Running,
    }));
    bus.publish(SystemEvent::Agent(AgentEvent::Output {
        id: AgentId::new(),
        text: "hello".into(),
        kind: OutputKind::Text,
    }));
    bus.publish(SystemEvent::System(SystemEventKind::Notification {
        title: "test".into(),
        body: "body".into(),
        severity: NotificationSeverity::Info,
    }));

    // All 4 events should be received in order
    assert!(matches!(rx.try_recv(), Ok(SystemEvent::System(SystemEventKind::Startup))));
    assert!(matches!(rx.try_recv(), Ok(SystemEvent::Agent(AgentEvent::StatusChanged { .. }))));
    assert!(matches!(rx.try_recv(), Ok(SystemEvent::Agent(AgentEvent::Output { .. }))));
    assert!(matches!(rx.try_recv(), Ok(SystemEvent::System(SystemEventKind::Notification { .. }))));
    assert!(rx.try_recv().is_err()); // no more events
}

#[test]
fn test_bus_independent_subscribers() {
    let bus = EventBus::new(64);
    let mut rx1 = bus.subscribe();
    let mut rx2 = bus.subscribe();

    bus.publish(SystemEvent::System(SystemEventKind::Shutdown));

    // Both subscribers receive independently
    assert!(rx1.try_recv().is_ok());
    assert!(rx2.try_recv().is_ok());
}
