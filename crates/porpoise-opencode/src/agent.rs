use std::path::{Path, PathBuf};

use async_trait::async_trait;
use porpoise_agent::{
    traits::{Agent, AgentHandle},
    types::{AgentKind, AgentOutput, OutputType},
};
use porpoise_core::error::{PorpoiseError, Result};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
};

use crate::{
    config::OpenCodeConfig,
    protocol::{OpenCodeEvent, OpenCodeResult, accumulate_events, parse_event_line},
};

pub struct OpenCodeAgent {
    config: OpenCodeConfig,
}

impl OpenCodeAgent {
    pub fn new() -> Self {
        Self {
            config: OpenCodeConfig::default(),
        }
    }

    pub fn with_config(config: OpenCodeConfig) -> Self {
        Self { config }
    }

    fn resolve_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.config.binary_path {
            if path.is_file() {
                return Ok(path.clone());
            }
            return Err(PorpoiseError::Agent(format!(
                "configured binary not found: {}",
                path.display()
            )));
        }
        which_opencode().ok_or_else(|| PorpoiseError::AgentNotDetected {
            kind: "opencode".into(),
        })
    }

    #[cfg(test)]
    fn build_run_args(&self, message: &str) -> Vec<String> {
        let mut args = vec!["run".to_string()];

        args.push("--format".to_string());
        args.push(self.config.output_format.to_string());

        if let Some(model) = &self.config.default_model {
            args.push("-m".to_string());
            args.push(model.clone());
        }

        if let Some(agent) = &self.config.default_agent {
            args.push("--agent".to_string());
            args.push(agent.clone());
        }

        if self.config.pure_mode {
            args.push("--pure".to_string());
        }

        if let Some(level) = &self.config.log_level {
            args.push("--log-level".to_string());
            args.push(level.clone());
        }

        if self.config.show_thinking {
            args.push("--thinking".to_string());
        }

        if self.config.dangerously_skip_permissions {
            args.push("--dangerously-skip-permissions".to_string());
        }

        args.extend(self.config.extra_args.clone());

        if !message.is_empty() {
            for word in message.split_whitespace() {
                args.push(word.to_string());
            }
        }

        args
    }
}

impl Default for OpenCodeAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for OpenCodeAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::OpenCode
    }

    fn binary_name(&self) -> &str {
        "opencode"
    }

    async fn spawn(&self, worktree: &Path) -> Result<Box<dyn AgentHandle>> {
        let binary = self.resolve_binary()?;
        let child = Command::new(&binary)
            .current_dir(worktree)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| PorpoiseError::Agent(format!("spawn opencode: {e}")))?;

        let pid = child.id();
        Ok(Box::new(OpenCodeHandle {
            child: Some(child),
            pid,
            config: self.config.clone(),
            events: vec![],
        }))
    }
}

pub struct OpenCodeHandle {
    child: Option<Child>,
    pid: Option<u32>,
    config: OpenCodeConfig,
    events: Vec<OpenCodeEvent>,
}

impl OpenCodeHandle {
    pub async fn run_message(&mut self, worktree: &Path, message: &str) -> Result<OpenCodeResult> {
        let binary = self
            .config
            .binary_path
            .clone()
            .unwrap_or_else(|| which_opencode().unwrap_or_else(|| PathBuf::from("opencode")));

        let args = self.build_run_args_inner(message);

        let mut child = Command::new(&binary)
            .current_dir(worktree)
            .args(&args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| PorpoiseError::Agent(format!("spawn opencode run: {e}")))?;

        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let mut events = Vec::new();
        let timeout = std::time::Duration::from_secs(self.config.run_timeout_secs);

        let result = tokio::time::timeout(timeout, async {
            while let Some(line) = lines.next_line().await.unwrap_or(None) {
                if let Some(event) = parse_event_line(&line) {
                    events.push(event);
                }
            }
            accumulate_events(&events)
        })
        .await;

        match result {
            Ok(r) => Ok(r),
            Err(_) => {
                let _ = child.kill().await;
                Err(PorpoiseError::Agent(format!(
                    "opencode run timed out after {}s",
                    self.config.run_timeout_secs
                )))
            }
        }
    }

    fn build_run_args_inner(&self, message: &str) -> Vec<String> {
        let mut args = vec!["run".to_string(), "--format".to_string(), "json".to_string()];

        if let Some(model) = &self.config.default_model {
            args.push("-m".to_string());
            args.push(model.clone());
        }

        if self.config.pure_mode {
            args.push("--pure".to_string());
        }

        if self.config.dangerously_skip_permissions {
            args.push("--dangerously-skip-permissions".to_string());
        }

        args.extend(self.config.extra_args.clone());

        for word in message.split_whitespace() {
            args.push(word.to_string());
        }

        args
    }
}

#[async_trait]
impl AgentHandle for OpenCodeHandle {
    fn pid(&self) -> Option<u32> {
        self.pid
    }

    fn is_running(&self) -> bool {
        self.child.is_some()
    }

    async fn read_output(&mut self) -> Result<Option<AgentOutput>> {
        if let Some(ref mut child) = self.child {
            let mut output = String::new();
            if let Some(ref mut stdout) = child.stdout {
                use tokio::io::AsyncReadExt;
                let mut buf = [0u8; 4096];
                match stdout.read(&mut buf).await {
                    Ok(0) => {}
                    Ok(n) => {
                        let chunk = String::from_utf8_lossy(&buf[..n]);
                        output.push_str(&chunk);

                        for line in chunk.lines() {
                            if let Some(event) = parse_event_line(line) {
                                self.events.push(event);
                            }
                        }
                    }
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
            stdin.write_all(b"\n").await.ok();
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
        #[cfg(windows)]
        if let Some(ref mut child) = self.child {
            child.kill().await.ok();
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

fn which_opencode() -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let exe = dir.join("opencode");
            if exe.is_file() {
                Some(exe)
            } else {
                let exe_exe = dir.join("opencode.exe");
                if exe_exe.is_file() { Some(exe_exe) } else { None }
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_run_args() {
        let agent = OpenCodeAgent::with_config(OpenCodeConfig {
            default_model: Some("anthropic/claude-sonnet-4-20250514".into()),
            pure_mode: true,
            ..Default::default()
        });

        let args = agent.build_run_args("fix the bug");
        assert!(args.contains(&"run".to_string()));
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
        assert!(args.contains(&"-m".to_string()));
        assert!(args.contains(&"anthropic/claude-sonnet-4-20250514".to_string()));
        assert!(args.contains(&"--pure".to_string()));
        assert!(args.contains(&"fix".to_string()));
        assert!(args.contains(&"the".to_string()));
        assert!(args.contains(&"bug".to_string()));
    }

    #[test]
    fn test_resolve_binary_default() {
        let agent = OpenCodeAgent::new();
        let result = agent.resolve_binary();
        match result {
            Ok(_) => {}
            Err(PorpoiseError::AgentNotDetected { .. }) => {}
            Err(e) => panic!("unexpected error: {e}"),
        }
    }
}
