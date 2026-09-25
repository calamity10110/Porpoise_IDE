use std::{sync::Arc, time::Duration};

use crate::process::ProcessManager;

pub async fn setup_signal_handlers(process_manager: Arc<ProcessManager>) {
    #[cfg(not(unix))]
    {
        let _ = &process_manager;
    }
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut term = signal(SignalKind::terminate()).expect("failed to register SIGTERM handler");
        let mut int = signal(SignalKind::interrupt()).expect("failed to register SIGINT handler");

        tokio::select! {
            _ = term.recv() => {
                tracing::info!("received SIGTERM — shutting down");
                shutdown(&process_manager).await;
            }
            _ = int.recv() => {
                tracing::info!("received SIGINT — shutting down");
                shutdown(&process_manager).await;
            }
        }
    }
}

#[allow(dead_code)]
async fn shutdown(process_manager: &ProcessManager) {
    tracing::info!(
        "graceful shutdown: killing {} processes",
        process_manager.list().await.len()
    );
    process_manager.shutdown_all(Duration::from_secs(5)).await.ok();
    std::process::exit(0);
}
