//! Annotates diff lines with agent attribution.

use super::types::*;

/// Annotate all lines in a diff with agent info.
pub fn annotate(diff: &mut AnnotatedDiff, agent_id: &str, agent_kind: &str) {
    let annotation = LineAnnotation {
        agent_id: agent_id.to_string(),
        agent_kind: agent_kind.to_string(),
        confidence: 1.0,
        reason: None,
    };
    for patch in &mut diff.patches {
        for hunk in &mut patch.hunks {
            for line in &mut hunk.lines {
                if line.kind != LineKind::Context {
                    line.annotation = Some(annotation.clone());
                }
            }
        }
    }
    diff.agent_id = Some(agent_id.to_string());
}

/// Compute a summary of changes in the diff.
pub fn summarize(diff: &AnnotatedDiff) -> DiffSummary {
    let mut files_changed = 0u32;
    let mut additions = 0u32;
    let mut deletions = 0u32;

    for patch in &diff.patches {
        files_changed += 1;
        for hunk in &patch.hunks {
            for line in &hunk.lines {
                match line.kind {
                    LineKind::Add => additions += 1,
                    LineKind::Delete => deletions += 1,
                    LineKind::Context => {}
                }
            }
        }
    }

    DiffSummary {
        files_changed,
        additions,
        deletions,
    }
}

/// Summary statistics for a diff.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiffSummary {
    pub files_changed: u32,
    pub additions: u32,
    pub deletions: u32,
}
