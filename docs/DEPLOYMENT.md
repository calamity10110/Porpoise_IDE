# Porpoise Deployment Guide

> Self-hosted vs Cloud deployment options, architecture, and operations guide.

---

## Deployment Models

### Model 1: Single-User Desktop (Self-Hosted)

```
┌──────────────────────────────────────────┐
│                Laptop/PC                  │
│                                           │
│  ┌──────────┐  ┌──────────────────────┐  │
│  │ Desktop  │  │    porpoise-server   │  │
│  │   App    │←→│    (daemon)          │  │
│  │ (Tauri)  │  │                      │  │
│  └──────────┘  │  ┌────────────────┐  │  │
│                │  │  Agent Pool    │  │  │
│  ┌──────────┐  │  │  • Claude Code │  │  │
│  │   CLI    │  │  │  • Codex       │  │  │
│  │ (clap)   │  │  │  • Gemini      │  │  │
│  └──────────┘  │  └────────────────┘  │  │
│                │  ┌────────────────┐  │  │
│                │  │  SQLite DB     │  │  │
│                │  │  + Credential  │  │  │
│                │  │    Vault       │  │  │
│                │  └────────────────┘  │  │
│                └──────────────────────┘  │
└──────────────────────────────────────────┘
```

**Who**: Individual developer.
**Setup**: `cargo build --release -p porpoise-app` → run installer.
**Cost**: Free (MIT license).
**Data**: All local. Nothing leaves your machine.

---

### Model 2: Team Server (Self-Hosted)

```
┌──────────────────────────────────────────────────────┐
│                   Build Server / VPS                   │
│                                                       │
│  ┌─────────────────────────────────────────────────┐  │
│  │              porpoise-server (daemon)            │  │
│  │                                                  │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐        │  │
│  │  │ Agent 1  │ │ Agent 2  │ │ Agent 3  │  ...   │  │
│  │  │ (Claude) │ │ (Codex)  │ │ (Gemini) │        │  │
│  │  └──────────┘ └──────────┘ └──────────┘        │  │
│  │                                                  │  │
│  │  ┌──────────────────┐  ┌────────────────────┐  │  │
│  │  │  SQLite + Vault  │  │  WSS Server :9876  │  │  │
│  │  └──────────────────┘  └─────────┬──────────┘  │  │
│  └──────────────────────────────────┼──────────────┘  │
└──────────────────────────────────────┼──────────────────┘
                                       │ WSS (TLS 1.3)
                    ┌──────────────────┼──────────────┐
                    │                  │              │
              ┌─────▼─────┐    ┌──────▼──────┐  ┌────▼─────┐
              │ Developer │    │  Developer  │  │ Mobile   │
              │   CLI     │    │   Desktop   │  │ (Flutter)│
              │ (via SSH) │    │   App       │  │          │
              └───────────┘    └─────────────┘  └──────────┘
```

**Who**: Small team (2-10 developers) sharing a build server.
**Setup**:
```bash
# On the server
git clone https://github.com/porpoise-ide/porpoise.git
cd porpoise && cargo build --release -p porpoise-server
PORPOISE_WS_PORT=9876 PORPOISE_WS_TLS=1 ./target/release/porpoise-server

# Each developer connects via:
# CLI:   ssh -L /tmp/porpoise.sock:/tmp/porpoise.sock user@server
# Desktop: Point to server IP
# Mobile: Scan QR code
```
**Cost**: Server cost only. Porpoise is free.
**Data**: All on the server. Developers access via IPC/WS.

---

### Model 3: Docker / Kubernetes (Self-Hosted)

```yaml
# docker-compose.yml
version: '3.8'
services:
  porpoise:
    build: .
    ports:
      - "9876:9876"
    volumes:
      - porpoise-data:/root/.local/share/porpoise
      - repos:/repos
    environment:
      - PORPOISE_WS_PORT=9876
      - PORPOISE_WS_TLS=1
      - RUST_LOG=info
    restart: unless-stopped

volumes:
  porpoise-data:
  repos:
```

**Kubernetes manifest** (future):
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: porpoise-server
spec:
  replicas: 1
  selector:
    matchLabels:
      app: porpoise
  template:
    spec:
      containers:
        - name: porpoise
          image: porpoise/server:latest
          ports:
            - containerPort: 9876
          env:
            - name: PORPOISE_WS_TLS
              value: "1"
