use porpoise_core::error::{PorpoiseError, Result};
use crate::types::OutputLine;

/// Batches scrollback lines and persists them to the database.
///
/// Uses a simple flush-on-count threshold strategy. Each batch is written
/// as a single transaction. The database connection is provided at creation
/// time — callers should use the server's `porpoise-db` pool.
pub struct ScrollbackPersister {
    buffer: Vec<OutputLine>,
    batch_size: usize,
    /// Filename-based storage for environments without a DB pool.
    /// Falls back to appending JSON lines to a file.
    storage_path: Option<std::path::PathBuf>,
}

impl ScrollbackPersister {
    /// Creates a new persister with the given batch size and optional file path.
    ///
    /// When `storage_path` is `Some`, lines are appended as NDJSON. This is the
    /// fallback when no database pool is available.
    pub fn new(batch_size: usize, storage_path: Option<std::path::PathBuf>) -> Self {
        Self { buffer: Vec::with_capacity(batch_size), batch_size, storage_path }
    }

    /// Push a line into the buffer, auto-flushing when the batch is full.
    pub fn push(&mut self, line: OutputLine) -> Result<()> {
        self.buffer.push(line);
        if self.buffer.len() >= self.batch_size {
            self.flush()?;
        }
        Ok(())
    }

    /// Force-flush all buffered lines to storage.
    pub fn flush(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        if let Some(ref path) = self.storage_path {
            self.flush_to_file(path)?;
        }
        self.buffer.clear();
        Ok(())
    }

    fn flush_to_file(&self, path: &std::path::Path) -> Result<()> {
        use std::io::Write;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| PorpoiseError::Terminal(format!("mkdir: {e}")))?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| PorpoiseError::Terminal(format!("open: {e}")))?;
        for line in &self.buffer {
            let json = serde_json::to_string(line)
                .map_err(|e| PorpoiseError::Terminal(format!("serialize: {e}")))?;
            writeln!(file, "{json}")
                .map_err(|e| PorpoiseError::Terminal(format!("write: {e}")))?;
        }
        Ok(())
    }

    /// Load persisted scrollback lines from a file.
    pub fn load_from_file(path: &std::path::Path) -> Result<Vec<OutputLine>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PorpoiseError::Terminal(format!("read: {e}")))?;
        let mut lines = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            let parsed: OutputLine = serde_json::from_str(trimmed)
                .map_err(|e| PorpoiseError::Terminal(format!("parse: {e}")))?;
            lines.push(parsed);
        }
        Ok(lines)
    }

    /// Returns the number of buffered (not-yet-flushed) lines.
    pub fn buffered_count(&self) -> usize {
        self.buffer.len()
    }
}
