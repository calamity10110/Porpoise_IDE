use chrono::{DateTime, Utc};
use porpoise_core::error::Result;

pub async fn handle_health(start_time: &DateTime<Utc>, agent_count: usize) -> Result<serde_json::Value> {
    let uptime = chrono::Utc::now().signed_duration_since(*start_time);
    Ok(serde_json::json!({
        "status": "ok",
        "uptime_seconds": uptime.num_seconds(),
        "started_at": start_time.to_rfc3339(),
        "now": Utc::now().to_rfc3339(),
        "agents": agent_count,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
