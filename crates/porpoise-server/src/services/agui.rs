//! AG-UI run service: spawn an agent, translate its output into AG-UI
//! 1.0 events, keep a per-run log for polling clients, and publish every
//! batch on the event bus (which the WebSocket relay forwards to
//! connected clients).
//!
//! Three RPC methods expose it:
//! - `agent_agui_run` — start a run, returns ids immediately
//! - `agent_agui_events` — poll the event log from an offset (the CLI
//!   prints these as SSE frames)
//! - `agent_agui_stop` — interrupt a live run (it ends with the
//!   `cancelled` outcome)

use std::{collections::HashMap, path::Path, sync::Arc, time::Duration};

use porpoise_agent::{AgentHandle, AgentPool};
use porpoise_agui::{AgUiInput, AgUiSession, Event};
use porpoise_core::{
    bus::EventBus,
    error::{PorpoiseError, Result},
    state::AppState,
    types::{
        event::{AgentEvent, SystemEvent},
        id::AgentId,
    },
};
use tokio::sync::{Mutex, Notify};
use uuid::Uuid;

/// How long a stop request waits for the process to acknowledge before
/// the run task simply keeps reading to its natural end.
const STOP_GRACE: Duration = Duration::from_millis(50);

/// One AG-UI run: its identity, the ordered event log, and the stop
/// signal understood by the task that streams it.
struct AgUiRun {
    agent_id: AgentId,
    thread_id: String,
    events: Vec<serde_json::Value>,
    finished: bool,
    stop: Arc<Notify>,
}

/// Shared registry of AG-UI runs, used by the run task (append/publish)
/// and by the RPC handlers (poll/stop).
#[derive(Default)]
pub struct AgUiRunRegistry {
    runs: Mutex<HashMap<String, AgUiRun>>,
}

impl AgUiRunRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    async fn insert(&self, run_id: String, agent_id: AgentId, thread_id: String) -> Arc<Notify> {
        let stop = Arc::new(Notify::new());
        self.runs.lock().await.insert(
            run_id,
            AgUiRun {
                agent_id,
                thread_id,
                events: Vec::new(),
                finished: false,
                stop: stop.clone(),
            },
        );
        stop
    }

    /// Append a batch to the run's log and return the serialised batch
    /// for the event bus. An unknown run yields an empty batch.
    async fn append(&self, run_id: &str, events: &[Event]) -> Vec<serde_json::Value> {
        let mut runs = self.runs.lock().await;
        let Some(run) = runs.get_mut(run_id) else {
            return Vec::new();
        };
        let mut batch = Vec::with_capacity(events.len());
        for event in events {
            match serde_json::to_value(event) {
                Ok(value) => {
                    run.events.push(value.clone());
                    batch.push(value);
                }
                Err(e) => tracing::warn!(%run_id, "AG-UI event could not be serialised: {e}"),
            }
            if event.is_terminal() {
                run.finished = true;
            }
        }
        batch
    }

    /// The events appended after `after`, with enough context for a
    /// polling client to render and to know when the run is over.
    async fn events_after(&self, run_id: &str, after: usize) -> Option<serde_json::Value> {
        let runs = self.runs.lock().await;
        let run = runs.get(run_id)?;
        let start = after.min(run.events.len());
        Some(serde_json::json!({
            "run_id": run_id,
            "thread_id": run.thread_id,
            "agent_id": run.agent_id.to_string(),
            "events": run.events[start..],
            "total": run.events.len(),
            "finished": run.finished,
        }))
    }

    /// Ask a live run to stop. Returns `false` when the run has already
    /// ended; `None` when the run id is unknown.
    async fn request_stop(&self, run_id: &str) -> Option<bool> {
        let runs = self.runs.lock().await;
        let run = runs.get(run_id)?;
        if run.finished {
            return Some(false);
        }
        // `notify_one` stores a permit, so a stop that arrives before the
        // run task reaches its select is not lost.
        run.stop.notify_one();
        Some(true)
    }
}

