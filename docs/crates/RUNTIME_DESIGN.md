# porpoise-runtime: Process & PTY Management

> Async-native process supervision, PTY allocation, and resource management.
> The runtime is the heart of Porpoise — it manages every subprocess and agent session.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                      porpoise-runtime                             │
│                                                                  │
│  ┌──────────────────────┐     ┌──────────────────────────────┐  │
│  │    ProcessManager     │     │        PtyManager            │  │
│  │                       │     │                              │  │
│  │  spawn()              │     │  alloc() → PtySession        │  │
│  │  monitor()            │     │  resize()                    │  │
│  │  kill()               │     │  read() → OutputStream       │  │
│  │  list()               │     │  write() → InputStream       │  │
│  │  signal()             │     │  close()                     │  │
│  └──────────┬───────────┘     └──────────────┬───────────────┘  │
│             │                                │                   │
│  ┌──────────▼────────────────────────────────▼───────────────┐  │
│  │                PtyMultiplexer                               │  │
│  │  Maps multiple PTYs to a single session, handles layout   │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌──────────────────────┐     ┌──────────────────────────────┐  │
│  │   HealthChecker       │     │      ResourceController      │  │
│  │   periodic process    │     │      CPU/mem limits per      │  │
│  │   liveness checks     │     │      process group           │  │
│  └──────────────────────┘     └──────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

---

## 2. ProcessManager

The `ProcessManager` is a tokio-based supervisor for subprocesses. It wraps `tokio::process::Command` with additional monitoring, lifecycle management, and resource control.

```rust
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, watch};
use std::collections::HashMap;

/// Handle to a managed process — used to interact with it after spawn.
#[derive(Debug, Clone)]
pub struct ProcessHandle {
    pub id: ProcessId,
    pub pid: u32,
    pub kind: ProcessKind,
    pub created_at: DateTime<Utc>,
    status_rx: watch::Receiver<ProcessStatus>,
    cmd_tx: mpsc::Sender<ProcessCommand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessKind {
    Agent,
    Shell,
    Git,
    Build,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessStatus {
    Running,
    Exited(i32),
    Signalled(String),
    Killed,
    Error(String),
}

#[derive(Debug)]
pub enum ProcessCommand {
    Kill,
    Signal(String),
    Resize { rows: u16, cols: u16 },
}

/// Supervisor for all spawned processes.
pub struct ProcessManager {
    processes: Arc<RwLock<HashMap<ProcessId, ProcessEntry>>>,
    config: RuntimeConfig,
    event_bus: EventBus,
}

struct ProcessEntry {
    handle: ProcessHandle,
    child: Option<Child>,     // None after child is reaped
    task: JoinHandle<()>,     // monitoring task
}

impl ProcessManager {
    /// Spawn a new process with the given command and arguments.
    /// Returns a ProcessHandle for interaction.
    pub async fn spawn(
        &self,
        kind: ProcessKind,
        cmd: &str,
        args: &[&str],
        cwd: Option<&Path>,
        envs: Vec<(&str, &str)>,
    ) -> Result<ProcessHandle> {
        let id = ProcessId::new();
        
        let mut command = Command::new(cmd);
        command.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        if let Some(dir) = cwd {
            command.current_dir(dir);
        }
        for (k, v) in envs {
            command.env(k, v);
        }
        
        let mut child = command.spawn()
            .map_err(|e| PorpoiseError::Runtime(format!("spawn failed: {e}")))?;
        let pid = child.id().ok_or_else(|| {
            PorpoiseError::Runtime("no pid from spawned process".into())
        })?;
        
        let (status_tx, status_rx) = watch::channel(ProcessStatus::Running);
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<ProcessCommand>(32);
        
        let handle = ProcessHandle {
            id,
            pid,
            kind,
            created_at: Utc::now(),
            status_rx,
            cmd_tx,
        };
        
        // Monitoring task — watches for exit and handles commands
        let child_id = id;
        let event_bus = self.event_bus.clone();
        let monitor_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Wait for process exit
                    exit_status = child.wait() => {
                        let status = match exit_status {
                            Ok(status) if status.success() =>
                                ProcessStatus::Exited(status.code().unwrap_or(0)),
                            Ok(status) =>
                                ProcessStatus::Exited(status.code().unwrap_or(-1)),
                            Err(e) =>
                                ProcessStatus::Error(e.to_string()),
                        };
                        let _ = status_tx.send(status);
                        event_bus.publish(SystemEvent::from(AgentEvent::Exited {
                            id: child_id.into(),
                            exit_code: exit_status.ok()
                                .and_then(|s| s.code()).unwrap_or(-1),
                        }));
                        break;
                    }
                    // Handle commands from the process handle
                    Some(cmd) = cmd_rx.recv() => {
                        match cmd {
                            ProcessCommand::Kill => {
                                let _ = child.kill().await;
                                let _ = status_tx.send(ProcessStatus::Killed);
                                break;
                            }
                            ProcessCommand::Signal(sig) => {
                                #[cfg(unix)]
                                kill_signal(pid, &sig).ok();
                                #[cfg(not(unix))]
                                let _ = child.kill().await;
                            }
                            ProcessCommand::Resize { rows, cols } => {
                                // handled by PtyManager, not here
                            }
                        }
                    }
                }
            }
        });
        
        self.processes.write().await.insert(id, ProcessEntry {
            handle: handle.clone(),
            child: Some(child),
            task: monitor_task,
        });
        
        Ok(handle)
    }
    
    /// Kill a managed process immediately.
    pub async fn kill(&self, id: ProcessId) -> Result<()> {
        let entry = self.processes.write().await.remove(&id)
            .ok_or_else(|| PorpoiseError::Runtime(format!("process {id} not found")))?;
        entry.cmd_tx.send(ProcessCommand::Kill).await.ok();
        Ok(())
    }
    
    /// Gracefully shutdown all managed processes.
    pub async fn shutdown_all(&self, timeout: Duration) -> Result<()> {
        let ids: Vec<ProcessId> = self.processes.read().await.keys().copied().collect();
        
        // Send kill to all
        for id in &ids {
            if let Some(entry) = self.processes.read().await.get(id) {
                entry.cmd_tx.send(ProcessCommand::Kill).await.ok();
            }
        }
        
        // Wait for all monitor tasks
        tokio::time::sleep(timeout).await;
        self.processes.write().await.clear();
        Ok(())
    }
    
    /// List all managed processes.
    pub async fn list(&self) -> Vec<ProcessHandle> {
        self.processes.read().await.values()
            .map(|e| e.handle.clone())
            .collect()
    }
}
```

