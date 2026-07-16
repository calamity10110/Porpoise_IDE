pub mod handle;
pub mod pool;

use std::{collections::HashMap, path::Path, sync::Arc};

use chrono::Utc;
pub use handle::ProcessHandle;
use porpoise_core::{
    bus::EventBus,
    error::{PorpoiseError, Result},
    types::id::ProcessId,
};
use tokio::{
    process::Command,
    sync::{RwLock, mpsc, watch},
};

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

struct ProcessEntry {
    handle: ProcessHandle,
    child: Option<tokio::process::Child>,
}

pub struct ProcessManager {
    processes: Arc<RwLock<HashMap<ProcessId, ProcessEntry>>>,
    #[allow(dead_code)]
    event_bus: EventBus,
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
        command
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if let Some(dir) = cwd {
            command.current_dir(dir);
        }
        for (k, v) in envs {
            command.env(k, v);
        }

        let child = command
            .spawn()
            .map_err(|e| PorpoiseError::Runtime(format!("spawn failed: {e}")))?;
        let pid = child
            .id()
            .ok_or_else(|| PorpoiseError::Runtime("no pid from spawned process".into()))?;

        let (_status_tx, status_rx) = watch::channel(ProcessStatus::Running);
        let (cmd_tx, _cmd_rx) = mpsc::channel::<ProcessCommand>(32);

        let handle = ProcessHandle {
            id,
            pid,
            kind,
            created_at: Utc::now(),
            status_rx,
            cmd_tx,
        };

        self.processes.write().await.insert(
            id,
            ProcessEntry {
                handle: handle.clone(),
                child: Some(child),
            },
        );

        Ok(handle)
    }

    pub async fn kill(&self, id: ProcessId) -> Result<()> {
        let mut procs = self.processes.write().await;
        let entry = procs
            .remove(&id)
            .ok_or_else(|| PorpoiseError::Runtime(format!("process {id} not found")))?;
        if let Some(mut child) = entry.child {
            child.start_kill().map_err(|e| PorpoiseError::Runtime(format!("kill: {e}")))?;
            child.wait().await.map_err(|e| PorpoiseError::Runtime(format!("wait: {e}")))?;
        }
        Ok(())
    }

    pub async fn shutdown_all(&self, _timeout: std::time::Duration) -> Result<()> {
        let ids: Vec<ProcessId> = self.processes.read().await.keys().copied().collect();
        for id in &ids {
            self.kill(*id).await.ok();
        }
        Ok(())
    }

    pub async fn list(&self) -> Vec<ProcessHandle> {
        self.processes.read().await.values().map(|e| e.handle.clone()).collect()
    }
}
