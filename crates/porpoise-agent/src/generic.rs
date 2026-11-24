use std::path::Path;

use async_trait::async_trait;
use porpoise_core::error::{PorpoiseError, Result};
use tokio::io::AsyncReadExt;

use crate::{
    traits::{Agent, AgentHandle},
    types::{AgentKind, AgentOutput, OutputType},
};

pub struct GenericAgent {
    binary: String,
    kind: AgentKind,
    args: Vec<String>,
}

impl GenericAgent {
    pub fn new(binary: impl Into<String>) -> Self {
        let binary = binary.into();
        Self {
            kind: AgentKind::Custom(binary.clone()),
            binary,
            args: Vec::new(),
        }
    }

    pub fn with_kind(binary: impl Into<String>, kind: AgentKind) -> Self {
        Self {
            binary: binary.into(),
            kind,
            args: Vec::new(),
        }
    }

    /// Extra CLI arguments for the spawned binary. Built by an adapter's
    /// `build_args` so the prompt and worktree reach the agent in the
    /// format that binary expects.
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }
}

#[async_trait]
impl Agent for GenericAgent {
    fn kind(&self) -> AgentKind {
        self.kind.clone()
    }
    fn binary_name(&self) -> &str {
        &self.binary
    }

    async fn spawn(&self, worktree: &Path) -> Result<Box<dyn AgentHandle>> {
        let child = tokio::process::Command::new(&self.binary)
            .args(&self.args)
            .current_dir(worktree)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| PorpoiseError::Agent(format!("spawn {}: {e}", self.binary)))?;
        let mut child = child;
        let pid = child.id();
        // Take the streams out of the child so `read_output` can borrow
        // them directly while the child is waited on separately.
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        Ok(Box::new(GenericHandle {
            child: Some(child),
            pid,
            stdout,
            stderr,
        }))
    }
}

pub struct GenericHandle {
    child: Option<tokio::process::Child>,
    pid: Option<u32>,
    stdout: Option<tokio::process::ChildStdout>,
    stderr: Option<tokio::process::ChildStderr>,
}

/// Result of one non-blocking-ish read attempt on a child stream.
enum ReadOutcome {
    Data(Vec<u8>),
    Eof,
}

/// Read one chunk from an optional stream. A missing stream (already at
/// EOF and cleared) pends forever so `select!` only waits on streams that
/// still exist.
async fn read_chunk<R: tokio::io::AsyncRead + Unpin>(stream: &mut Option<R>) -> ReadOutcome {
    let Some(active) = stream.as_mut() else {
        return std::future::pending().await;
    };
    let mut buf = [0u8; 8192];
    match active.read(&mut buf).await {
        Ok(0) | Err(_) => ReadOutcome::Eof,
        Ok(n) => ReadOutcome::Data(buf[..n].to_vec()),
    }
}

#[async_trait]
impl AgentHandle for GenericHandle {
    fn pid(&self) -> Option<u32> {
        self.pid
    }
    fn is_running(&self) -> bool {
        self.child.is_some()
    }

    /// Read the next chunk of agent output.
    ///
    /// Returns stdout/stderr data as it arrives; once both streams hit
    /// EOF, waits for the process to exit and returns a single
    /// [`OutputType::Exit`] as the run's last output. After that every
    /// call returns `None`, which is the loop's termination signal.
    async fn read_output(&mut self) -> Result<Option<AgentOutput>> {
        while self.stdout.is_some() || self.stderr.is_some() {
            tokio::select! {
                biased;
                outcome = read_chunk(&mut self.stdout) => match outcome {
                    ReadOutcome::Data(data) => {
                        return Ok(Some(AgentOutput {
                            text: String::from_utf8_lossy(&data).into_owned(),
                            output_type: OutputType::Stdout,
                        }));
                    }
                    ReadOutcome::Eof => self.stdout = None,
                },
                outcome = read_chunk(&mut self.stderr) => match outcome {
                    ReadOutcome::Data(data) => {
                        return Ok(Some(AgentOutput {
                            text: String::from_utf8_lossy(&data).into_owned(),
                            output_type: OutputType::Stderr,
                        }));
                    }
                    ReadOutcome::Eof => self.stderr = None,
                },
            }
        }

        // Both streams are closed: the process is on its way out (or
        // already gone). Wait for it exactly once and report the exit.
        if let Some(mut child) = self.child.take() {
            let code = child
                .wait()
                .await
                .map(|status| status.code().unwrap_or(-1))
                .unwrap_or(-1);
            return Ok(Some(AgentOutput {
                text: String::new(),
                output_type: OutputType::Exit(code),
            }));
        }
        Ok(None)
    }

    async fn send_input(&mut self, text: &str) -> Result<()> {
        if let Some(ref mut child) = self.child
            && let Some(ref mut stdin) = child.stdin
        {
            use tokio::io::AsyncWriteExt;
            stdin
                .write_all(text.as_bytes())
                .await
                .map_err(|e| PorpoiseError::Agent(format!("write stdin: {e}")))?;
        }
        Ok(())
    }

    async fn interrupt(&mut self) -> Result<()> {
        #[cfg(unix)]
        if let Some(pid) = self.pid {
            // SAFETY: kill() sends SIGINT to the child process for interruption.
            // `pid` comes from child.id() captured at spawn time. Sending a signal
            // to a non-existent PID is a harmless no-op (ESRCH).
            unsafe {
                libc::kill(pid as i32, libc::SIGINT);
            }
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        // Dropping the streams unblocks any concurrent read_output.
        self.stdout = None;
        self.stderr = None;
        if let Some(mut child) = self.child.take() {
            child
                .kill()
                .await
                .map_err(|e| PorpoiseError::Agent(format!("kill: {e}")))?;
            child.wait().await.ok();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    /// An AG-UI run needs the exit exactly once, after all output: spawn a
    /// real process and read it to completion.
    #[tokio::test]
    async fn read_output_drains_streams_then_reports_exit_once() {
        let agent = GenericAgent::with_kind(if cfg!(windows) { "cmd" } else { "sh" }, AgentKind::Custom("sh".into()))
            .with_args(if cfg!(windows) {
                vec!["/C".into(), "echo out & echo err 1>&2".into()]
            } else {
                vec!["-c".into(), "echo out; echo err 1>&2".into()]
            });
        let workdir = std::env::temp_dir();
        let mut handle = agent.spawn(&workdir).await.expect("spawn");

        let mut saw_stdout = false;
        let mut saw_stderr = false;
        let mut exits = 0;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        loop {
            let output = tokio::time::timeout(Duration::from_secs(5), handle.read_output())
                .await
                .expect("read timeout")
                .expect("read");
            match output {
                Some(out) => match out.output_type {
                    OutputType::Stdout => saw_stdout = true,
                    OutputType::Stderr => saw_stderr = true,
                    OutputType::Exit(code) => {
                        exits += 1;
                        assert_eq!(code, 0, "echo exits cleanly");
                        break;
                    }
                    _ => {}
                },
                None => panic!("stream ended before an exit was reported"),
            }
            assert!(tokio::time::Instant::now() < deadline, "run did not terminate");
        }
        assert!(saw_stdout, "stdout was drained");
        assert!(saw_stderr, "stderr was drained, not deadlocked in the pipe");
        assert!(!handle.is_running(), "handle reports not running after exit");
        // After the single Exit, the stream is done.
        assert!(handle.read_output().await.expect("read").is_none());
        assert_eq!(exits, 1);
    }
}