---

## 3. PtyManager

PTY allocation is platform-specific but exposed through a unified async trait.

```rust
use std::os::unix::io::{FromRawFd, AsRawFd};
use nix::pty::{self, Winsize};
use nix::unistd::{self, ForkResult, Pid};
use libc::{STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO};

/// A PTY session storing file descriptor as i32 for cross-platform compatibility.
pub struct PtySession {
    pub id: TerminalId,
    pub fd: i32,                    // RawFd on Unix, HANDLE on Windows
    pub child_pid: u32,
    pub rows: u16,
    pub cols: u16,
}

/// Manages PTY allocation and I/O.
pub struct PtyManager {
    sessions: Arc<RwLock<HashMap<TerminalId, PtySession>>>,
    event_bus: EventBus,
}

impl PtyManager {
    /// Allocate a new PTY with the given dimensions.
    /// On Unix, this calls nix::pty::forkpty().
    /// On Windows, this calls CreatePseudoConsole().
    pub async fn alloc(&self, rows: u16, cols: u16, shell: &str) -> Result<PtySession> {
        #[cfg(unix)]
        return self.alloc_unix(rows, cols, shell).await;
        
        #[cfg(windows)]
        return self.alloc_windows(rows, cols, shell).await;
    }
    
    #[cfg(unix)]
    async fn alloc_unix(&self, rows: u16, cols: u16, shell: &str) -> Result<PtySession> {
        use nix::pty::Winsize;
        
        // Open a new PTY pair
        let winsize = Winsize {
            ws_row: rows as u16,
            ws_col: cols as u16,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        
        let (ptym, ptys) = pty::openpty(&winsize, None)
            .map_err(|e| PorpoiseError::PtyError(e.to_string()))?;
        
        // Fork
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                // Parent: close slave, keep master
                close(ptys).ok();
                Ok(PtySession {
                    id: TerminalId::new(),
                    master_fd: ptym,
                    child_pid: child,
                    rows,
                    cols,
                })
            }
            Ok(ForkResult::Child) => {
                // Child: set up slave as stdio
                setsid().ok(); // create new session
                close(ptym).ok();
                
                // Duplicate slave fd to stdin/stdout/stderr
                dup2(ptys, STDIN_FILENO).ok();
                dup2(ptys, STDOUT_FILENO).ok();
                dup2(ptys, STDERR_FILENO).ok();
                
                // Close the extra slave fd (dup2 already duplicated it)
                if ptys != STDIN_FILENO && ptys != STDOUT_FILENO && ptys != STDERR_FILENO {
                    close(ptys).ok();
                }
                
                // Execute shell
                std::process::Command::new(shell)
                    .env("TERM", "xterm-256color")
                    .exec();
                
                // Only reached if exec fails
                std::process::exit(1);
            }
            Err(e) => Err(PorpoiseError::PtyError(e.to_string())),
        }
    }
    
    #[cfg(windows)]
    async fn alloc_windows(&self, rows: u16, cols: u16, shell: &str) -> Result<PtySession> {
        // Windows uses CreatePseudoConsole + ConPTY
        // See: https://learn.microsoft.com/en-us/windows/console/createpseudoconsole
        // This requires windows::Win32::System::Console interop
        todo!("Windows PTY via ConPTY API")
    }
    
    /// Read from a PTY master fd (non-blocking, async).
    pub async fn read(&self, id: TerminalId, buf: &mut [u8]) -> Result<usize> {
        let session = self.sessions.read().await;
        let session = session.get(&id)
            .ok_or_else(|| PorpoiseError::Terminal("session not found".into()))?;
        
        // Use tokio::fs::File to wrap raw fd for async I/O
        let mut file = unsafe { File::from_raw_fd(session.master_fd.as_raw_fd()) };
        let n = file.read(buf).await
            .map_err(|e| PorpoiseError::PtyError(e.to_string()))?;
        std::mem::forget(file); // prevent close on drop — ownership stays with PtySession
        Ok(n)
    }
    
    /// Write to a PTY master fd (async).
    pub async fn write(&self, id: TerminalId, data: &[u8]) -> Result<()> {
        let session = self.sessions.read().await;
        let session = session.get(&id)
            .ok_or_else(|| PorpoiseError::Terminal("session not found".into()))?;
        
        let mut file = unsafe { File::from_raw_fd(session.master_fd.as_raw_fd()) };
        file.write_all(data).await
            .map_err(|e| PorpoiseError::PtyError(e.to_string()))?;
        std::mem::forget(file);
        Ok(())
    }
    
    /// Resize a PTY.
    pub fn resize(&self, id: TerminalId, rows: u16, cols: u16) -> Result<()> {
        let session = self.sessions.read().await;
        let session = session.get(&id)
            .ok_or_else(|| PorpoiseError::Terminal("session not found".into()))?;
        
        #[cfg(unix)]
        {
            let ws = nix::pty::Winsize {
                ws_row: rows,
                ws_col: cols,
                ws_xpixel: 0,
                ws_ypixel: 0,
            };
            nix::pty::ptsname_r(&session.master_fd)?;
            // TIOCSWINSZ ioctl on the master fd
            // (nix doesn't expose this directly — we use libc::ioctl)
            let res = unsafe {
                libc::ioctl(
                    session.master_fd.as_raw_fd(),
                    libc::TIOCSWINSZ,
                    &ws,
                )
            };
            if res != 0 {
                return Err(PorpoiseError::PtyError("resize failed".into()));
            }
        }
        
        Ok(())
    }
}
```

