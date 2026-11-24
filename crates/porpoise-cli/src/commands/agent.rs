use porpoise_core::error::Result;

use crate::{app::AgentAction, daemon, output::OutputFormat};

pub async fn handle(args: crate::app::AgentArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        AgentAction::List => {
            let body = daemon::call("agent_list", serde_json::json!({})).await?;
            Ok(format.format(&body))
        }
        AgentAction::Run { kind, worktree, prompt } => {
            let body = daemon::call(
                "agent_run",
                serde_json::json!({
                    "kind": kind, "worktree": worktree, "prompt": prompt
                }),
            )
            .await?;
            Ok(format!("started agent: {body}"))
        }
        AgentAction::Stop { id } => {
            daemon::call("agent_stop", serde_json::json!({"id": id})).await?;
            Ok(format!("agent {id} stopped"))
        }
        AgentAction::Logs { id, lines } => {
            let body = daemon::call("agent_logs", serde_json::json!({"id": id, "lines": lines})).await?;
            Ok(format!("{body}"))
        }
        AgentAction::AguiRun {
            kind,
            worktree,
            prompt,
            thread,
        } => {
            let body = daemon::call(
                "agent_agui_run",
                serde_json::json!({
                    "kind": kind,
                    "worktree": worktree,
                    "prompt": prompt,
                    "thread_id": thread,
                }),
            )
            .await?;
            Ok(format.format(&body))
        }
        AgentAction::AguiEvents { run_id, after, follow } => {
            if !follow {
                let body = daemon::call(
                    "agent_agui_events",
                    serde_json::json!({ "run_id": run_id, "after": after }),
                )
                .await?;
                return Ok(format.format(&body));
            }
            // Follow mode: poll the run's log and write each new event to
            // stdout as an SSE frame, stopping at the terminal event. The
            // frames stream out directly, so nothing is returned for the
            // caller to print or page.
            follow_run(&run_id, after).await?;
            Ok(String::new())
        }
        AgentAction::AguiStop { run_id } => {
            let body = daemon::call("agent_agui_stop", serde_json::json!({ "run_id": run_id })).await?;
            Ok(format.format(&body))
        }
    }
}

/// Poll `agent_agui_events` until the run finishes, printing every new
/// event as an SSE `data:` frame and flushing so a pipe or terminal sees
/// each event as it lands.
async fn follow_run(run_id: &str, mut cursor: usize) -> Result<()> {
    use std::io::Write;

    let mut stdout = std::io::stdout().lock();
    loop {
        let body = daemon::call(
            "agent_agui_events",
            serde_json::json!({ "run_id": run_id, "after": cursor }),
        )
        .await?;
        let events = body
            .get("events")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let finished = body.get("finished").and_then(|v| v.as_bool()).unwrap_or(false);

        for event in &events {
            // A broken stdout (closed pipe) ends the follow quietly.
            if writeln!(stdout, "data: {event}\n").is_err() {
                return Ok(());
            }
        }
        if !events.is_empty() && stdout.flush().is_err() {
            return Ok(());
        }
        cursor += events.len();

        if finished {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    }
    Ok(())
}
