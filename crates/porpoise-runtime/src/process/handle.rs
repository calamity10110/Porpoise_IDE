use chrono::{DateTime, Utc};
use tokio::sync::{mpsc, watch};

use porpoise_core::types::id::ProcessId;

use super::{ProcessCommand, ProcessKind, ProcessStatus};

#[derive(Debug, Clone)]
pub struct ProcessHandle {
    pub id: ProcessId,
    pub pid: u32,
    pub kind: ProcessKind,
    pub created_at: DateTime<Utc>,
    pub status_rx: watch::Receiver<ProcessStatus>,
    pub cmd_tx: mpsc::Sender<ProcessCommand>,
}
