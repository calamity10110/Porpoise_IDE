use std::collections::HashMap;

use porpoise_core::types::id::TerminalId;

use crate::types::{SplitDirection, TerminalPane};

pub struct TerminalLayout {
    panes: HashMap<TerminalId, TerminalPane>,
    root_pane: Option<TerminalId>,
    total_rows: u16,
    total_cols: u16,
}

impl TerminalLayout {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            panes: HashMap::new(),
            root_pane: None,
            total_rows: rows,
            total_cols: cols,
        }
    }

    pub fn add_pane(&mut self, pane: TerminalPane) {
        if self.root_pane.is_none() {
            self.root_pane = Some(pane.id);
        }
        self.panes.insert(pane.id, pane);
    }

    pub fn remove_pane(&mut self, id: TerminalId) {
        self.panes.remove(&id);
        if self.root_pane == Some(id) {
            self.root_pane = self.panes.keys().next().copied();
        }
    }

    pub fn split(&mut self, id: TerminalId, direction: SplitDirection, new_pane: TerminalPane) {
        if let Some(parent) = self.panes.get(&id) {
            let mut child = new_pane;
            match direction {
                SplitDirection::Horizontal => {
                    child.rows = parent.rows / 2;
                }
                SplitDirection::Vertical => {
                    child.cols = parent.cols / 2;
                }
            }
            self.panes.insert(child.id, child);
        }
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.total_rows = rows;
        self.total_cols = cols;
        for pane in self.panes.values_mut() {
            pane.rows = rows;
            pane.cols = cols;
        }
    }

    pub fn get_pane(&self, id: TerminalId) -> Option<&TerminalPane> {
        self.panes.get(&id)
    }

    pub fn all_panes(&self) -> Vec<&TerminalPane> {
        self.panes.values().collect()
    }

    pub fn pane_count(&self) -> usize {
        self.panes.len()
    }
}
