//! Workflow container — wraps a graph with metadata and persistence.

use serde::{Deserialize, Serialize};

use crate::graph::WorkflowGraph;

/// Metadata for a saved workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMeta {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: u32,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<String>,
}

/// A complete workflow with metadata and graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub meta: WorkflowMeta,
    pub graph: WorkflowGraph,
}

impl Workflow {
    /// Create a new workflow with a given name.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            meta: WorkflowMeta {
                id: id.into(),
                name: name.into(),
                description: None,
                version: 1,
                created_at: now.clone(),
                updated_at: now,
                tags: Vec::new(),
            },
            graph: WorkflowGraph::new(),
        }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    /// Bump the version and update timestamp.
    pub fn bump_version(&mut self) {
        self.meta.version += 1;
        self.meta.updated_at = chrono::Utc::now().to_rfc3339();
    }
}
