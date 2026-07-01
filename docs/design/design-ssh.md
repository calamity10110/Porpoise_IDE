# Module Design: porpoise-ssh

> SSH connections, remote sessions, and port forwarding.

## Purpose
Enables agents to run on remote machines over SSH. Manages connections, authentication, interactive shells, file transfer, and port forwarding.

## Dependencies
- porpoise-core (types, events)
- porpoise-network (optional HTTP client)

## Key Types
- `SshManager`: connection pool, connect/disconnect/list
- `SshSession`: exec, shell, port_forward, file_read, file_write
- `AuthMethod`: KeyAuth, PasswordAuth, AgentAuth

## Bridges
- SSH sessions bridge to porpoise-runtime for PTY over SSH
- Remote worktrees bridge to porpoise-git for remote git operations
