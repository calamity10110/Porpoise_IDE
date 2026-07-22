use porpoise_core::error::{PorpoiseError, Result};
use porpoise_relay::RelayClient;

pub async fn connect() -> Result<RelayClient> {
    let data_dir = porpoise_core::config::AppConfig::default_data_dir()
        .map_err(|e| PorpoiseError::Config(format!("data dir: {e}")))?;
    std::fs::create_dir_all(&data_dir).map_err(|e| PorpoiseError::Config(format!("mkdir: {e}")))?;
    let socket_path = data_dir.join("porpoise.sock");
    let token_path = data_dir.join("ipc-token");
    let token = if token_path.exists() {
        porpoise_relay::auth::load_token(&token_path)?
    } else {
        String::new()
    };
    RelayClient::connect_with_auth(&socket_path, &token).await
}

pub async fn call(method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
    let client = connect().await?;
    client.call(method, params).await
}