---

## 4. WorktreeManager

Bridge between git worktree operations and process management — when a worktree is created for an agent, the runtime spawns the agent in that worktree's directory.

```rust
pub struct WorktreeManager {
    process_manager: Arc<ProcessManager>,
    git_engine: Arc<GitEngine>,
    state: AppState,
}

impl WorktreeManager {
    pub async fn create_agent_worktree(
        &self,
        name: &str,
        repo_path: &Path,
        agent_kind: &str,
        prompt: &str,
    ) -> Result<WorktreeId> {
        // 1. Create git worktree
        let branch = format!("agent/{}/{}", agent_kind, name);
        let wt_path = repo_path.parent().unwrap().join(name);
        self.git_engine.create_worktree(repo_path, &branch, &wt_path).await?;
        
        // 2. Persist worktree metadata
        let id = WorktreeId::new();
        self.state.db().execute(
            "INSERT INTO worktrees (id, repo_path, worktree_path, branch, status)
             VALUES (?1, ?2, ?3, ?4, 'idle')",
            rusqlite::params![id.to_string(), repo_path.to_str(), wt_path.to_str(), branch],
        ).await?;
        
        // 3. Spawn agent in worktree
        let agent_id = AgentId::new();
        let handle = self.process_manager.spawn(
            ProcessKind::Agent,
            agent_kind,
            &[],
            Some(&wt_path),
            vec![("AGENT_PROMPT", prompt)],
        ).await?;
        
        // 4. Emit events
        self.state.event_bus().publish(SystemEvent::Worktree(
            WorktreeEvent::Created { id, path: wt_path, name: name.into() }
        ));
        self.state.event_bus().publish(SystemEvent::Agent(
            AgentEvent::Spawned { id: agent_id, kind: agent_kind.into(), pid: handle.pid }
        ));
        
        Ok(id)
    }
}
```

---

## 5. Health Checker

Periodic process health monitoring — detects hung/stale processes and cleans them up.

