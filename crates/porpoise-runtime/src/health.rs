use std::sync::Arc;
use std::time::Duration;

use porpoise_core::bus::EventBus;
use porpoise_core::error::Result;

use crate::process::ProcessManager;

pub struct HealthChecker {
    process_manager: Arc<ProcessManager>,
    interval: Duration,
    _event_bus: EventBus,
}

impl HealthChecker {
    pub fn new(pm: Arc<ProcessManager>, interval: Duration, event_bus: EventBus) -> Self {
        Self { process_manager: pm, interval, _event_bus: event_bus }
    }

    pub async fn run(&self) {
        let mut ticker = tokio::time::interval(self.interval);
        loop {
            ticker.tick().await;
            if let Err(e) = self.check().await {
                tracing::warn!("health check failed: {e}");
            }
        }
    }

    async fn check(&self) -> Result<()> {
        for handle in self.process_manager.list().await {
            let status = handle.status_rx.borrow().clone();
            match status {
                crate::process::ProcessStatus::Running => {
                    #[cfg(unix)]
                    {
                        let exists = unsafe { libc::kill(handle.pid as i32, 0) == 0 };
                        if !exists {
                            tracing::warn!(
                                "process {} (PID {}) dead but not reaped",
                                handle.id, handle.pid
                            );
                            self.process_manager.kill(handle.id).await.ok();
                        }
                    }
                }
                crate::process::ProcessStatus::Exited(_) | crate::process::ProcessStatus::Killed => {}
                crate::process::ProcessStatus::Error(ref e) => {
                    tracing::error!("process {} error: {e}", handle.id);
                }
                crate::process::ProcessStatus::Signalled(ref sig) => {
                    tracing::warn!("process {} killed by signal {sig}", handle.id);
                }
            }
        }
        Ok(())
    }
}
