use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::Utc;
use porpoise_core::{
    error::{PorpoiseError, Result},
    types::id::{ProcessId, WorktreeId},
};
use tokio::{process::Command, sync::RwLock};

#[derive(Debug, Clone)]
pub struct WorktreeProcess {
    pub process_id: ProcessId,
    pub worktree_id: WorktreeId,
    pub pid: u32,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub started_at: chrono::DateTime<Utc>,
}

struct ManagedProcess {
    info: WorktreeProcess,
    child: Option<tokio::process::Child>,
}

pub struct WorktreeProcessManager {
    processes: Arc<RwLock<HashMap<ProcessId, ManagedProcess>>>,
    by_worktree: Arc<RwLock<HashMap<WorktreeId, Vec<ProcessId>>>>,
    max_per_worktree: usize,
}

impl Default for WorktreeProcessManager {
    fn default() -> Self {
        Self::new(8)
    }
}

impl WorktreeProcessManager {
    pub fn new(max_per_worktree: usize) -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            by_worktree: Arc::new(RwLock::new(HashMap::new())),
            max_per_worktree,
        }
    }

    pub async fn spawn(
        &self,
        worktree_id: WorktreeId,
        worktree_path: &Path,
        command: &str,
        args: &[&str],
        envs: Vec<(&str, &str)>,
    ) -> Result<WorktreeProcess> {
        let count = self
            .by_worktree
            .read()
            .await
            .get(&worktree_id)
            .map(|v| v.len())
            .unwrap_or(0);
        if count >= self.max_per_worktree {
            return Err(PorpoiseError::ResourceLimit(format!(
                "max processes per worktree ({}) reached",
                self.max_per_worktree
            )));
        }

        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .current_dir(worktree_path);

        for (k, v) in &envs {
            cmd.env(k, v);
        }

        let child = cmd
            .spawn()
            .map_err(|e| PorpoiseError::Runtime(format!("spawn '{command}': {e}")))?;
        let pid = child
            .id()
            .ok_or_else(|| PorpoiseError::Runtime("no pid from spawned process".into()))?;

        let process_id = ProcessId::new();
        let info = WorktreeProcess {
            process_id,
            worktree_id,
            pid,
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            working_dir: worktree_path.to_path_buf(),
            started_at: Utc::now(),
        };

        self.processes.write().await.insert(
            process_id,
            ManagedProcess {
                info: info.clone(),
                child: Some(child),
            },
        );
        self.by_worktree
            .write()
            .await
            .entry(worktree_id)
            .or_default()
            .push(process_id);

        Ok(info)
    }

    pub async fn list(&self) -> Vec<WorktreeProcess> {
        self.processes.read().await.values().map(|e| e.info.clone()).collect()
    }

    pub async fn list_for_worktree(&self, worktree_id: WorktreeId) -> Vec<WorktreeProcess> {
        let ids = self
            .by_worktree
            .read()
            .await
            .get(&worktree_id)
            .cloned()
            .unwrap_or_default();
        let procs = self.processes.read().await;
        ids.iter()
            .filter_map(|id| procs.get(id).map(|e| e.info.clone()))
            .collect()
    }

    pub async fn kill(&self, process_id: ProcessId) -> Result<()> {
        let mut entry = self
            .processes
            .write()
            .await
            .remove(&process_id)
            .ok_or_else(|| PorpoiseError::Runtime(format!("process {process_id} not found")))?;

        if let Some(ref mut child) = entry.child {
            child
                .start_kill()
                .map_err(|e| PorpoiseError::Runtime(format!("kill: {e}")))?;
            child
                .wait()
                .await
                .map_err(|e| PorpoiseError::Runtime(format!("wait: {e}")))?;
        }

        if let Some(ids) = self.by_worktree.write().await.get_mut(&entry.info.worktree_id) {
            ids.retain(|id| *id != process_id);
        }

        Ok(())
    }

    pub async fn kill_worktree(&self, worktree_id: WorktreeId) -> Result<()> {
        let ids = self.by_worktree.write().await.remove(&worktree_id).unwrap_or_default();
        for id in ids {
            self.kill(id).await.ok();
        }
        Ok(())
    }

    pub async fn shutdown_all(&self) -> Result<()> {
        let ids: Vec<ProcessId> = self.processes.read().await.keys().copied().collect();
        for id in ids {
            self.kill(id).await.ok();
        }
        self.by_worktree.write().await.clear();
        Ok(())
    }

    pub async fn count(&self) -> usize {
        self.processes.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test_spawn_and_list() {
        let dir = TempDir::new().unwrap();
        let mgr = WorktreeProcessManager::new(4);
        let wt = WorktreeId::new();

        #[cfg(unix)]
        let (cmd, args) = ("sleep", vec!["10"]);
        #[cfg(windows)]
        let (cmd, args) = ("ping", vec!["-n", "10", "127.0.0.1"]);

        let proc = mgr.spawn(wt, dir.path(), cmd, &args, vec![]).await.unwrap();
        assert_eq!(proc.worktree_id, wt);
        assert_eq!(mgr.list().await.len(), 1);
        assert_eq!(mgr.list_for_worktree(wt).await.len(), 1);

        mgr.kill(proc.process_id).await.unwrap();
        assert_eq!(mgr.list().await.len(), 0);
    }

    #[tokio::test]
    async fn test_max_per_worktree() {
        let dir = TempDir::new().unwrap();
        let mgr = WorktreeProcessManager::new(2);
        let wt = WorktreeId::new();

        #[cfg(unix)]
        let (cmd, args) = ("sleep", vec!["10"]);
        #[cfg(windows)]
        let (cmd, args) = ("ping", vec!["-n", "10", "127.0.0.1"]);

        mgr.spawn(wt, dir.path(), cmd, &args, vec![]).await.unwrap();
        mgr.spawn(wt, dir.path(), cmd, &args, vec![]).await.unwrap();
        let result = mgr.spawn(wt, dir.path(), cmd, &args, vec![]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_kill_worktree() {
        let dir = TempDir::new().unwrap();
        let mgr = WorktreeProcessManager::new(10);
        let wt1 = WorktreeId::new();
        let wt2 = WorktreeId::new();

        #[cfg(unix)]
        let (cmd, args) = ("sleep", vec!["10"]);
        #[cfg(windows)]
        let (cmd, args) = ("ping", vec!["-n", "10", "127.0.0.1"]);

        mgr.spawn(wt1, dir.path(), cmd, &args, vec![]).await.unwrap();
        mgr.spawn(wt1, dir.path(), cmd, &args, vec![]).await.unwrap();
        mgr.spawn(wt2, dir.path(), cmd, &args, vec![]).await.unwrap();

        assert_eq!(mgr.count().await, 3);
        mgr.kill_worktree(wt1).await.unwrap();
        assert_eq!(mgr.list_for_worktree(wt1).await.len(), 0);
        assert_eq!(mgr.list_for_worktree(wt2).await.len(), 1);
    }
}
