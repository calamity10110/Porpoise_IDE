//! Workflow graph data model — nodes, edges, and the graph container.

use serde::{Deserialize, Serialize};

/// Unique node identifier.
pub type NodeId = String;
/// Unique edge identifier.
pub type EdgeId = String;

/// Canvas position for drag-and-drop layout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// The kind of node in the workflow graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum NodeKind {
    /// An AI agent node (Claude, Codex, etc.)
    Agent {
        agent_kind: String,
        prompt: Option<String>,
    },
    /// A shell command node.
    Command { command: String },
    /// A conditional branch node.
    Condition { expression: String },
    /// A merge/join node that waits for multiple inputs.
    Merge,
    /// Start entry point.
    Start,
    /// End exit point.
    End,
}

/// A single node in the workflow graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub label: String,
    pub kind: NodeKind,
    pub position: Position,
    #[serde(default)]
    pub config: serde_json::Value,
}

/// A directed edge connecting two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub condition: Option<String>,
}

/// The complete workflow graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl WorkflowGraph {
    /// Create an empty graph.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Find a node by ID.
    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Find an edge by ID.
    pub fn get_edge(&self, id: &str) -> Option<&Edge> {
        self.edges.iter().find(|e| e.id == id)
    }

    /// Remove a node and all connected edges.
    pub fn remove_node(&mut self, id: &str) {
        self.nodes.retain(|n| n.id != id);
        self.edges.retain(|e| e.source != id && e.target != id);
    }

    /// Remove an edge.
    pub fn remove_edge(&mut self, id: &str) {
        self.edges.retain(|e| e.id != id);
    }

    /// Get all outgoing edges from a node.
    pub fn outgoing(&self, node_id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.source == node_id).collect()
    }

    /// Get all incoming edges to a node.
    pub fn incoming(&self, node_id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.target == node_id).collect()
    }

    /// Topological sort of nodes (execution order).
    pub fn topo_sort(&self) -> Result<Vec<&NodeId>, String> {
        let mut in_degree: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for node in &self.nodes {
            in_degree.entry(&node.id).or_insert(0);
        }
        for edge in &self.edges {
            *in_degree.entry(&edge.target).or_insert(0) += 1;
        }

        let mut queue: std::collections::VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut sorted = Vec::new();
        while let Some(current) = queue.pop_front() {
            sorted.push(current);
            for edge in self.outgoing(current) {
                let deg = in_degree.get_mut(edge.target.as_str()).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(&edge.target);
                }
            }
        }

        if sorted.len() == self.nodes.len() {
            Ok(sorted.iter().map(|s| self.nodes.iter().find(|n| n.id == *s).unwrap()).map(|n| &n.id).collect())
        } else {
            Err("Cycle detected in workflow graph".to_string())
        }
    }
}

impl Default for WorkflowGraph {
    fn default() -> Self {
        Self::new()
    }
}
