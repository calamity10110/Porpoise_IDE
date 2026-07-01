# Module Design: porpoise-runtime

> Process supervision, PTY allocation, and resource management — the engine room.

---

## Purpose

`porpoise-runtime` manages every subprocess that Porpoise spawns: AI agents, shell sessions, git operations, and any other child processes. It provides async-native process supervision, PTY (pseudo-terminal) allocation, health monitoring, and resource limits.

**What problem it solves:** Without a dedicated runtime manager, Porpoise would need to handle process lifecycle (spawn, monitor, reap) and PTY I/O manually in every service crate. The runtime centralizes this complexity: concurrent process supervision, cross-platform PTY handling (forkpty on Unix, ConPTY on Windows), signal handling, and resource governance.

---

## Dependencies

### External

| Crate | Version | Why |
|-------|---------|-----|
| `tokio` | 1.x | `process::Command`, async I/O, task supervision |
| `nix` | 0.29+ | `pty::forkpty()`, `unistd::dup2`, signal constants (Unix) |
| `signal-hook` / `tokio-signal` | — | Signal handling (Unix) |
| `tracing` | 0.1 | Process lifecycle logging |
| `libc` | 0.2 | Raw `ioctl()` for PTY resize, `kill()` for health check |

### Internal

| Crate | Dependency Type |
|-------|----------------|
| `porpoise-core` | Import — `ProcessId`, `RuntimeConfig`, `SystemEvent`, `PorpoiseError::Runtime*` |
| `porpoise-relay` | (Optional) For agent output streaming |

---

## Required Input

| Input | Source | Mechanism | Format |
|-------|--------|-----------|--------|
| Spawn requests | Server services | `ProcessManager::spawn()` | `(kind, cmd, args, cwd, envs)` |
| PTY allocate requests | Server/terminal services | `PtyManager::alloc()` | `(rows, cols, shell)` |
| PTY write data | CLI / App via server | `PtyManager::write()` | `&[u8]` (raw bytes) |
| Kill requests | Server services | `ProcessManager::kill()` | `ProcessId` |
| Resize requests | Server services | `PtyManager::resize()` | `(rows, cols)` |
| OS signals | Kernel | `tokio::signal::unix` | `SIGTERM`, `SIGINT`, `SIGHUP` |

---

## Required Output

| Output | Consumer | Mechanism | Format |
|--------|----------|-----------|--------|
| `ProcessHandle` | Caller of `spawn()` | Return value | `{ id, pid, status_rx, cmd_tx }` |
| PTY output bytes | Server terminal service | `mpsc::Receiver` | `Vec<u8>` chunks |
| Process exit events | EventBus subscribers | `ProcessStatus::Exited` | Typed enum |
| Health check results | Server logging | `tracing::warn!()` on failure | Log lines |
| Signal-triggered shutdown | ProcessManager | `shutdown_all()` | Cleans up all children |

---

## Ownership

| Owns | Does Not Own |
|------|-------------|
| Process table (HashMap<PID, ProcessEntry>) | Worktree metadata |
| PTY master file descriptors | Terminal scrollback |
| Child process lifecycle (spawn → reap) | Agent protocol state |
| Signal handler registration | Database |
| Resource limit enforcement | Config persistence |
| Health check interval timer | Event bus (shared reference) |

---

## Program Flow

### Process Lifecycle

```
┌──────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────┐
│  Service  │───>│  ProcessMgr  │───>│  tokios::    │───>│   OS     │
│  requests │    │  .spawn()    │    │  process::   │    │  kernel  │
│  spawn    │    │              │    │  Command     │    │          │
└──────────┘    └──────┬───────┘    └──────────────┘    └──────────┘
                       │
                  ┌────▼─────┐
                  │  Create   │
                  │  Monitor  │
                  │  Task     │
                  │ (tokio)   │
                  │           │
                  │  ┌─────┐  │  ┌──────────────┐    ┌──────────┐
                  │  │ Wait │──│─>│  Child exits  │───>│  Reap    │
                  │  │ for  │  │  └──────────────┘    │  (wait)  │
                  │  │ exit │  │                       └──────────┘
                  │  └─────┘  │
                  │  ┌─────┐  │  ┌──────────────┐
                  │  │ Cmd  │──│─>│  Process kill │
                  │  │ chan │  │  │  or signal    │
                  │  └─────┘  │  └──────────────┘
                  └───────────┘
```