/// Start an AG-UI run: spawn the agent with `prompt`, emit `RUN_STARTED`,
/// and detach a task that streams translated output until the process
/// exits (or is stopped).
pub async fn handle_agui_run(
    registry: &Arc<AgUiRunRegistry>,
    pool: &Arc<AgentPool>,
    state: &AppState,
    kind: &str,
    worktree: &str,
    prompt: &str,
    thread_id: Option<&str>,
) -> Result<serde_json::Value> {
    let agent_kind = super::agent::parse_kind(kind);
    let worktree_path = Path::new(worktree);
    let run_id = Uuid::now_v7().to_string();
    let thread_id = thread_id
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    let mut session = AgUiSession::with_ids(&thread_id, &run_id);
    // Spawn first: a spawn failure must not leave a phantom run behind.
    let agent_id = pool.spawn_with_prompt(agent_kind, worktree_path, Some(prompt)).await?;
    let stop = registry.insert(run_id.clone(), agent_id, thread_id.clone()).await;

    emit(registry, state.event_bus(), agent_id, &run_id, &session.begin()).await;

    let registry = registry.clone();
    let pool = pool.clone();
    let bus = state.event_bus().clone();
    let task_run_id = run_id.clone();
    tokio::spawn(async move {
        stream_run(&mut session, &pool, &registry, &bus, agent_id, &task_run_id, &stop).await;
    });

    Ok(serde_json::json!({
        "run_id": run_id,
        "thread_id": thread_id,
        "agent_id": agent_id.to_string(),
        "protocol_version": porpoise_agui::PROTOCOL_VERSION,
        "status": "started",
    }))
}

/// Read the agent's output until the run is terminal, translating each
/// chunk into AG-UI events and publishing every batch. Takes the handle
/// back out afterwards so the caller can return it to the pool.
async fn stream_run(
    session: &mut AgUiSession,
    pool: &Arc<AgentPool>,
    registry: &Arc<AgUiRunRegistry>,
    bus: &EventBus,
    agent_id: AgentId,
    run_id: &str,
    stop: &Arc<Notify>,
) {
    // Take the handle out of the pool: this task alone reads the process.
    let handle = match pool.take_handle(agent_id).await {
        Some(handle) => handle,
        None => {
            let tail = session.finish_error("agent handle unavailable");
            emit(registry, bus, agent_id, run_id, &tail).await;
            return;
        }
    };

    let handle = stream_handle(session, handle, registry, bus, agent_id, run_id, stop).await;

    // Hand the handle back for later inspection/stop; if the pool entry
    // vanished meanwhile (agent_stop during the run), `restore_handle`
    // shuts the process down itself.
    if !pool.restore_handle(agent_id, handle).await {
        tracing::debug!(%run_id, "pool entry gone at run end; handle cleaned up by the pool");
    }
}

/// The AG-UI run loop proper, parameterised over the handle so tests can
/// drive the exact production loop with a scripted agent instead of a
/// live process. Returns the handle once the run is terminal.
async fn stream_handle(
    session: &mut AgUiSession,
    mut handle: Box<dyn AgentHandle>,
    registry: &Arc<AgUiRunRegistry>,
    bus: &EventBus,
    agent_id: AgentId,
    run_id: &str,
    stop: &Arc<Notify>,
) -> Box<dyn AgentHandle> {
    let mut stopped = false;
    let mut read_error: Option<String> = None;

    loop {
        // Each iteration owns the read future inside the select, so the
        // stop branch can use `handle` again after the select resolves.
        enum Step {
            Stopped,
            Read(Result<Option<porpoise_agent::types::AgentOutput>>),
        }
        let step = tokio::select! {
            biased;
            _ = stop.notified() => Step::Stopped,
            read = handle.read_output() => Step::Read(read),
        };

        match step {
            Step::Stopped => {
                if !stopped {
                    stopped = true;
                    if let Err(e) = handle.interrupt().await {
                        tracing::warn!(%run_id, "AG-UI stop interrupt failed: {e}");
                    }
                }
            }
            Step::Read(Err(e)) => {
                read_error = Some(format!("agent output read failed: {e}"));
                break;
            }
            Step::Read(Ok(None)) => {
                if !handle.is_running() {
                    break;
                }
                // A handle that reports no data while still running: back
                // off instead of spinning.
                tokio::time::sleep(STOP_GRACE).await;
            }
            Step::Read(Ok(Some(output))) => {
                let input = AgUiInput::from(&output);
                let is_exit = matches!(input, AgUiInput::Done { .. });
                // A user stop must end as `cancelled`, not as whatever
                // exit code the signal produced.
                let events = if stopped && is_exit {
                    session.cancel()
                } else {
                    session.feed(input)
                };
                emit(registry, bus, agent_id, run_id, &events).await;
                if session.is_finished() {
                    break;
                }
            }
        }
    }

    // Guarantee exactly one terminal event even when the stream ended
    // without an exit report (read error, mock handle, stop race).
    if !session.is_finished() {
        let tail = if stopped {
            session.cancel()
        } else if let Some(message) = read_error {
            session.finish_error(message)
        } else {
            session.finish_ok()
        };
        emit(registry, bus, agent_id, run_id, &tail).await;
    }

    handle
}

