//! Workflow validation — checks graph integrity before execution.

use crate::graph::WorkflowGraph;

/// A single validation error.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub node_id: Option<String>,
}

/// Result of validating a workflow.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
}

/// Validate a workflow graph for correctness.
pub fn validate_workflow(graph: &WorkflowGraph) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Must have at least one Start node
    let start_count = graph
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, crate::graph::NodeKind::Start))
        .count();
    if start_count == 0 {
        errors.push(ValidationError {
            code: "NO_START".into(),
            message: "Workflow must have at least one Start node".into(),
            node_id: None,
        });
    } else if start_count > 1 {
        warnings.push(ValidationError {
            code: "MULTIPLE_STARTS".into(),
            message: "Workflow has multiple Start nodes".into(),
            node_id: None,
        });
    }

    // Must have at least one End node
    let end_count = graph
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, crate::graph::NodeKind::End))
        .count();
    if end_count == 0 {
        errors.push(ValidationError {
            code: "NO_END".into(),
            message: "Workflow must have at least one End node".into(),
            node_id: None,
        });
    }

    // Check for orphan nodes (no edges)
    for node in &graph.nodes {
        let has_incoming = graph.edges.iter().any(|e| e.target == node.id);
        let has_outgoing = graph.edges.iter().any(|e| e.source == node.id);

        if !has_incoming
            && !has_outgoing
            && !matches!(node.kind, crate::graph::NodeKind::Start | crate::graph::NodeKind::End)
        {
            warnings.push(ValidationError {
                code: "ORPHAN_NODE".into(),
                message: format!("Node '{}' has no connections", node.label),
                node_id: Some(node.id.clone()),
            });
        }
    }

    // Check for cycles via topo sort
    if let Err(msg) = graph.topo_sort() {
        errors.push(ValidationError {
            code: "CYCLE".into(),
            message: msg,
            node_id: None,
        });
    }

    // Check edges reference valid nodes
    for edge in &graph.edges {
        if !graph.nodes.iter().any(|n| n.id == edge.source) {
            errors.push(ValidationError {
                code: "INVALID_EDGE_SOURCE".into(),
                message: format!("Edge '{}' references missing source '{}'", edge.id, edge.source),
                node_id: Some(edge.source.clone()),
            });
        }
        if !graph.nodes.iter().any(|n| n.id == edge.target) {
            errors.push(ValidationError {
                code: "INVALID_EDGE_TARGET".into(),
                message: format!("Edge '{}' references missing target '{}'", edge.id, edge.target),
                node_id: Some(edge.target.clone()),
            });
        }
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}
