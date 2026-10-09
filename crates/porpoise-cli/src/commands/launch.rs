use std::{
    io::{IsTerminal as _, Write as _},
    process::{Command, Stdio},
    time::Duration,
};

use porpoise_core::error::{PorpoiseError, Result};

use crate::{app::LaunchArgs, daemon, output::OutputFormat};

const MAX_ATTEMPTS: u32 = 30;
const RETRY_DELAY: Duration = Duration::from_secs(1);
const BAR_WIDTH: usize = 24;
const DEFAULT_WS_PORT: &str = "9876";

pub async fn handle(args: LaunchArgs, format: &OutputFormat) -> Result<String> {
    let only_app = args.app && !args.server;
    let only_server = args.server && !args.app;
    let quiet = !matches!(format, OutputFormat::Plain);

    let daemon_was_up = daemon_healthy().await;

    if daemon_was_up {
        note(quiet, "daemon: already running");
    } else if only_app {
        return Err(PorpoiseError::Ipc(
            "daemon not running; start it with `porpoise launch --server` or `porpoise launch`".into(),
        ));
    } else {
        spawn_detached("porpoise-server", true)?;
        note(quiet, "daemon: spawned porpoise-server");
        wait_for_daemon(quiet).await?;
    }

    let pairing = daemon::call("mobile/pairing_info", serde_json::json!({})).await?;
    let port = pairing["port"].as_str().unwrap_or_default().to_string();
    let token = pairing["token"].as_str().unwrap_or_default().to_string();
    let tls = pairing["tls_enabled"].as_bool().unwrap_or(false);
    let host = effective_host(pairing["host"].as_str().unwrap_or("localhost"));
    let scheme = if tls { "wss" } else { "ws" };
    let url = format!("{scheme}://{host}:{port}");
    let qr_payload = format!("{url}?token={token}");

    if port.is_empty() {
        note(
            quiet,
            "mobile: daemon has no WebSocket listener (PORPOISE_WS_PORT unset at boot); \
             restart it via `porpoise launch --server` to enable pairing",
        );
    }

    if only_server {
        let mut summary = serde_json::json!({
            "daemon": "running",
            "host": host,
            "port": port,
            "token": token,
        });
        if !port.is_empty() {
            summary["mobile_pairing_url"] = serde_json::json!(qr_payload);
        }
        return Ok(format.format(&summary));
    }

    spawn_detached("porpoise-app", false)?;
    note(quiet, "app: spawned porpoise-app");

    let qr_art = if port.is_empty() {
        String::new()
    } else {
        format!("\nScan to connect the mobile companion:\n\n{}", render_qr(&qr_payload)?)
    };
    Ok(match format {
        OutputFormat::Plain => {
            format!("Porpoise is up.{qr_art}\n{qr_payload}\nManual pairing: host {host}, port {port}, token {token}")
        }
        _ => {
            let mut summary = serde_json::json!({
                "daemon": "running",
                "app": "launched",
                "host": host,
                "port": port,
                "token": token,
            });
            if !port.is_empty() {
                summary["mobile_pairing_url"] = serde_json::json!(qr_payload);
            }
            format.format(&summary)
        }
    })
}

/// Loopback hosts are unreachable from a phone — substitute this machine's LAN IP.
fn effective_host(host: &str) -> String {
    if !host.is_empty() && host != "localhost" && host != "127.0.0.1" && host != "0.0.0.0" {
        return host.to_string();
    }
    // connect() on UDP only picks a route — no packet is sent. IPv4 first:
    // a [::]-bound socket may refuse the IPv4 literal on some adapters.
    for bind_addr in ["0.0.0.0:0", "[::]:0"] {
        let Ok(sock) = std::net::UdpSocket::bind(bind_addr) else {
            continue;
        };
        if sock.connect("8.8.8.8:80").is_ok()
            && let Ok(addr) = sock.local_addr()
            && !addr.ip().is_loopback()
            && !addr.ip().is_unspecified()
        {
            return addr.ip().to_string();
        }
    }
    "localhost".to_string()
}

/// Pairing-ready daemon env; explicit user env vars always win.
fn mobile_env() -> Vec<(String, String)> {
    let mut envs = Vec::new();
    if std::env::var_os("PORPOISE_WS_PORT").is_none() {
        envs.push(("PORPOISE_WS_PORT".into(), DEFAULT_WS_PORT.into()));
    }
    if std::env::var_os("PORPOISE_WS_TLS").is_none() {
        envs.push(("PORPOISE_WS_TLS".into(), "1".into()));
    }
    if std::env::var_os("PORPOISE_WS_HOSTS").is_none() {
        let ip = effective_host("localhost");
        if ip != "localhost" {
            envs.push(("PORPOISE_WS_HOSTS".into(), format!("localhost,127.0.0.1,{ip}")));
        }
    }
    envs
}

