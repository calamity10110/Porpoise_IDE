# Module Design: porpoise-server

> Background daemon — orchestrates all Porpoise services.

## Purpose
The server is the central process that owns all state, manages child processes, handles IPC requests from CLI/App, and coordinates all services (git, terminal, agent, SSH, browser, skills).

## Dependencies
- All service crates (db, relay, runtime, git, terminal, agent, ssh, browser, skills)
- porpoise-core (AppState, EventBus, config)

## Key Types
- `Daemon`: server lifecycle (new, start, run)
- `WorktreeService`: worktree CRUD + agent lifecycle
- `TerminalService`: PTY allocation + I/O
- `AgentService`: agent detection + management
- Router: maps IPC method strings to handler functions

## Bridges
- CLI connects via RelayClient over Unix socket
- All state changes publish to EventBus
- Database accessed exclusively through DbPool
