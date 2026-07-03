# Module Design: porpoise-ssh

> SSH connections, remote sessions, and port forwarding.

## Purpose
Enables agents to run on remote machines over SSH. Manages connections, authentication, command execution, port forwarding, and TCP keepalive.

## Dependencies
- porpoise-core (types, events, errors)
- porpoise-network (optional HTTP client)
- ssh2 (libssh2 bindings)

## Key Types
- `SshManager`: connection pool, connect/disconnect/list
- `SshSession`: exec, set_keepalive, port_forward, forward_listen
- `AuthMethod`: Password, KeyFile, Agent

## Implemented Features
| Feature | Status | Details |
|---------|--------|---------|
| TCP connect + ssh2 handshake | ✅ | tokio::net::TcpStream → ssh2::Session |
| Authentication | ✅ | Password, KeyFile (with passphrase), Agent |
| Command exec | ✅ | ssh2::Channel with read loop, stderr merged |
| TCP keepalive | ✅ | `session.set_keepalive(want_reply, interval)` |
| Port forwarding (outbound) | ✅ | `channel_direct_tcpip(host, port)` |
| Port forwarding (inbound) | ✅ | `channel_forward_listen(port, host)` → Listener |
| SSH config parser | ✅ | `~/.ssh/config` → HostConfig struct |
| Auto-reconnect | ◐ | TCP keepalive enables dead connection detection |

## Bridges
- SSH sessions bridge to porpoise-runtime for PTY over SSH
- Remote worktrees bridge to porpoise-git for remote git operations