```rust
pub struct HealthChecker {
    process_manager: Arc<ProcessManager>,
    interval: Duration,
    event_bus: EventBus,
}

impl HealthChecker {
    pub fn new(pm: Arc<ProcessManager>, interval: Duration, bus: EventBus) -> Self {
        Self { process_manager: pm, interval, bus }
    }
    
    /// Run the health check loop. Should be spawned as a tokio task.
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
                ProcessStatus::Running => {
                    // Check if process is still alive via PID
                    #[cfg(unix)] {
                        let exists = unsafe {
                            libc::kill(handle.pid as i32, 0) == 0
                        };
                        if !exists {
                            tracing::warn!("process {} (PID {}) dead but not reaped",
                                handle.id, handle.pid);
                            self.process_manager.kill(handle.id).await?;
                        }
                    }
                }
                ProcessStatus::Exited(_) | ProcessStatus::Killed => {
                    // Already handled — skip
                }
                ProcessStatus::Error(e) => {
                    tracing::error!("process {} error: {e}", handle.id);
                    self.event_bus.publish(SystemEvent::System(
                        SystemEventKind::Error {
                            message: format!("process {} failed: {e}", handle.id)
                        }
                    ));
                }
                ProcessStatus::Signalled(sig) => {
                    tracing::warn!("process {} killed by signal {sig}", handle.id);
                }
            }
        }
        Ok(())
    }
}
```

---

## 6. Signal Handling (Unix)

```rust
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

/// Set up signal handlers for graceful shutdown.
pub async fn setup_signal_handlers(process_manager: Arc<ProcessManager>) {
    let mut term = signal(SignalKind::terminate())
        .expect("failed to register SIGTERM handler");
    let mut int = signal(SignalKind::interrupt())
        .expect("failed to register SIGINT handler");
    
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

async fn shutdown(process_manager: &ProcessManager) {
    tracing::info!("graceful shutdown: killing {} processes",
        process_manager.list().await.len());
    process_manager.shutdown_all(Duration::from_secs(5)).await.ok();
    std::process::exit(0);
}
```

---

## 7. Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Tokio tasks for each process | Isolated failure, independent lifecycle, straightforward cancel |
| Watch channel for status | Last-value cache + async notification — subscribers always have current state |
| Mpsc channel for commands | Backpressure-safe, bounded queuing for process commands |
| `unsafe` for PTY fd wrapping | Minimal, isolated, well-documented — only place raw fds cross async boundary |
| Platform `#[cfg]` for PTY | PTY APIs are fundamentally different on Unix vs Windows — shared trait + platform impl |
| Health checker as separate task | Decoupled from command handling — health problems don't degrade command latency |

---

## 8. Dependencies

| Crate | Version | Usage |
|-------|---------|-------|
| `tokio` | 1.x | Async runtime, process, sync primitives |
| `nix` | 0.29+ | PTY, fork, Unix-specific operations |
| `signal-hook` / `tokio-signal` | — | Signal handling (Unix) |
| `tracing` | 0.1 | Logging and diagnostics |

---

## Implementation Status (2026-07-01)

| Feature | Status | Notes |
|---------|--------|-------|
| ProcessManager: spawn/kill/list/shutdown_all | ✅ Done | Uses tokio::process::Command |
| PtyManager: alloc/read/write/resize/close | ✅ Done | Platform-conditional impl |
| Unix PTY (forkpty) | ✅ Done | nix::pty::openpty() + fork() in #[cfg(unix)] |
| Windows PTY (ConPTY) | ⬜ Stub | CreatePseudoConsole not yet implemented |
| HealthChecker | ✅ Done | Periodic kill(pid, 0) liveness checks |
| Signal handling (SIGTERM/SIGINT) | ✅ Done | Graceful shutdown via tokio::signal |
| ResourceLimits | ✅ Done | RLIMIT_NOFILE via setrlimit |
| WorktreeProcessManager | ⬜ Not started | Will bridge process spawning with worktree directories |
| cgroups integration | ⬜ Not started | Linux-only resource control |

### Actual Implementation Details

The implementation differs from the initial design in several ways:

1. **fd type**: Uses `i32` instead of `OwnedFd` for cross-platform compatibility (Windows handles don't implement `OwnedFd`)
2. **Async PTY I/O**: Uses `std::fs::File::from_raw_fd()` + `tokio::fs::File::from_std()` pattern (tokio's File doesn't implement `from_raw_fd`)
3. **ProcessManager monitoring**: Simplified to a basic tokio::select! loop (full SIGCHLD monitoring deferred)
4. **Platform gating**: Uses `#[cfg(target_os = "windows")]` instead of `#[cfg(windows)]` for clarity in platform-specific functions
5. **PtySession struct**: Unified struct with platform-specific constructor functions, not separate Unix/Windows types

---

*The runtime is the most platform-sensitive crate in Porpoise. Always test PTY and process operations on the target platform before merging. See the cross-platform test strategy in [DEVELOPMENT_PLAN.md](../DEVELOPMENT_PLAN.md).*
