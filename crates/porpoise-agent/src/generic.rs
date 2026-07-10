use std::path::Path;

use async_trait::async_trait;
use porpoise_core::error::{PorpoiseError, Result};

use crate::{
    traits::{Agent, AgentHandle},
    types::{AgentKind, AgentOutput, OutputType},
};

pub struct GenericAgent {
    binary: String,
    kind: AgentKind,
}

impl GenericAgent {
    pub fn new(binary: impl Into<String>) -> Self {
        let binary = binary.into();
        Self {
            kind: AgentKind::Custom(binary.clone()),
            binary,
        }
    }

    pub fn with_kind(binary: impl Into<String>, kind: AgentKind) -> Self {
        Self {
            binary: binary.into(),
            kind,
        }
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
            .current_dir(worktree)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| PorpoiseError::Agent(format!("spawn {}: {e}", self.binary)))?;
        let pid = child.id();
        Ok(Box::new(GenericHandle {
            child: Some(child),
            pid,
        }))
    }
}

pub struct GenericHandle {
    child: Option<tokio::process::Child>,
    pid: Option<u32>,
}

#[async_trait]
impl AgentHandle for GenericHandle {
    fn pid(&self) -> Option<u32> {
        self.pid
    }
    fn is_running(&self) -> bool {
        self.child.is_some()
    }

    async fn read_output(&mut self) -> Result<Option<AgentOutput>> {
        if let Some(ref mut child) = self.child {
            let mut output = String::new();
            use tokio::io::AsyncReadExt;
            if let Some(ref mut stdout) = child.stdout {
                let mut buf = [0u8; 4096];
                match stdout.read(&mut buf).await {
                    Ok(0) => {}
                    Ok(n) => output.push_str(&String::from_utf8_lossy(&buf[..n])),
                    Err(_) => {}
                }
            }
            if output.is_empty() {
                Ok(None)
            } else {
                Ok(Some(AgentOutput {
                    text: output,
                    output_type: OutputType::Stdout,
                }))
            }
        } else {
            Ok(None)
        }
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
            unsafe {
                libc::kill(pid as i32, libc::SIGINT);
            }
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
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
