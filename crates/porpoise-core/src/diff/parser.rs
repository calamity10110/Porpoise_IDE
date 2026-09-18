//! Unified diff parser.

use super::types::*;

/// Parse a unified diff string into an `AnnotatedDiff`.
pub fn parse_unified_diff(raw: &str) -> AnnotatedDiff {
    let mut patches = Vec::new();
    let mut current_patch: Option<FilePatch> = None;
    let mut current_hunk: Option<Hunk> = None;
    let mut old_line = 0u32;
    let mut new_line = 0u32;

    for line in raw.lines() {
        // File header: diff --git a/... b/...
        if line.starts_with("diff --git") {
            if let Some(patch) = current_patch.take() {
                if let Some(mut h) = current_hunk.take() {
                    patch_hunk_close(&mut h);
                    // push hunk into patch via rebind
                    let mut p = patch;
                    p.hunks.push(h);
                    patches.push(p);
                } else {
                    patches.push(patch);
                }
            }
            let paths = parse_diff_header(line);
            current_patch = Some(FilePatch {
                old_path: paths.0,
                new_path: paths.1.unwrap_or_default(),
                hunks: Vec::new(),
                status: FileStatus::Modified,
            });
            current_hunk = None;
            continue;
        }

        // --- / +++ lines
        if line.starts_with("--- ") {
            continue;
        }
        if line.starts_with("+++ ") {
            if let Some(ref mut patch) = current_patch {
                patch.new_path = line.strip_prefix("+++ ").unwrap_or(&line[4..]).trim().to_string();
                if let Some(stripped) = patch.new_path.strip_prefix("b/") {
                    patch.new_path = stripped.to_string();
                }
            }
            continue;
        }

        // Hunk header
        if line.starts_with("@@") {
            if let (Some(ref mut patch), Some(h)) = (current_patch.as_mut(), current_hunk.take()) {
                let mut h = h;
                patch_hunk_close(&mut h);
                patch.hunks.push(h);
            }
            if let Some((os, oc, ns, nc, hdr)) = parse_hunk_header(line) {
                old_line = os;
                new_line = ns;
                current_hunk = Some(Hunk {
                    old_start: os,
                    old_count: oc,
                    new_start: ns,
                    new_count: nc,
                    header: hdr,
                    lines: Vec::new(),
                });
            }
            continue;
        }

        // Diff lines
        if let Some(ref mut hunk) = current_hunk {
            if let Some(first) = line.chars().next() {
                match first {
                    '+' => {
                        hunk.lines.push(DiffLine {
                            kind: LineKind::Add,
                            content: line[1..].to_string(),
                            old_line_no: None,
                            new_line_no: Some(new_line),
                            annotation: None,
                        });
                        new_line += 1;
                    }
                    '-' => {
                        hunk.lines.push(DiffLine {
                            kind: LineKind::Delete,
                            content: line[1..].to_string(),
                            old_line_no: Some(old_line),
                            new_line_no: None,
                            annotation: None,
                        });
                        old_line += 1;
                    }
                    ' ' => {
                        hunk.lines.push(DiffLine {
                            kind: LineKind::Context,
                            content: line[1..].to_string(),
                            old_line_no: Some(old_line),
                            new_line_no: Some(new_line),
                            annotation: None,
                        });
                        old_line += 1;
                        new_line += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    // Flush last patch
    if let Some(mut patch) = current_patch {
        if let Some(mut h) = current_hunk.take() {
            patch_hunk_close(&mut h);
            patch.hunks.push(h);
        }
        patch.status = infer_status(&patch);
        patches.push(patch);
    }

    AnnotatedDiff {
        patches,
        agent_id: None,
        timestamp: None,
    }
}

fn patch_hunk_close(_hunk: &mut Hunk) {
    // placeholder for future post-processing
}

fn parse_diff_header(line: &str) -> (Option<String>, Option<String>) {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let old = parts.get(2).and_then(|p| {
        let s = p.trim();
        if let Some(stripped) = s.strip_prefix("a/") {
            Some(stripped.to_string())
        } else {
            Some(s.to_string())
        }
    });
    let new = parts.get(3).and_then(|p| {
        let s = p.trim();
        if let Some(stripped) = s.strip_prefix("b/") {
            Some(stripped.to_string())
        } else {
            Some(s.to_string())
        }
    });
    (old, new)
}

fn parse_hunk_header(line: &str) -> Option<(u32, u32, u32, u32, Option<String>)> {
    // @@ -old_start,old_count +new_start,new_count @@ optional header
    let rest = line.strip_prefix("@@")?.trim();
    let parts: Vec<&str> = rest.split("@@").collect();
    let range_str = parts.first()?.trim();
    let header = parts.get(1).map(|s| s.trim().to_string());

    let mut ranges = range_str.split_whitespace();
    let old_range = ranges.next()?;
    let new_range = ranges.next()?;

    let (os, oc) = parse_range(old_range)?;
    let (ns, nc) = parse_range(new_range)?;

    Some((os, oc, ns, nc, header))
}

fn parse_range(s: &str) -> Option<(u32, u32)> {
    let s = s.strip_prefix('+').or_else(|| s.strip_prefix('-'))?;
    let parts: Vec<&str> = s.split(',').collect();
    let start = parts.first()?.parse().ok()?;
    let count = parts.get(1).and_then(|c| c.parse().ok()).unwrap_or(1);
    Some((start, count))
}

fn infer_status(patch: &FilePatch) -> FileStatus {
    let has_adds = patch
        .hunks
        .iter()
        .any(|h| h.lines.iter().any(|l| l.kind == LineKind::Add));
    let has_dels = patch
        .hunks
        .iter()
        .any(|h| h.lines.iter().any(|l| l.kind == LineKind::Delete));
    let old_is_devnull = patch.old_path.as_deref() == Some("/dev/null");

    if old_is_devnull {
        FileStatus::Added
    } else if !has_adds && has_dels {
        FileStatus::Deleted
    } else {
        FileStatus::Modified
    }
}