### PTY Session Lifecycle

```
┌──────────┐    ┌──────────────┐    ┌───────────────────┐
│  Service │───>│  PtyManager  │───>│  nix::pty::       │
│          │    │  .alloc()    │    │  openpty()        │
└──────────┘    └──────┬───────┘    └───────────────────┘
                       │
                  ┌────▼─────┐
                  │  fork()  │
                  │          │
            ┌─────┤    ├───────┐
            │     └─────┘      │
            ▼                   ▼
     ┌────────────┐     ┌──────────────┐
     │  Parent    │     │  Child       │
     │            │     │              │
     │ close(slave)│    │ close(master)│
     │ keep(master)│    │ dup2(slave,  │
     │            │     │  stdin/out/  │
     │ read()/    │     │  err)       │
     │ write() to │     │              │
     │ master     │     │ exec(shell)  │
     └────────────┘     └──────────────┘
```

### Health Check Loop

```
┌───────────────────┐
│ HealthChecker.run │  (runs every `health_check_interval_secs`)
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ Iterate all       │
│ managed processes │
└────────┬──────────┘
         │
         ▼
┌───────────────────────────────────────────────────────┐
│ For each process:                                      │
│                                                         │
│  Status == Running? ──> kill(pid, 0) check             │
│    │                       │                            │
│    │                  ┌────┴────┐                       │
│    │               Alive    Dead                         │
│    │                  │       │                          │
│    │               skip    kill + reap                   │
│    │                                                      │
│  Status == Exited? ──> already cleaned → skip             │
│  Status == Error?  ──> log + emit event                   │
└───────────────────────────────────────────────────────┘
```

---

## Bridge to Other Modules

| Module | Bridge Type | What Flows | Direction |
|--------|-------------|------------|-----------|
| porpoise-core | Import | `ProcessId`, `RuntimeConfig`, `ProcessEvent`, `PorpoiseError` | core → runtime |
| porpoise-server | Direct call | Creates `ProcessManager`, `PtyManager`, `HealthChecker` | server → runtime |
| porpoise-terminal | Direct call | Uses `PtyManager` for terminal sessions | terminal → runtime |
| porpoise-agent | Direct call | Uses `ProcessManager` for agent spawn/monitor | agent → runtime |
| porpoise-relay | (Optional) | Output streaming via IPC | relay ↔ runtime |

---

## Platform Abstraction

| Operation | Unix (macOS/Linux) | Windows |
|-----------|-------------------|---------|
| PTY allocate | `nix::pty::openpty()` + `fork()` | `CreatePseudoConsole()` |
| PTY resize | `ioctl(TIOCSWINSZ)` | `ResizePseudoConsole()` |
| PTY read/write | Read/write master fd | Read/write pipe from `CONOUT` |
| Signal handling | `tokio::signal::unix` | `SetConsoleCtrlHandler` |
| Process kill | `SIGTERM` → `SIGKILL` | `TerminateProcess` |
| Health check | `kill(pid, 0)` | `OpenProcess` + `WaitForSingleObject` |

All platform differences are encapsulated behind `#[cfg(unix)]` / `#[cfg(windows)]` blocks inside `PtyManager` and `ProcessManager`. Service crates never see platform-specific code.

---

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| One tokio task per child process | Independent lifecycle, isolated failure, clean cancellation via drop |
| Watch channel for status | Consumers always see latest status (cache) + get notified on change |
| Mpsc channel for commands | Bounded backpressure; prevents runaway stdin writes from overflowing PTY buffer |
| RAII cleanup on task drop | Dropping the monitor task kills the child — no orphaned processes |
| `ioctl(TIOCSWINSZ)` for resize | Unix standard, works with all terminal emulators and PTY multiplexers |
| Separate HealthChecker task | Decouples monitoring from command dispatch; health issues never slow down commands |

---

*The runtime manages the most security-sensitive resources in Porpoise — child processes and PTY file descriptors. Every `unsafe` block must be documented with its safety invariants. Test PTY operations extensively on all target platforms.*
