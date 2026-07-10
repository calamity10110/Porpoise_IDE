use porpoise_core::{
    error::{PorpoiseError, Result},
    types::id::TerminalId,
};
use porpoise_runtime::PtyManager;

pub async fn handle_create_pty(pty: &PtyManager, rows: u16, cols: u16, shell: &str) -> Result<serde_json::Value> {
    let id = pty.alloc(rows, cols, shell).await?;
    Ok(serde_json::json!({ "id": id.to_string(), "status": "created" }))
}

pub async fn handle_send_pty(pty: &PtyManager, id_str: &str, data: &str) -> Result<serde_json::Value> {
    let id: TerminalId = id_str
        .parse()
        .map_err(|_| PorpoiseError::Terminal("invalid terminal id".into()))?;
    pty.write(id, data.as_bytes()).await?;
    Ok(serde_json::json!({ "status": "ok" }))
}

pub async fn handle_read_pty(pty: &PtyManager, id_str: &str, max_bytes: usize) -> Result<serde_json::Value> {
    let id: TerminalId = id_str
        .parse()
        .map_err(|_| PorpoiseError::Terminal("invalid terminal id".into()))?;
    let mut buf = vec![0u8; max_bytes.min(4096)];
    let n = pty.read(id, &mut buf).await?;
    buf.truncate(n);
    let text = String::from_utf8_lossy(&buf).to_string();
    Ok(serde_json::json!({ "id": id_str, "data": text, "bytes": n }))
}

pub async fn handle_resize_pty(pty: &PtyManager, id_str: &str, rows: u16, cols: u16) -> Result<serde_json::Value> {
    let id: TerminalId = id_str
        .parse()
        .map_err(|_| PorpoiseError::Terminal("invalid terminal id".into()))?;
    pty.resize(id, rows, cols).await?;
    Ok(serde_json::json!({ "status": "ok" }))
}

pub async fn handle_close_pty(pty: &PtyManager, id_str: &str) -> Result<serde_json::Value> {
    let id: TerminalId = id_str
        .parse()
        .map_err(|_| PorpoiseError::Terminal("invalid terminal id".into()))?;
    pty.close(id).await?;
    Ok(serde_json::json!({ "status": "closed" }))
}
