use porpoise_core::{
    bus::EventBus,
    types::{
        event::{AgentEvent, OutputKind, SystemEvent},
        id::AgentId,
    },
};

pub struct HookServer {
    event_bus: EventBus,
    agent_id: AgentId,
}

impl HookServer {
    pub fn new(event_bus: EventBus, agent_id: AgentId) -> Self {
        Self { event_bus, agent_id }
    }

    pub fn process_output(&self, text: &str) {
        let kind = if text.starts_with("```") {
            OutputKind::Code
        } else {
            OutputKind::Text
        };
        self.event_bus.publish(SystemEvent::Agent(AgentEvent::Output {
            id: self.agent_id,
            text: text.to_string(),
            kind,
        }));
    }
}
