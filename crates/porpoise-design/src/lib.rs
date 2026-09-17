//! Porpoise Design Mode
//!
//! Visual workflow designer for agent pipelines. Provides a graph-based
//! data model (nodes + edges) that the frontend renders as a drag-and-drop
//! canvas. The backend validates, serializes, and can execute workflows.

pub mod canvas;
pub mod components;
pub mod export;
pub mod graph;
pub mod props;
pub mod validation;
pub mod workflow;

pub use graph::{Edge, EdgeId, Node, NodeId, NodeKind, Position, WorkflowGraph};
pub use validation::{validate_workflow, ValidationError, ValidationResult};
pub use workflow::{Workflow, WorkflowMeta};
