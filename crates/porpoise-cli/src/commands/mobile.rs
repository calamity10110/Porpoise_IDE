use porpoise_core::error::Result;

use crate::{app::MobileAction, daemon, output::OutputFormat};

pub async fn handle(args: crate::app::MobileArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        MobileAction::Qr => handle_qr(format).await,
    }
}

async fn handle_qr(format: &OutputFormat) -> Result<String> {
    let pairing = daemon::call("mobile/pairing_info", serde_json::json!({})).await?;

    let host = pairing["host"].as_str().unwrap_or("localhost");
    let port = pairing["port"].as_str().unwrap_or("9876");
    let token = pairing["token"].as_str().unwrap_or_default();
    let tls_enabled = pairing["tls_enabled"].as_bool().unwrap_or(false);
    let fp = pairing["tls_fingerprint"].as_str().unwrap_or("");

    let scheme = if tls_enabled { "wss" } else { "ws" };
    let url = format!("{scheme}://{host}:{port}");

    let output = serde_json::json!({
        "url": url,
        "host": host,
        "port": port,
        "tls_enabled": tls_enabled,
        "tls_fingerprint": fp,
        "token": token,
    });

    Ok(format.format(&output))
}