async fn daemon_healthy() -> bool {
    daemon::call("health", serde_json::json!({})).await.is_ok()
}

async fn wait_for_daemon(quiet: bool) -> Result<()> {
    let mut attempt: u32 = 0;
    loop {
        attempt += 1;
        if daemon_healthy().await {
            if !quiet && std::io::stderr().is_terminal() {
                let _ = write!(std::io::stderr(), "\r{}\r", " ".repeat(BAR_WIDTH + 40));
            }
            note(quiet, &format!("daemon: healthy (after {attempt} attempt(s))"));
            return Ok(());
        }
        if attempt >= MAX_ATTEMPTS {
            return Err(PorpoiseError::Ipc(format!(
                "daemon did not become healthy within {MAX_ATTEMPTS}s; check the server log in the porpoise data dir"
            )));
        }
        draw_bar(attempt, quiet);
        tokio::time::sleep(RETRY_DELAY).await;
    }
}

fn draw_bar(attempt: u32, quiet: bool) {
    if quiet || !std::io::stderr().is_terminal() {
        return;
    }
    let bar = render_bar(attempt, MAX_ATTEMPTS, BAR_WIDTH);
    let _ = write!(
        std::io::stderr(),
        "\r  waiting for daemon [{bar}] {attempt}/{MAX_ATTEMPTS}s"
    );
    let _ = std::io::stderr().flush();
}

fn render_bar(attempt: u32, total: u32, width: usize) -> String {
    let ratio = f64::from(attempt.min(total)) / f64::from(total);
    let filled = (ratio * width as f64).round() as usize;
    let filled = filled.min(width);
    format!("{}{}", "=".repeat(filled), "-".repeat(width - filled))
}

fn note(quiet: bool, msg: &str) {
    if !quiet {
        eprintln!("{msg}");
    }
}

fn render_qr(payload: &str) -> Result<String> {
    let code =
        qrcode::QrCode::new(payload.as_bytes()).map_err(|e| PorpoiseError::Internal(format!("qr encode: {e}")))?;
    let dark: Vec<bool> = code.to_colors().into_iter().map(|c| c == qrcode::Color::Dark).collect();
    let width = code.width();
    let mut art = String::new();
    for y in (0..dark.len() / width).step_by(2) {
        art.push_str("  ");
        for x in 0..width {
            let top = dark[y * width + x];
            let bottom = dark.get((y + 1) * width + x).copied().unwrap_or(false);
            let ch = match (top, bottom) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            };
            art.push(ch);
        }
        art.push('\n');
    }
    Ok(art)
}

fn spawn_detached(name: &str, log_output: bool) -> Result<()> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| PorpoiseError::Internal(format!("current exe: {e}")))?
        .parent()
        .ok_or_else(|| PorpoiseError::Internal("no parent dir for current exe".into()))?
        .to_path_buf();
    let exe = exe_dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    if !exe.exists() {
        return Err(PorpoiseError::Internal(format!(
            "{} not found next to the CLI; build it with `cargo build -p {}` first",
            exe.display(),
            name
        )));
    }

    let mut cmd = Command::new(&exe);
    cmd.stdin(Stdio::null());
    if log_output {
        for (key, value) in mobile_env() {
            cmd.env(key, value);
        }
        let log_path = porpoise_core::config::AppConfig::default_data_dir()
            .map_err(|e| PorpoiseError::Config(format!("data dir: {e}")))?
            .join("server.log");
        if let Ok(log) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
            let cloned = log
                .try_clone()
                .map_err(|e| PorpoiseError::Internal(format!("log clone: {e}")))?;
            cmd.stdout(Stdio::from(cloned)).stderr(Stdio::from(log));
        }
    } else {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }
    #[cfg(unix)]
    {
        cmd.process_group(0);
    }
    cmd.spawn()
        .map_err(|e| PorpoiseError::Internal(format!("spawn {name}: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_progresses_and_clamps() {
        assert_eq!(render_bar(0, 30, 10), "----------");
        assert_eq!(render_bar(15, 30, 10), "=====-----");
        assert_eq!(render_bar(30, 30, 10), "==========");
        assert_eq!(render_bar(99, 30, 10), "==========");
    }

    #[test]
    fn qr_renders_for_payload() {
        let art = render_qr("ws://127.0.0.1:9876?token=abc").unwrap();
        assert!(art.contains('█') || art.contains('▀') || art.contains('▄'));
    }

    #[test]
    fn explicit_hosts_pass_through() {
        assert_eq!(effective_host("192.168.1.10"), "192.168.1.10");
        assert_eq!(effective_host("porpoise.local"), "porpoise.local");
    }

    #[test]
    fn lan_ip_resolves_or_falls_back() {
        let host = effective_host("localhost");
        assert!(!host.is_empty());
        assert!(!host.contains(':') || host.parse::<std::net::Ipv6Addr>().is_ok());
    }
}
