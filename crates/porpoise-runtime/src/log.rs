use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

use porpoise_core::error::{PorpoiseError, Result};

/// Configuration for log rotation.
#[derive(Debug, Clone)]
pub struct LogRotationConfig {
    /// Maximum file size in bytes before rotation triggers.
    pub max_size_bytes: u64,
    /// Number of rotated backup files to keep.
    pub max_files: usize,
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 10 * 1024 * 1024, // 10 MB
            max_files: 5,
        }
    }
}

/// A rotating log file writer.
///
/// Writes log lines to a file. When the file exceeds `max_size_bytes`,
/// it is renamed to `<name>.1`, older backups are shifted, and a new
/// file is opened. At most `max_files` backup copies are retained.
///
/// This is designed for the server daemon's log output. It is
/// thread-safe via an internal `Mutex`.
pub struct RotatingLogFile {
    path: PathBuf,
    config: LogRotationConfig,
    inner: Mutex<Inner>,
}

struct Inner {
    file: Option<fs::File>,
    current_size: u64,
}

impl RotatingLogFile {
    /// Creates a new rotating log at the given path.
    pub fn new(path: impl Into<PathBuf>, config: LogRotationConfig) -> Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| PorpoiseError::Runtime(format!("create log dir: {e}")))?;
        }
        let current_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| PorpoiseError::Runtime(format!("open log file: {e}")))?;

        Ok(Self {
            path,
            config,
            inner: Mutex::new(Inner {
                file: Some(file),
                current_size,
            }),
        })
    }

    /// Writes a line to the log file, rotating if necessary.
    pub fn writeln(&self, line: &str) -> Result<()> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| PorpoiseError::Runtime(format!("log lock: {e}")))?;

        let data = format!("{line}\n");
        let data_bytes = data.len() as u64;

        if guard.current_size + data_bytes > self.config.max_size_bytes {
            self.rotate_locked(&mut guard)?;
        }

        if let Some(ref mut file) = guard.file {
            file.write_all(data.as_bytes())
                .map_err(|e| PorpoiseError::Runtime(format!("log write: {e}")))?;
            file.flush()
                .map_err(|e| PorpoiseError::Runtime(format!("log flush: {e}")))?;
            guard.current_size += data_bytes;
        }

        Ok(())
    }

    /// Forces a rotation regardless of current file size.
    pub fn rotate(&self) -> Result<()> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| PorpoiseError::Runtime(format!("log lock: {e}")))?;
        self.rotate_locked(&mut guard)
    }

    fn rotate_locked(&self, guard: &mut Inner) -> Result<()> {
        guard.file = None;
        guard.current_size = 0;

        let oldest = self.nth_path(self.config.max_files);
        if oldest.exists() {
            fs::remove_file(&oldest).ok();
        }

        for i in (1..self.config.max_files).rev() {
            let from = self.nth_path(i);
            let to = self.nth_path(i + 1);
            if from.exists() {
                fs::rename(&from, &to).ok();
            }
        }

        if self.path.exists() {
            let backup = self.nth_path(1);
            fs::rename(&self.path, &backup).map_err(|e| PorpoiseError::Runtime(format!("rotate rename: {e}")))?;
        }

        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| PorpoiseError::Runtime(format!("open log after rotate: {e}")))?;
        guard.file = Some(file);

        Ok(())
    }

    fn nth_path(&self, n: usize) -> PathBuf {
        self.path.with_extension(format!("log.{n}"))
    }

    /// Returns the current log file path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the current file size in bytes.
    pub fn current_size(&self) -> u64 {
        self.inner.lock().map(|g| g.current_size).unwrap_or(0)
    }

    /// Returns the rotation configuration.
    pub fn config(&self) -> &LogRotationConfig {
        &self.config
    }
}

pub fn create_rotating_log(log_dir: &Path, config: LogRotationConfig) -> Result<RotatingLogFile> {
    fs::create_dir_all(log_dir).map_err(|e| PorpoiseError::Runtime(format!("create log dir: {e}")))?;
    let log_path = log_dir.join("porpoise.log");
    RotatingLogFile::new(&log_path, config)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_write_and_rotate() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.log");
        let config = LogRotationConfig {
            max_size_bytes: 100,
            max_files: 3,
        };
        let logger = RotatingLogFile::new(&path, config).unwrap();

        // Write enough data to trigger rotation
        for i in 0..20 {
            logger
                .writeln(&format!("line {i}: some padding text to fill up space"))
                .unwrap();
        }

        // Current file should exist
        assert!(path.exists());
        // At least one backup should exist
        assert!(path.with_extension("log.1").exists());
    }

    #[test]
    fn test_no_rotation_under_limit() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("small.log");
        let config = LogRotationConfig {
            max_size_bytes: 10_000,
            max_files: 3,
        };
        let logger = RotatingLogFile::new(&path, config).unwrap();
        logger.writeln("hello").unwrap();
        logger.writeln("world").unwrap();

        assert!(!path.with_extension("log.1").exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("hello"));
        assert!(content.contains("world"));
    }

    #[test]
    fn test_max_files_enforced() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("max.log");
        let config = LogRotationConfig {
            max_size_bytes: 50,
            max_files: 2,
        };
        let logger = RotatingLogFile::new(&path, config).unwrap();

        for i in 0..50 {
            logger.writeln(&format!("line {i}")).unwrap();
        }

        // .1 and .2 should exist, .3 should not
        assert!(path.with_extension("log.1").exists());
        assert!(path.with_extension("log.2").exists());
        assert!(!path.with_extension("log.3").exists());
    }
}
