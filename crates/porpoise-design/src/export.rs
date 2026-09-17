//! Export — serialize workflows to various formats.
//!
//! Supported formats:
//! - **JSON** — full fidelity, round-trippable via `Workflow::from_json`.
//! - **DOT** — Graphviz DOT language for static visualization.
//! - **SVG** — minimal SVG rendering of the graph (nodes as rectangles,
//!   edges as lines).

use std::fmt::Write;

use crate::{
    graph::{NodeKind, WorkflowGraph},
    workflow::Workflow,
};

/// Export errors.
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("format error: {0}")]
    Fmt(#[from] std::fmt::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Export a workflow to pretty-printed JSON.
pub fn to_json(workflow: &Workflow) -> Result<String, ExportError> {
    Ok(workflow.to_json()?)
}

/// Export a workflow graph to Graphviz DOT format.
pub fn to_dot(graph: &WorkflowGraph) -> Result<String, ExportError> {
    let mut out = String::new();
    writeln!(out, "digraph workflow {{")?;
    writeln!(out, "  rankdir=LR;")?;
    writeln!(out, "  node [shape=box, style=rounded, fontname=\"Helvetica\"];")?;
    writeln!(out)?;

    for node in &graph.nodes {
        let (shape, color) = match &node.kind {
            NodeKind::Start => ("oval", "#2ECC71"),
            NodeKind::End => ("oval", "#E74C3C"),
            NodeKind::Agent { .. } => ("box", "#4A90D9"),
            NodeKind::Command { .. } => ("box", "#50C878"),
            NodeKind::Condition { .. } => ("diamond", "#F5A623"),
            NodeKind::Merge => ("hexagon", "#9B59B6"),
        };
        writeln!(
            out,
            "  \"{}\" [label=\"{}\", shape={}, fillcolor=\"{}\", style=\"filled,rounded\"];",
            node.id, node.label, shape, color
        )?;
    }

    writeln!(out)?;
    for edge in &graph.edges {
        let label = edge.label.as_deref().unwrap_or("");
        if label.is_empty() {
            writeln!(out, "  \"{}\" -> \"{}\";", edge.source, edge.target)?;
        } else {
            writeln!(
                out,
                "  \"{}\" -> \"{}\" [label=\"{}\"];",
                edge.source, edge.target, label
            )?;
        }
    }

    writeln!(out, "}}")?;
    Ok(out)
}

/// Export a workflow graph to a minimal SVG.
///
/// Nodes are rendered as rounded rectangles; edges as straight lines.
/// This is a *model-level* export — the frontend can produce richer SVGs.
pub fn to_svg(graph: &WorkflowGraph) -> Result<String, ExportError> {
    let node_w: f64 = 180.0;
    let node_h: f64 = 60.0;
    let pad: f64 = 40.0;

    // Compute bounding box
    let mut max_x: f64 = 0.0;
    let mut max_y: f64 = 0.0;
    for node in &graph.nodes {
        let right = node.position.x + node_w + pad;
        let bottom = node.position.y + node_h + pad;
        if right > max_x {
            max_x = right;
        }
        if bottom > max_y {
            max_y = bottom;
        }
    }
    if max_x < 200.0 {
        max_x = 200.0;
    }
    if max_y < 100.0 {
        max_y = 100.0;
    }

    let mut out = String::new();
    writeln!(
        out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{:.0}\" height=\"{:.0}\" viewBox=\"0 0 {:.0} {:.0}\">",
        max_x, max_y, max_x, max_y
    )?;
    writeln!(
        out,
        "  <style>text {{ font-family: Helvetica, sans-serif; font-size: 13px; }}</style>"
    )?;

    // Edges
    for edge in &graph.edges {
        if let (Some(src), Some(tgt)) = (graph.get_node(&edge.source), graph.get_node(&edge.target)) {
            let x1 = src.position.x + node_w;
            let y1 = src.position.y + node_h / 2.0;
            let x2 = tgt.position.x;
            let y2 = tgt.position.y + node_h / 2.0;
            writeln!(
                out,
                "  <line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#888\" stroke-width=\"2\" marker-end=\"url(#arrow)\" />",
                x1, y1, x2, y2
            )?;
        }
    }

    // Nodes
    for node in &graph.nodes {
        let fill = match &node.kind {
            NodeKind::Start => "#2ECC71",
            NodeKind::End => "#E74C3C",
            NodeKind::Agent { .. } => "#4A90D9",
            NodeKind::Command { .. } => "#50C878",
            NodeKind::Condition { .. } => "#F5A623",
            NodeKind::Merge => "#9B59B6",
        };
        writeln!(
            out,
            "  <rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.0}\" height=\"{:.0}\" rx=\"8\" fill=\"{}\" />",
            node.position.x, node.position.y, node_w, node_h, fill
        )?;
        writeln!(
            out,
            "  <text x=\"{:.1}\" y=\"{:.1}\" fill=\"white\" text-anchor=\"middle\" dominant-baseline=\"central\">{}</text>",
            node.position.x + node_w / 2.0,
            node.position.y + node_h / 2.0,
            xml_escape(&node.label)
        )?;
    }

    writeln!(out, "</svg>")?;
    Ok(out)
}

/// Minimal XML escape for text content.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        graph::{Edge, Node, NodeKind, Position, WorkflowGraph},
        workflow::Workflow,
    };

    fn sample_graph() -> WorkflowGraph {
        let mut g = WorkflowGraph::new();
        g.add_node(Node {
            id: "s".into(),
            label: "Start".into(),
            kind: NodeKind::Start,
            position: Position { x: 20.0, y: 40.0 },
            config: serde_json::Value::Null,
        });
        g.add_node(Node {
            id: "a".into(),
            label: "Run Agent".into(),
            kind: NodeKind::Agent {
                agent_kind: "claude".into(),
                prompt: None,
            },
            position: Position { x: 240.0, y: 40.0 },
            config: serde_json::Value::Null,
        });
        g.add_node(Node {
            id: "e".into(),
            label: "End".into(),
            kind: NodeKind::End,
            position: Position { x: 460.0, y: 40.0 },
            config: serde_json::Value::Null,
        });
        g.add_edge(Edge {
            id: "e1".into(),
            source: "s".into(),
            target: "a".into(),
            label: None,
            condition: None,
        });
        g.add_edge(Edge {
            id: "e2".into(),
            source: "a".into(),
            target: "e".into(),
            label: Some("done".into()),
            condition: None,
        });
        g
    }

    #[test]
    fn json_roundtrip() {
        let wf = Workflow::new("t1", "Test");
        let json = to_json(&wf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["meta"]["name"], "Test");
    }

    #[test]
    fn dot_contains_nodes() {
        let dot = to_dot(&sample_graph()).unwrap();
        assert!(dot.contains("\"s\""));
        assert!(dot.contains("\"a\""));
        assert!(dot.contains("digraph"));
    }

    #[test]
    fn svg_contains_elements() {
        let svg = to_svg(&sample_graph()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<rect"));
        assert!(svg.contains("<line"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn xml_escape_works() {
        assert_eq!(xml_escape("a<b>&\"c"), "a&lt;b&gt;&amp;&quot;c");
    }
}
