//! Diff annotation types.

use serde::{Deserialize, Serialize};

/// A parsed unified diff containing one or more file patches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedDiff {
    pub patches: Vec<FilePatch>,
    pub agent_id: Option<String>,
    pub timestamp: Option<String>,
}

/// A single file's patch within a diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePatch {
    pub old_path: Option<String>,
    pub new_path: String,
    pub hunks: Vec<Hunk>,
    pub status: FileStatus,
}

/// Status of a file in the diff.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed { from: String },
    Copied { from: String },
}

/// A hunk within a file patch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hunk {
    pub old_start: u32,
    pub old_count: u32,
    pub new_start: u32,
    pub new_count: u32,
    pub header: Option<String>,
    pub lines: Vec<DiffLine>,
}

/// A single line in a hunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub kind: LineKind,
    pub content: String,
    pub old_line_no: Option<u32>,
    pub new_line_no: Option<u32>,
    pub annotation: Option<LineAnnotation>,
}

/// Whether a line is added, removed, or context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LineKind {
    Add,
    Delete,
    Context,
}

/// Agent annotation attached to a diff line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineAnnotation {
    pub agent_id: String,
    pub agent_kind: String,
    pub confidence: f32,
    pub reason: Option<String>,
}
