use porpoise_core::{
    bus::EventBus,
    types::{
        event::{AgentEvent, OutputKind, SystemEvent},
        id::AgentId,
    },
};

pub struct HookServer {
    event_bus: EventBus,
}

impl HookServer {
    pub fn new(event_bus: EventBus) -> Self {
        Self { event_bus }
    }

    pub fn process_output(&self, text: &str) {
        let kind = if text.starts_with("```") {
            OutputKind::Code
        } else {
            OutputKind::Text
        };
        self.event_bus.publish(SystemEvent::Agent(AgentEvent::Output {
            id: AgentId::new(),
            text: text.to_string(),
            kind,
        }));
    }
}
