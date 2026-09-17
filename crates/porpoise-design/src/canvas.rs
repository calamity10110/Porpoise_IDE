//! Canvas rendering model — viewport, zoom, pan, and node layout.
//!
//! Provides the data structures needed by the frontend to render a workflow
//! graph on an interactive canvas. This crate owns the *model*; the actual
//! drawing happens in the frontend (HTML Canvas / SVG).

use serde::{Deserialize, Serialize};

use crate::graph::{NodeId, Position, WorkflowGraph};

/// Zoom level (1.0 = 100%).
pub type Zoom = f64;

/// Viewport rectangle in canvas coordinates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Viewport {
    /// Top-left corner x.
    pub x: f64,
    /// Top-left corner y.
    pub y: f64,
    /// Visible width.
    pub width: f64,
    /// Visible height.
    pub height: f64,
    /// Current zoom level.
    pub zoom: Zoom,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1200.0,
            height: 800.0,
            zoom: 1.0,
        }
    }
}

impl Viewport {
    /// Pan the viewport by a delta.
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }

    /// Set zoom level, clamped to sane range.
    pub fn set_zoom(&mut self, zoom: Zoom) {
        self.zoom = zoom.clamp(0.1, 5.0);
    }

    /// Check whether a canvas position is visible in this viewport.
    pub fn contains(&self, pos: &Position) -> bool {
        let sx = (pos.x - self.x) * self.zoom;
        let sy = (pos.y - self.y) * self.zoom;
        sx >= 0.0 && sx <= self.width && sy >= 0.0 && sy <= self.height
    }
}

/// Default node dimensions used for layout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeSize {
    pub width: f64,
    pub height: f64,
}

impl Default for NodeSize {
    fn default() -> Self {
        Self {
            width: 180.0,
            height: 80.0,
        }
    }
}

/// A positioned node ready for rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedNode {
    pub node_id: NodeId,
    pub label: String,
    pub kind_label: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub selected: bool,
}

/// A rendered edge with computed path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedEdge {
    pub edge_id: String,
    pub source_id: NodeId,
    pub target_id: NodeId,
    /// Start point (source node right-center).
    pub start: Position,
    /// End point (target node left-center).
    pub end: Position,
    pub label: Option<String>,
}

/// Complete render state for a workflow graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasState {
    pub viewport: Viewport,
    pub nodes: Vec<RenderedNode>,
    pub edges: Vec<RenderedEdge>,
    pub node_size: NodeSize,
}

impl CanvasState {
    /// Build a canvas state from a workflow graph, using each node's stored
    /// position and the default node size.
    pub fn from_graph(graph: &WorkflowGraph, viewport: Viewport) -> Self {
        let node_size = NodeSize::default();
        let nodes: Vec<RenderedNode> = graph
            .nodes
            .iter()
            .map(|n| RenderedNode {
                node_id: n.id.clone(),
                label: n.label.clone(),
                kind_label: format!("{:?}", n.kind),
                x: n.position.x,
                y: n.position.y,
                width: node_size.width,
                height: node_size.height,
                selected: false,
            })
            .collect();

        let edges: Vec<RenderedEdge> = graph
            .edges
            .iter()
            .filter_map(|e| {
                let src = graph.get_node(&e.source)?;
                let tgt = graph.get_node(&e.target)?;
                Some(RenderedEdge {
                    edge_id: e.id.clone(),
                    source_id: e.source.clone(),
                    target_id: e.target.clone(),
                    start: Position {
                        x: src.position.x + node_size.width,
                        y: src.position.y + node_size.height / 2.0,
                    },
                    end: Position {
                        x: tgt.position.x,
                        y: tgt.position.y + node_size.height / 2.0,
                    },
                    label: e.label.clone(),
                })
            })
            .collect();

        Self {
            viewport,
            nodes,
            edges,
            node_size,
        }
    }

    /// Auto-layout nodes in a grid pattern (simple fallback).
    pub fn auto_layout(&mut self, cols: usize, spacing_x: f64, spacing_y: f64) {
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let col = i % cols;
            let row = i / cols;
            node.x = col as f64 * (self.node_size.width + spacing_x) + 40.0;
            node.y = row as f64 * (self.node_size.height + spacing_y) + 40.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Node, NodeKind, WorkflowGraph};

    #[test]
    fn viewport_contains() {
        let vp = Viewport::default();
        assert!(vp.contains(&Position { x: 100.0, y: 100.0 }));
        assert!(!vp.contains(&Position { x: -10.0, y: 0.0 }));
    }

    #[test]
    fn viewport_zoom_clamp() {
        let mut vp = Viewport::default();
        vp.set_zoom(10.0);
        assert_eq!(vp.zoom, 5.0);
        vp.set_zoom(0.01);
        assert_eq!(vp.zoom, 0.1);
    }

    #[test]
    fn canvas_from_graph() {
        let mut graph = WorkflowGraph::new();
        graph.add_node(Node {
            id: "a".into(),
            label: "Start".into(),
            kind: NodeKind::Start,
            position: Position { x: 0.0, y: 0.0 },
            config: serde_json::Value::Null,
        });
        graph.add_node(Node {
            id: "b".into(),
            label: "End".into(),
            kind: NodeKind::End,
            position: Position { x: 200.0, y: 0.0 },
            config: serde_json::Value::Null,
        });
        graph.add_edge(crate::graph::Edge {
            id: "e1".into(),
            source: "a".into(),
            target: "b".into(),
            label: None,
            condition: None,
        });

        let canvas = CanvasState::from_graph(&graph, Viewport::default());
        assert_eq!(canvas.nodes.len(), 2);
        assert_eq!(canvas.edges.len(), 1);
        assert_eq!(canvas.edges[0].start.x, 180.0); // node width
    }

    #[test]
    fn auto_layout() {
        let mut graph = WorkflowGraph::new();
        for i in 0..5 {
            graph.add_node(Node {
                id: format!("n{i}"),
                label: format!("Node {i}"),
                kind: NodeKind::Command {
                    command: "echo".into(),
                },
                position: Position { x: 0.0, y: 0.0 },
                config: serde_json::Value::Null,
            });
        }
        let mut canvas = CanvasState::from_graph(&graph, Viewport::default());
        canvas.auto_layout(3, 20.0, 20.0);
        // First row: 3 nodes, second row: 2 nodes
        assert_eq!(canvas.nodes[0].x, 40.0);
        assert_eq!(canvas.nodes[3].x, 40.0); // second row, col 0
        assert_eq!(canvas.nodes[3].y, 40.0 + 80.0 + 20.0); // row 1
    }
}
