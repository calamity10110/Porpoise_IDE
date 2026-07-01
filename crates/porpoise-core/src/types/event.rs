use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use super::id::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SystemEvent {
    Worktree(WorktreeEvent),
    Terminal(TerminalEvent),
    Agent(AgentEvent),
    Git(GitEvent),
    Ssh(SshEvent),
    Browser(BrowserEvent),
    System(SystemEventKind),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorktreeEvent {
    Created {
        id: WorktreeId,
        path: PathBuf,
        name: String,
    },
    Deleted {
        id: WorktreeId,
        path: PathBuf,
    },
    StatusChanged {
        id: WorktreeId,
        status: WorktreeStatus,
    },
    AgentAttached {
        id: WorktreeId,
        agent: AgentId,
    },
    AgentDetached {
        id: WorktreeId,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TerminalEvent {
    Created {
        id: TerminalId,
        session_id: SessionId,
        worktree_id: WorktreeId,
    },
    Output {
        id: TerminalId,
        data: Vec<u8>,
        timestamp: i64,
    },
    Resized {
        id: TerminalId,
        rows: u16,
        cols: u16,
    },
    Closed {
        id: TerminalId,
        exit_code: Option<i32>,
    },
    Bell {
        id: TerminalId,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AgentEvent {
    Spawned {
        id: AgentId,
        kind: String,
        pid: u32,
    },
    Output {
        id: AgentId,
        text: String,
        kind: OutputKind,
    },
    StatusChanged {
        id: AgentId,
        status: AgentStatusKind,
    },
    Error {
        id: AgentId,
        message: String,
    },
    Exited {
        id: AgentId,
        exit_code: i32,
    },
    HookFired {
        id: AgentId,
        hook_type: String,
        payload: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OutputKind {
    Thinking,
    Code,
    ToolCall,
    ToolResult,
    Text,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AgentStatusKind {
    Spawning,
    Running,
    Thinking,
    WaitingForInput,
    Error,
    Completed,
    Killed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GitEvent {
    CloneStarted { repo: String },
    CloneCompleted { path: PathBuf },
    StatusChanged { repo: PathBuf, changes: usize },
    BranchCreated { repo: PathBuf, branch: String },
    WorktreeCreated { repo: PathBuf, path: PathBuf },
    WorktreeDeleted { path: PathBuf },
    FetchCompleted { repo: PathBuf },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SshEvent {
    Connected { host: String },
    Disconnected { host: String },
    Reconnected { host: String },
    PortForwardStarted { host: String, local_port: u16 },
    PortForwardStopped { host: String, local_port: u16 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BrowserEvent {
    PageLoaded { page_id: PageId, url: String },
    SnapshotTaken { page_id: PageId, path: PathBuf },
    ElementClicked { page_id: PageId, selector: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SystemEventKind {
    Startup,
    Shutdown,
    ConfigReloaded,
    LowMemory { available_mb: u64 },
    Error { message: String },
    Notification { title: String, body: String, severity: NotificationSeverity },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorktreeStatus {
    Idle,
    Setup,
    Running { agent: AgentId },
    Completed { exit_code: i32 },
    Failed { error: String },
}
