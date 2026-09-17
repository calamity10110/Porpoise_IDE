//! Adapter registry – maps AgentKind to the correct adapter.

use std::collections::HashMap;

use crate::types::AgentKind;

use super::traits::AgentAdapter;
use super::{aider::AiderAdapter, claude::ClaudeAdapter, codex::CodexAdapter, opencode::OpenCodeAdapter};

/// Registry of all known agent adapters.
pub struct AdapterRegistry {
    adapters: HashMap<AgentKind, Box<dyn AgentAdapter + Send + Sync>>,
}

impl AdapterRegistry {
    /// Create a registry with all built-in adapters.
    pub fn with_defaults() -> Self {
        let mut adapters: HashMap<AgentKind, Box<dyn AgentAdapter + Send + Sync>> = HashMap::new();
        adapters.insert(AgentKind::ClaudeCode, Box::new(ClaudeAdapter::new()));
        adapters.insert(AgentKind::Codex, Box::new(CodexAdapter::new()));
        adapters.insert(AgentKind::Aider, Box::new(AiderAdapter::new()));
        adapters.insert(AgentKind::OpenCode, Box::new(OpenCodeAdapter::new()));
        Self { adapters }
    }

    /// Get adapter for a given kind.
    pub fn get(&self, kind: &AgentKind) -> Option<&(dyn AgentAdapter + Send + Sync)> {
        self.adapters.get(kind).map(|a| a.as_ref())
    }

    /// List all registered kinds.
    pub fn registered_kinds(&self) -> Vec<AgentKind> {
        self.adapters.keys().cloned().collect()
    }

    /// List only adapters whose binaries are detected on PATH.
    /// `which::which` is a fast filesystem check (no process spawn),
    /// so a per-adapter timeout is unnecessary.
    pub fn available(&self) -> Vec<&AgentKind> {
        self.adapters
            .iter()
            .filter(|(_, a)| a.is_available())
            .map(|(k, _)| k)
            .collect()
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}