/// Append events to the run log and publish them as one batch on the
/// event bus. Empty batches are skipped.
async fn emit(registry: &Arc<AgUiRunRegistry>, bus: &EventBus, agent_id: AgentId, run_id: &str, events: &[Event]) {
    if events.is_empty() {
        return;
    }
    let batch = registry.append(run_id, events).await;
    if batch.is_empty() {
        return;
    }
    bus.publish(SystemEvent::Agent(AgentEvent::AgUi {
        id: agent_id,
        run_id: run_id.to_string(),
        events: batch,
    }));
}

/// Poll a run's event log from offset `after`.
pub async fn handle_agui_events(
    registry: &Arc<AgUiRunRegistry>,
    run_id: &str,
    after: usize,
) -> Result<serde_json::Value> {
    registry
        .events_after(run_id, after)
        .await
        .ok_or_else(|| PorpoiseError::Agent(format!("unknown AG-UI run: {run_id}")))
}

/// Interrupt a live run; it ends with the `cancelled` outcome.
pub async fn handle_agui_stop(registry: &Arc<AgUiRunRegistry>, run_id: &str) -> Result<serde_json::Value> {
    match registry.request_stop(run_id).await {
        Some(stopping) => Ok(serde_json::json!({ "run_id": run_id, "stopping": stopping })),
        None => Err(PorpoiseError::Agent(format!("unknown AG-UI run: {run_id}"))),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::atomic::{AtomicBool, Ordering},
    };

    use porpoise_agent::types::{AgentOutput, OutputType};

    use super::*;

    fn registry() -> Arc<AgUiRunRegistry> {
        AgUiRunRegistry::new()
    }

    fn app_state() -> (EventBus, AppState) {
        let bus = EventBus::new(64);
        let state = AppState::new(porpoise_core::config::AppConfig::default(), bus.clone());
        (bus, state)
    }

    /// An agent handle that replays a fixed script of outputs, so the
    /// production AG-UI loop can be tested without spawning a process.
    struct ScriptedHandle {
        outputs: VecDeque<AgentOutput>,
        running: bool,
    }

    impl ScriptedHandle {
        fn new(outputs: Vec<AgentOutput>) -> Self {
            Self {
                outputs: outputs.into(),
                running: true,
            }
        }

        fn stdout(text: &str) -> AgentOutput {
            AgentOutput {
                text: text.to_string(),
                output_type: OutputType::Stdout,
            }
        }

        fn exit(code: i32) -> AgentOutput {
            AgentOutput {
                text: String::new(),
                output_type: OutputType::Exit(code),
            }
        }
    }

    #[async_trait::async_trait]
    impl AgentHandle for ScriptedHandle {
        fn pid(&self) -> Option<u32> {
            Some(42)
        }
        async fn read_output(&mut self) -> Result<Option<AgentOutput>> {
            match self.outputs.pop_front() {
                Some(output) => {
                    if matches!(output.output_type, OutputType::Exit(_)) {
                        self.running = false;
                    }
                    Ok(Some(output))
                }
                None => Ok(None),
            }
        }
        async fn send_input(&mut self, _text: &str) -> Result<()> {
            Ok(())
        }
        async fn interrupt(&mut self) -> Result<()> {
            Ok(())
        }
        async fn shutdown(&mut self) -> Result<()> {
            self.running = false;
            Ok(())
        }
        fn is_running(&self) -> bool {
            self.running
        }
    }

    /// Collect every AG-UI batch published on the bus for `run_id`.
    fn collect_agui_batches(
        rx: &mut tokio::sync::broadcast::Receiver<SystemEvent>,
        run_id: &str,
    ) -> Vec<serde_json::Value> {
        let mut batches = Vec::new();
        while let Ok(event) = rx.try_recv() {
            if let SystemEvent::Agent(AgentEvent::AgUi { run_id: id, events, .. }) = event
                && id == run_id
            {
                batches.extend(events);
            }
        }
        batches
    }

    #[tokio::test]
    async fn scripted_run_streams_to_terminal_event() {
        let (bus, state) = app_state();
        let mut rx = bus.subscribe();
        let reg = registry();
        reg.insert("run-1".into(), AgentId::new(), "thread-1".into()).await;

        let mut session = AgUiSession::with_ids("thread-1", "run-1");
        let started = session.begin();
        reg.append("run-1", &started).await;
        bus.publish(SystemEvent::Agent(AgentEvent::AgUi {
            id: AgentId::new(),
            run_id: "run-1".into(),
            events: started.iter().map(|e| serde_json::to_value(e).unwrap()).collect(),
        }));

        let handle = Box::new(ScriptedHandle::new(vec![
            ScriptedHandle::stdout("Hello from the agent"),
            ScriptedHandle::exit(0),
        ]));
        let stop = Arc::new(Notify::new());
        let agent_id = AgentId::new();
        let _state = state;
        let returned = stream_handle(&mut session, handle, &reg, &bus, agent_id, "run-1", &stop).await;
        assert!(!returned.is_running(), "script ended with an exit");

        // The log ends with exactly one terminal event.
        let reply = reg.events_after("run-1", 0).await.expect("reply");
        assert_eq!(reply["finished"], true);
        let events = reply["events"].as_array().unwrap();
        assert_eq!(events[0]["type"], "RUN_STARTED");
        assert_eq!(events.last().unwrap()["type"], "RUN_FINISHED");
        assert_eq!(
            events
                .iter()
                .filter(|e| e["type"] == "RUN_FINISHED" || e["type"] == "RUN_ERROR")
                .count(),
            1,
            "exactly one terminal event"
        );
        // The text arrived as a streamed message.
        let text: String = events
            .iter()
            .filter(|e| e["type"] == "TEXT_MESSAGE_CONTENT")
            .map(|e| e["delta"].as_str().unwrap_or_default())
            .collect();
        assert_eq!(text, "Hello from the agent");

        // The same events reached the bus for WebSocket consumers.
        let published = collect_agui_batches(&mut rx, "run-1");
        assert_eq!(published.len(), events.len(), "bus and log agree");
        assert_eq!(published[0]["type"], "RUN_STARTED");
        assert_eq!(published[0]["protocolVersion"], "1.0");
    }

    /// A handle whose read blocks until interrupted: the stop path must
    /// interrupt it and close the run with the `cancelled` outcome.
    struct BlockingHandle {
        interrupted: Arc<AtomicBool>,
        resumed: tokio::sync::Notify,
        exited: AtomicBool,
        running: AtomicBool,
    }

    #[async_trait::async_trait]
    impl AgentHandle for BlockingHandle {
        fn pid(&self) -> Option<u32> {
            Some(43)
        }
        async fn read_output(&mut self) -> Result<Option<AgentOutput>> {
            if self.exited.load(Ordering::SeqCst) {
                return Ok(None);
            }
            // Block until the run is interrupted, then report the exit
            // a SIGINT would produce.
            self.resumed.notified().await;
            self.exited.store(true, Ordering::SeqCst);
            self.running.store(false, Ordering::SeqCst);
            Ok(Some(AgentOutput {
                text: String::new(),
                output_type: OutputType::Exit(-1),
            }))
        }
        async fn send_input(&mut self, _text: &str) -> Result<()> {
            Ok(())
        }
        async fn interrupt(&mut self) -> Result<()> {
            self.interrupted.store(true, Ordering::SeqCst);
            self.resumed.notify_one();
            Ok(())
        }
        async fn shutdown(&mut self) -> Result<()> {
            self.running.store(false, Ordering::SeqCst);
            Ok(())
        }
        fn is_running(&self) -> bool {
            self.running.load(Ordering::SeqCst)
        }
    }

    #[tokio::test]
    async fn stop_interrupts_and_closes_run_as_cancelled() {
        let (bus, _state) = app_state();
        let reg = registry();
        let agent_id = AgentId::new();
        // The registry owns the stop signal the RPC notifies, and hands
        // the very same handle to the run task — as `handle_agui_run` does.
        let stop = reg.insert("run-2".into(), agent_id, "thread-2".into()).await;

        let mut session = AgUiSession::with_ids("thread-2", "run-2");
        let _ = session.begin();

        let interrupted = Arc::new(AtomicBool::new(false));
        let handle = Box::new(BlockingHandle {
            interrupted: interrupted.clone(),
            resumed: tokio::sync::Notify::new(),
            exited: AtomicBool::new(false),
            running: AtomicBool::new(true),
        });

        // The spawned task owns everything it needs (`'static`), while
        // the test keeps its own Arc clone for assertions.
        let task = tokio::spawn({
            let reg = reg.clone();
            let bus = bus.clone();
            let stop = stop.clone();
            async move { stream_handle(&mut session, handle, &reg, &bus, agent_id, "run-2", &stop).await }
        });
        // Let the loop reach its select, then stop the run.
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(reg.request_stop("run-2").await, Some(true));

        let _returned = tokio::time::timeout(Duration::from_secs(5), task)
            .await
            .expect("run task finishes after stop")
            .expect("join");
        assert!(interrupted.load(Ordering::SeqCst), "stop interrupts the agent");

        // The run ends with the cancelled outcome, once.
        let reply = reg.events_after("run-2", 0).await.expect("reply");
        assert_eq!(reply["finished"], true);
        let events = reply["events"].as_array().unwrap();
        let terminal = events.last().unwrap();
        assert_eq!(terminal["type"], "RUN_FINISHED");
        assert_eq!(terminal["outcome"]["type"], "cancelled");
    }

    #[tokio::test]
    async fn spawn_failure_leaves_no_phantom_run() {
        let (_bus, state) = app_state();
        let reg = registry();
        let pool = Arc::new(AgentPool::new(2));

        let result = handle_agui_run(&reg, &pool, &state, "no-such-agent-xyz", ".", "hello", None).await;
        assert!(result.is_err(), "an unspawnable agent must fail the RPC");
        assert!(reg.runs.lock().await.is_empty(), "no phantom run is registered");
    }

    #[tokio::test]
    async fn append_orders_events_and_marks_terminal() {
        let reg = registry();
        reg.insert("run-1".into(), AgentId::new(), "thread-1".into()).await;

        let mut session = AgUiSession::with_ids("thread-1", "run-1");
        let first = session.begin();
        let batch = reg.append("run-1", &first).await;
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0]["type"], "RUN_STARTED");

        let tail = session.finish_ok();
        let batch = reg.append("run-1", &tail).await;
        assert_eq!(batch.last().unwrap()["type"], "RUN_FINISHED");

        let reply = reg.events_after("run-1", 0).await.expect("reply");
        assert_eq!(reply["total"], 2);
        assert_eq!(reply["finished"], true);
        assert_eq!(reply["thread_id"], "thread-1");

        // Polling from an offset only returns the tail.
        let reply = reg.events_after("run-1", 1).await.expect("reply");
        assert_eq!(reply["events"].as_array().unwrap().len(), 1);
        assert_eq!(reply["events"][0]["type"], "RUN_FINISHED");

        // Unknown runs are reported, not fabricated.
        assert!(reg.events_after("nope", 0).await.is_none());
    }

    #[tokio::test]
    async fn stop_is_idempotent_and_refuses_finished_runs() {
        let reg = registry();
        reg.insert("run-1".into(), AgentId::new(), "thread-1".into()).await;

        assert_eq!(reg.request_stop("run-1").await, Some(true));
        assert_eq!(reg.request_stop("run-1").await, Some(true), "repeat stops still signal");
        assert_eq!(reg.request_stop("missing").await, None);

        // A finished run refuses to be stopped.
        let mut session = AgUiSession::with_ids("thread-1", "run-1");
        let tail = session.finish_ok();
        reg.append("run-1", &tail).await;
        assert_eq!(reg.request_stop("run-1").await, Some(false));
    }
}
