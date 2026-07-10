use std::collections::VecDeque;

use chrono::Utc;

use crate::types::OutputLine;

pub struct ScrollbackBuffer {
    lines: VecDeque<OutputLine>,
    max_lines: usize,
}

impl ScrollbackBuffer {
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(max_lines.min(1000)),
            max_lines,
        }
    }

    pub fn push(&mut self, text: String, is_osc: bool) {
        let line = OutputLine {
            text,
            is_osc,
            timestamp: Utc::now().timestamp(),
        };
        if self.lines.len() >= self.max_lines {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    pub fn lines(&self) -> impl Iterator<Item = &OutputLine> {
        self.lines.iter()
    }

    pub fn search(&self, query: &str, case_sensitive: bool) -> Vec<&OutputLine> {
        self.lines
            .iter()
            .filter(|l| {
                if case_sensitive {
                    l.text.contains(query)
                } else {
                    l.text.to_lowercase().contains(&query.to_lowercase())
                }
            })
            .collect()
    }

    pub fn search_regex(&self, pattern: &regex::Regex) -> Vec<&OutputLine> {
        self.lines.iter().filter(|l| pattern.is_match(&l.text)).collect()
    }

    pub fn search_advanced(&self, query: &str, case_sensitive: bool, use_regex: bool) -> Vec<&OutputLine> {
        if use_regex {
            let pattern = if case_sensitive {
                regex::Regex::new(query)
            } else {
                regex::RegexBuilder::new(query).case_insensitive(true).build()
            };
            match pattern {
                Ok(re) => self.search_regex(&re),
                Err(_) => Vec::new(),
            }
        } else {
            self.search(query, case_sensitive)
        }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}
