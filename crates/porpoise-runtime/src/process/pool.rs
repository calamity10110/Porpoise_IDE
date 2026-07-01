use std::sync::Arc;
use porpoise_core::error::{PorpoiseError, Result};
use super::ProcessManager;

pub struct ProcessPool {
    manager: Arc<ProcessManager>,
    max_size: usize,
}

impl ProcessPool {
    pub fn new(manager: Arc<ProcessManager>, max_size: usize) -> Self {
        Self { manager, max_size }
    }

    pub async fn check_available(&self) -> Result<()> {
        let count = self.manager.list().await.len();
        if count >= self.max_size {
            return Err(PorpoiseError::ResourceLimit(
                format!("max processes ({}) reached", self.max_size),
            ));
        }
        Ok(())
    }

    pub fn max_size(&self) -> usize {
        self.max_size
    }
}
