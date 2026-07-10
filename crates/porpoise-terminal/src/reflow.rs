use crate::types::OutputLine;

/// Represents a wrapped terminal line with its logical position.
#[derive(Debug, Clone)]
pub struct ReflowedLine {
    pub text: String,
    pub wrapped_from: Option<usize>,
}

/// Reflows scrollback lines to fit a new terminal width.
///
/// When a terminal is resized, lines that were wrapped at the old width
/// need to be re-wrapped at the new width. This module handles:
/// - Joining lines that were hard-wrapped at the old column width
/// - Re-wrapping joined content at the new column width
/// - Preserving explicit newlines in the original content
pub fn reflow_lines(lines: &[OutputLine], old_cols: u16, new_cols: u16) -> Vec<OutputLine> {
    if lines.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(lines.len());
    let mut buffer = String::new();

    for line in lines {
        if line.text.len() >= old_cols as usize && !line.text.contains('\n') {
            buffer.push_str(&line.text);
        } else {
            if !buffer.is_empty() {
                let wrapped = wrap_text(&buffer, new_cols);
                for text in wrapped {
                    result.push(OutputLine {
                        text,
                        is_osc: line.is_osc,
                        timestamp: line.timestamp,
                    });
                }
                buffer.clear();
            }
            let wrapped = wrap_text(&line.text, new_cols);
            for text in wrapped {
                result.push(OutputLine {
                    text,
                    is_osc: line.is_osc,
                    timestamp: line.timestamp,
                });
            }
        }
    }

    if !buffer.is_empty() {
        let wrapped = wrap_text(&buffer, new_cols);
        for text in wrapped {
            result.push(OutputLine {
                text,
                is_osc: false,
                timestamp: chrono::Utc::now().timestamp(),
            });
        }
    }

    result
}

fn wrap_text(text: &str, max_width: u16) -> Vec<String> {
    let width = max_width as usize;
    if width == 0 {
        return vec![text.to_string()];
    }

    let mut result = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            result.push(String::new());
            continue;
        }

        let chars: Vec<char> = paragraph.chars().collect();
        let mut start = 0;
        while start < chars.len() {
            let end = (start + width).min(chars.len());
            let chunk: String = chars[start..end].iter().collect();
            result.push(chunk);
            start = end;
        }
    }

    if result.is_empty() {
        result.push(text.to_string());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(text: &str) -> OutputLine {
        OutputLine {
            text: text.into(),
            is_osc: false,
            timestamp: 1000,
        }
    }

    #[test]
    fn test_no_change_same_width() {
        let lines = vec![line("hello"), line("world")];
        let result = reflow_lines(&lines, 80, 80);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].text, "hello");
        assert_eq!(result[1].text, "world");
    }

    #[test]
    fn test_wrap_narrow() {
        let lines = vec![line("hello world this is a long line")];
        let result = reflow_lines(&lines, 80, 10);
        assert!(result.len() > 1);
        for l in &result {
            assert!(l.text.len() <= 10);
        }
    }

    #[test]
    fn test_join_and_rewrap() {
        let lines = vec![line("abcdefghijklmnopqrst"), line("uvwxyz0123456789")];
        let result = reflow_lines(&lines, 20, 10);
        assert!(result.len() > 2);
    }
}
