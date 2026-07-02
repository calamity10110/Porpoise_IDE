pub mod handle;
pub mod pool;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use chrono::Utc;
use tokio::process::Command;
use tokio::sync::{mpsc, watch, RwLock};
use tokio::task::JoinHandle;

use porpoise_core::error::{PorpoiseError, Result};
use porpoise_core::types::id::ProcessId;
use porpoise_core::bus::EventBus;

pub use handle::ProcessHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessKind {
    Agent,
    Shell,
    Git,
    Build,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessStatus {
    Running,
    Exited(i32),
    Signalled(String),
    Killed,
    Error(String),
}

#[derive(Debug)]
pub enum ProcessCommand {
    Kill,
    Signal(String),
}

pub struct ProcessManager {
    processes: Arc<RwLock<HashMap<ProcessId, ProcessEntry>>>,
    #[allow(dead_code)]
    event_bus: EventBus,
}

struct ProcessEntry {
    handle: ProcessHandle,
    _task: JoinHandle<()>,
}

impl ProcessManager {
    pub fn new(event_bus: EventBus) -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
        }
    }

    pub async fn spawn(
        &self,
        kind: ProcessKind,
        cmd: &str,
        args: &[&str],
        cwd: Option<&Path>,
        envs: Vec<(&str, &str)>,
    ) -> Result<ProcessHandle> {
        let id = ProcessId::new();

        let mut command = Command::new(cmd);
        command.args(args).stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if let Some(dir) = cwd {
            command.current_dir(dir);
        }
        for (k, v) in envs {
            command.env(k, v);
        }

        let child = command.spawn()
            .map_err(|e| PorpoiseError::Runtime(format!("spawn failed: {e}")))?;
        let pid = child.id().ok_or_else(|| {
            PorpoiseError::Runtime("no pid from spawned process".into())
        })?;

        let (_status_tx, status_rx) = watch::channel(ProcessStatus::Running);
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<ProcessCommand>(32);

        let handle = ProcessHandle {
            id,
            pid,
            kind,
            created_at: Utc::now(),
            status_rx,
            cmd_tx,
        };

        let _p = pid;
        let monitor_task = tokio::spawn(async move {
            let _ = cmd_rx.recv().await;
        });

        self.processes.write().await.insert(id, ProcessEntry {
            handle: handle.clone(),
            _task: monitor_task,
        });

        Ok(handle)
    }

    pub async fn kill(&self, id: ProcessId) -> Result<()> {
        let mut procs = self.processes.write().await;
        let entry = procs.remove(&id)
            .ok_or_else(|| PorpoiseError::Runtime(format!("process {id} not found")))?;
        entry.handle.cmd_tx.send(ProcessCommand::Kill).await.ok();
        Ok(())
    }

    pub async fn shutdown_all(&self, _timeout: std::time::Duration) -> Result<()> {
        let ids: Vec<ProcessId> = self.processes.read().await.keys().copied().collect();
        for id in &ids {
            self.kill(*id).await.ok();
        }
        tokio::time::sleep(_timeout).await;
        self.processes.write().await.clear();
        Ok(())
    }

    pub async fn list(&self) -> Vec<ProcessHandle> {
        self.processes.read().await.values()
            .map(|e| e.handle.clone())
            .collect()
    }
}
