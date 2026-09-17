//! Diff Annotation System
//!
//! Parses unified diffs and annotates lines with agent attribution,
//! enabling per-line provenance tracking across multi-agent workflows.

pub mod annotator;
pub mod parser;
pub mod types;

pub use annotator::{DiffSummary, annotate, summarize};
pub use parser::parse_unified_diff;
pub use types::*;