```

**Who**: Teams wanting reproducible deployments.
**Cost**: Container hosting.
**Data**: In persistent volume.

---

### Model 4: Porpoise Cloud (Future — Roadmap)

```
┌─────────────────────────────────────────────────────┐
│                  Porpoise Cloud                      │
│                                                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐          │
│  │ Tenant A │  │ Tenant B │  │ Tenant C │  ...     │
│  │ (isolated)│  │(isolated)│  │(isolated)│          │
│  └─────┬────┘  └─────┬────┘  └─────┬────┘          │
│        │             │             │                 │
│  ┌─────▼─────────────▼─────────────▼─────┐          │
│  │         Shared Agent Pool              │          │
│  │  Claude │ Codex │ Gemini │ OpenCode   │          │
│  └────────────────────────────────────────┘          │
│                                                      │
│  ┌────────────────────────────────────────┐          │
│  │  Object Storage (worktrees, builds)    │          │
│  └────────────────────────────────────────┘          │
└──────────────────┬──────────────────────────────────┘
                   │ WSS (TLS 1.3)
    ┌──────────────┼──────────────┐
    │              │              │
┌───▼───┐   ┌─────▼────┐  ┌─────▼─────┐
│Browser│   │ Mobile   │  │   CLI     │
│  App  │   │  App     │  │ (tunnel)  │
└───────┘   └──────────┘  └───────────┘
```

**Who**: Teams who don't want to self-host.
**Status**: **Not yet available.** Roadmap item post-v1.

---

## Feature Comparison: Self-Hosted vs Cloud

| Feature | Self-Hosted | Cloud (Future) |
|---------|------------|----------------|
| **Setup** | Manual (10 min) | Instant (sign up) |
| **Cost** | Free + server | Subscription |
| **Data residency** | Your infrastructure | Managed |
| **Agent API keys** | Your keys, your machine | Managed secrets |
| **Max agents** | Unlimited (hardware-limited) | Tier-based |
| **Worktree storage** | Local disk | Cloud storage |
| **Mobile access** | Requires open port / VPN | Built-in |
| **Updates** | Manual or CI | Automatic |
| **SSO/SAML** | Not available | Enterprise tier |
| **Audit logs** | Local SQLite | Centralized |
| **SLA** | None | Enterprise tier |
| **Custom plugins** | Full WASM SDK | Reviewed marketplace |

---

## Security Considerations by Deployment

### Self-Hosted (Single User)

| Concern | Mitigation |
|---------|-----------|
| IPC token theft | File permissions 0o600 (Unix), ACL-restricted (Windows) |
| Credential vault | AES-256-GCM, PBKDF2-600K, file-locked |
| Network eavesdropping | TLS 1.3 on all connections (rustls) |
| WASM plugin escape | Zero imports by default, fuel-metered, BLAKE3 integrity |

### Self-Hosted (Team Server)

| Concern | Mitigation |
|---------|-----------|
| Shared daemon access | Session token auth required |
| Network exposure | WSS with self-signed cert + fingerprint verification (TOFU) |
| Agent credential leakage | SSH passwords zeroized on Drop |
| Multi-user isolation | Each user gets own session token (future: per-user auth) |

### Docker

| Concern | Mitigation |
|---------|-----------|
| Container escape | Run as non-root, read-only filesystem |
| Volume encryption | Use encrypted volumes (LUKS, EBS encryption) |
| Network exposure | Expose only port 9876, use reverse proxy with TLS |

---

## Operations Runbook

### Backup

```bash
# Back up data directory (contains DB, credentials, certs)
tar czf porpoise-backup-$(date +%Y%m%d).tar.gz ~/.local/share/porpoise/

# For Docker
docker exec porpoise tar czf - /root/.local/share/porpoise/ > backup.tar.gz
```

### Restore

```bash
# Stop daemon
porpoise daemon stop

# Restore
tar xzf porpoise-backup-20260720.tar.gz -C /

# Restart
porpoise-server
```

### Upgrade

```bash
# Self-hosted: pull and rebuild
cd porpoise
git pull
cargo build --release -p porpoise-server
# Restart daemon

# Desktop: auto-update via tauri-plugin-updater (if configured)
# Or download new installer from Releases
```

### Monitoring

```bash
# Health check
porpoise --json health
# {"uptime": 3600, "agents": 3, "status": "healthy"}

# Log tail
tail -f ~/.local/share/porpoise/logs/porpoise.log

# Process check
ps aux | grep porpoise-server
```

### Disaster Recovery

| Scenario | Recovery |
|----------|----------|
| Daemon crash | `generation_id` marks stale sessions; restart daemon |
| DB corruption | Restore from backup; migrations are idempotent |
| Cert expired | Delete `daemon.cert.pem` + `daemon.key.pem`; daemon regenerates |
| Token compromised | Delete `ipc-token`; daemon generates new one on restart |
