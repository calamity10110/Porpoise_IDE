# Porpoise Marketing Package

> Internal marketing reference — elevator pitches, competitive analysis, target personas, and positioning.

---

## Elevator Pitches

### 30-Second Pitch

> Porpoise is a Rust-native AI coding agent orchestrator. Run Claude Code, Codex, Gemini, or any CLI agent in parallel git worktrees, each isolated with its own terminal, browser panel, and SSH tunnel. Monitor from your desktop, phone, or browser extension. Memory-safe, 15MB binary, zero Electron bloat.

### 10-Second Pitch

> Orca rebuilt in Rust. Parallel AI agents, isolated worktrees, terminal splits, mobile companion, browser automation — in a 15MB binary.

### One-Liner Taglines

- **"The AI Orchestrator for 100x builders."**
- "Memory-safe. Lightning-fast. Your agents, orchestrated."
- "Run five agents in five worktrees. Track them all in one place."
- "Electron-free AI development, written in Rust."

---

## Competitive Analysis

| Feature | Porpoise | Orca | Cursor | Aider | Goose | Devin |
|---------|----------|------|--------|-------|-------|-------|
| **Language** | Rust | TypeScript | TypeScript | Python | Python | TypeScript |
| **Memory safety** | Compile-time | GC | GC | GC | GC | GC |
| **Binary size** | ~15MB | ~300MB | ~200MB | ~50MB | ~50MB | Cloud |
| **Parallel worktrees** | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ |
| **Terminal splits** | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Embedded browser** | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ |
| **SSH remoting** | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ |
| **Mobile companion** | ✅ (Flutter) | ✅ | ❌ | ❌ | ❌ | ✅ |
| **Chrome extension** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **WASM plugins** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Workflow automation** | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| **Credential vault** | ✅ (AES-256) | ❌ | ❌ | ❌ | ❌ | ❌ |
| **TLS everywhere** | ✅ (rustls) | ❌ | ❌ | N/A | N/A | N/A |
| **Auto-update** | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ |
| **Self-hosted** | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ |
| **Open source** | ✅ (MIT) | ✅ (MIT) | ❌ | ✅ | ✅ | ❌ |
| **Agent count** | 25+ | 25+ | 1 (built-in) | 1 | 1 | 1 |

### Key Differentiators

1. **Rust-native** — no Electron, no V8, no garbage collector. 15MB binary vs 300MB.
2. **Multi-agent parallelism** — fan one task across 5 agents in isolated worktrees.
3. **Browser bridge** — Chrome extension sends DOM + screenshots directly to agents.
4. **WASM plugin sandbox** — extend Porpoise without compromising safety.
5. **Mobile companion** — WSS-encrypted, QR-paired, monitor agents from anywhere.
6. **Enterprise-ready** — AES-256 credential vault, PBKDF2-600K, zeroize, capability-based WASM.

---

## Target Personas

### 1. "The Power User" — Senior Backend Engineer

- **Profile**: Runs 3+ AI agents simultaneously, frustrated by terminal sprawl
- **Pain**: Managing multiple agent sessions across worktrees is manual and error-prone
- **Solution**: Porpoise's parallel worktree view + terminal splits + agent pool
- **Quote**: "I want Claude Code and Codex running side-by-side on the same bug, and I want to see both outputs without alt-tabbing."

### 2. "The Team Lead" — Engineering Manager

- **Profile**: Needs visibility into what agents are doing across the team
- **Pain**: No centralized view of agent activity, no audit trail
- **Solution**: Desktop dashboard + mobile companion + notification history
- **Quote**: "I need to know which agents finished, which failed, and what they produced — from my phone."

### 3. "The Security-Conscious Org" — DevSecOps Engineer

- **Profile**: Can't use cloud AI tools due to data residency requirements
- **Pain**: All AI coding tools are cloud-hosted or Electron-based (attack surface)
- **Solution**: Self-hosted Porpoise with TLS, encrypted credentials, WASM sandbox
- **Quote**: "We need AI coding tools that run on our infrastructure, with our keys, behind our firewall."

### 4. "The Plugin Developer" — Open Source Contributor

