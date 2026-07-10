use porpoise_core::{config::AppConfig, error::PorpoiseError};
use porpoise_server::Daemon;

#[tokio::main]
async fn main() -> Result<(), PorpoiseError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = AppConfig::default();
    let data_dir = AppConfig::default_data_dir()?;
    std::fs::create_dir_all(&data_dir).map_err(|e| PorpoiseError::Config(format!("cannot create data dir: {e}")))?;

    let db_path = data_dir.join("porpoise.db");
    let mut daemon = Daemon::new(config, db_path).await?;
    daemon.start().await?;
    tracing::info!("porpoise-server started");
    daemon.run().await
}
