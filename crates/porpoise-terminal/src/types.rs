use serde::{Deserialize, Serialize};
use porpoise_core::types::id::{TerminalId, SessionId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub rows: u16,
    pub cols: u16,
    pub shell: String,
    pub scrollback_lines: usize,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self { rows: 24, cols: 80, shell: "bash".into(), scrollback_lines: 10_000 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalPane {
    pub id: TerminalId,
    pub session_id: SessionId,
    pub rows: u16,
    pub cols: u16,
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputLine {
    pub text: String,
    pub is_osc: bool,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Default)]
pub struct ColorScheme {
    pub foreground: String,
    pub background: String,
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
    pub bright_black: String,
    pub bright_red: String,
    pub bright_green: String,
    pub bright_yellow: String,
    pub bright_blue: String,
    pub bright_magenta: String,
    pub bright_cyan: String,
    pub bright_white: String,
}