- **Profile**: Wants to build custom tools on top of an AI orchestration platform
- **Pain**: No safe plugin system in existing tools (Electron = full system access)
- **Solution**: WASM-sandboxed plugin SDK with capability-based permissions
- **Quote**: "I want to write a linter plugin that runs in isolation and can't escape the sandbox."

---

## Use Case Scenarios

### Scenario 1: Parallel Bug Fix Race

> "I have a critical auth bug. I'll spawn Claude Code, Codex, and Gemini — each in its own worktree — and see who fixes it first. Merge the winner."

```bash
porpoise worktree create --name fix-auth-claude --agent claude --prompt "Fix the auth bug in src/auth/"
porpoise worktree create --name fix-auth-codex --agent codex --prompt "Fix the auth bug in src/auth/"
porpoise worktree create --name fix-auth-gemini --agent gemini --prompt "Fix the auth bug in src/auth/"
```

### Scenario 2: Remote Build Server

> "My CI builds take 20 minutes locally but 2 minutes on the build server. I'll SSH-tunnel an agent to the remote machine and run the build there."

```bash
porpoise ssh connect --host build-server --user ci
porpoise ssh worktree --session-id ssh-1 --repo ./my-project
```

### Scenario 3: Browser-Driven Development

> "I'm building a web app. I'll use the Chrome extension to click any element, send its HTML + CSS to Claude Code, and get a fix back — without copy-pasting."

1. Open Chrome extension → pair with daemon
2. Navigate to `localhost:3000`
3. Right-click broken element → "Send to Porpoise agent"
4. Agent receives DOM snapshot + CSS → generates fix

### Scenario 4: Mobile Monitoring

> "I kicked off a long-running agent before leaving the office. I'll check progress from my phone."

1. Desktop: `PORPOISE_WS_PORT=9876 PORPOISE_WS_TLS=1 porpoise-server`
2. Desktop: `porpoise mobile qr` → scan QR with phone
3. Phone: Open Porpoise Mobile → see agent status, output, completion notifications

### Scenario 5: Automated Workflow

> "Every night, I want to: pull latest, run tests, if tests pass → deploy, if fail → send agent to investigate."

```yaml
id: nightly-ci
steps:
  - type: AgentCall
    id: pull-and-test
    agent_kind: opencode
    prompt: "git pull && cargo test"
  - type: Condition
    depends_on: [pull-and-test]
    if: "${pull-and-test.exit_code} == 0"
    then: deploy
    else: investigate
```

---

## Pricing Model

### Self-Hosted (Free, MIT License)

| Feature | Included |
|---------|----------|
| Full desktop app | ✅ |
| CLI + daemon | ✅ |
| All 25+ agents | ✅ |
| Mobile companion | ✅ |
| Chrome extension | ✅ |
| WASM plugin SDK | ✅ |
| Workflow automation | ✅ |
| Credential vault | ✅ |
| Community support | GitHub Issues |

### Porpoise Cloud (Future — Not Yet Available)

| Tier | Price | Features |
|------|-------|----------|
| **Free** | $0 | 1 agent, 3 worktrees, community support |
| **Pro** | $20/mo | 5 agents, 20 worktrees, mobile companion, priority support |
| **Team** | $50/seat/mo | Unlimited agents, shared workspaces, audit logs, SSO |
| **Enterprise** | Custom | Self-hosted + cloud, SLA, dedicated support, custom plugins |

> **Note**: Porpoise Cloud is a future roadmap item. The current release is self-hosted only.

---

## Messaging Guidelines

### Do Say
- "Rust-native" (not "written in Rust")
- "Memory-safe by construction" (not "no memory leaks")
- "15MB binary" (specific number, not "small")
- "Parallel agent worktrees" (not "multiple agents")
- "WASM-sandboxed plugins" (not "safe plugins")

### Don't Say
- "Better than Electron" (avoid direct attacks)
- "AI-powered" (meaningless buzzword)
- "Next-generation" (vague)
- "Revolutionary" (overused)
- "Blockchain" (we don't have one)

### Tone
- Technical but accessible
- Confident without arrogance
- Show, don't tell (use benchmarks, not adjectives)
- Developer-to-developer communication
