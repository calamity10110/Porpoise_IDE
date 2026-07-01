# Module Design: porpoise-git

> Git operations, worktree management, and remote provider integration.

---

## Purpose

`porpoise-git` provides all Git-related functionality: repository cloning, status/diff/log queries, branch management, and — most importantly — automated git worktree creation and management. It also integrates with remote Git providers (GitHub, GitLab, Gitea, Azure DevOps) for PR/issue browsing and creation.

**What problem it solves:** Git worktrees are the core abstraction of Porpoise — each agent runs in its own isolated worktree. Without a dedicated Git crate, every service would need to shell out to `git` commands and parse text output. This crate provides a type-safe, async-native, and tested Git API that all other crates rely on.

---

## Dependencies

### External

| Crate | Version | Why |
|-------|---------|-----|
| `git2` | 0.20+ | libgit2 bindings: clone, status, diff, log, branch, worktree |
| `octocrab` | 0.40+ | GitHub API client (PRs, issues, reviews) |
| `notify` | 7.x | File system watcher for git status change detection |
| `serde` / `serde_json` | 1.x | JSON serialization for API responses |
| `tracing` | 0.1 | Git operation logging |
| `tempfile` | 3.x | Temporary directories for clone verification |
| `uuid` | 1.x | Unique worktree names |

### Internal

| Crate | Dependency Type |
|-------|----------------|
| `porpoise-core` | Import — `PorpoiseError::Git*`, `GitConfig`, `GitEvent`, system events |
| `porpoise-network` | Optional — HTTP client for custom Git providers |

---

## Required Input

| Input | Source | Mechanism | Format |
|-------|--------|-----------|--------|
| Repository path | CLI / Server | `GitEngine::open()` / `GitEngine::clone()` | `PathBuf` |
| Worktree create params | Server worktree service | `WorktreeManager::create()` | `(repo, branch, path)` |
| Remote API tokens | `AppConfig` | Config loading | Auth headers / tokens |
| Watched directory paths | File watcher setup | `notify::Watcher::watch()` | `PathBuf` |
| Git provider URL | Config / CLI args | `RemoteProvider::connect()` | URL string |

---

## Required Output

| Output | Consumer | Mechanism | Format |
|--------|----------|-----------|--------|
| `Repository` handle | GitEngine callers | Return value from `open()`/`clone()` | `git2::Repository` wrapper |
| `StatusResult` | Server/CLI | `GitEngine::status()` | `{ branch, changes: Vec<DiffEntry> }` |
| `Vec<Commit>` | Server/CLI | `GitEngine::log()` | Commit structs with hash, message, author |
| Worktree path | Server worktree service | `WorktreeManager::create()` | `PathBuf` |
| File change events | EventBus | File watcher → `GitEvent::StatusChanged` | Typed event |
| PR/Issue data | CLI output | `RemoteProvider::list_prs()` | `Vec<PullRequest>` |
| Clone progress | Server / user | Callback via `git2::RemoteCallbacks` | Progress percentage |

---

## Ownership

| Owns | Does Not Own |
|------|-------------|
| `git2::Repository` handles | Worktree metadata records |
| File watcher instances | Process management |
| Remote provider API clients | PTY sessions |
| Worktree directory layout | Database |
| Git authentication (SSH keys, tokens) | Agent state |
| Clone/checkout operations | Config storage |

---

## Program Flow

### Worktree Creation Flow

```
User: porpoise worktree create --name fix-auth --agent codex

┌──────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  CLI     │───>│  Server      │───>│  GitEngine   │───>│  git2::      │
│  command │    │  Worktree    │    │  .clone()    │    │  Repository  │
│  parse   │    │  Service     │    │  or .open()  │    │              │
└──────────┘    └──────┬───────┘    └──────────────┘    └──────────────┘
                       │                                        │
                       ▼                                        ▼
                ┌──────────────┐                        ┌──────────────────┐
                │  Worktree    │                        │  git worktree    │
                │  Manager     │                        │  add <branch>    │
                │  .create()   │                        │  <path>          │
                └──────┬───────┘                        └──────────────────┘
                       │
                       ▼
                ┌──────────────┐    ┌──────────────┐
                │  EventBus    │───>│  UI/CLI      │
                │  .publish(   │    │  shows new   │
                │  Worktree-   │    │  worktree    │
                │  Created)    │    │              │
                └──────────────┘    └──────────────┘
```

### Git Status Monitoring Flow

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  notifys     │───>│  File        │───>│  GitEngine   │───>│  EventBus    │
│  watcher     │    │  Watcher    │    │  .status()   │    │  .publish(   │
│  detects     │    │  debounce   │    │              │    │  Status-     │
│  change      │    │  (500ms)    │    │              │    │  Changed)    │
└──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘
                                                                      │
                                                                      ▼
                                                               ┌──────────────┐
                                                               │  UI sidebar  │
                                                               │  refreshes   │
                                                               │  file status │
                                                               └──────────────┘
```

### Remote Provider Flow

```
CLI: porpoise git pr list --repo ./my-project

┌──────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  CLI     │───>│  Server      │───>│  GitHub      │───>│  octocrab    │
│  command │    │  git service │    │  Provider    │    │  API call    │
└──────────┘    └──────┬───────┘    └──────┬───────┘    └──────────────┘
                       │                   │
                       ▼                   ▼
                ┌──────────────┐    ┌──────────────────┐
                │  Format      │    │  Parse GitHub   │
                │  for output  │    │  response into  │
                │              │    │  our types      │
                └──────────────┘    └──────────────────┘
```

---

## Bridge to Other Modules

| Module | Bridge Type | What Flows | Direction |
|--------|-------------|------------|-----------|
| porpoise-core | Import | `GitConfig`, `GitEvent`, `PorpoiseError::Git*` | core → git |
| porpoise-server | Direct call | `GitEngine`, `WorktreeManager` used by server services | server ↔ git |
| porpoise-agent | Direct call | Agent spawn needs worktree path from git | agent → git |
| porpoise-runtime | Shared state | `WorktreeManager` passes cwd to `ProcessManager::spawn()` | git → runtime |
| porpoise-db | Indirect | Worktree metadata persisted by server | server → db → git |
| porpoise-network | Import | HTTP client for custom Git providers | git → network |
| porpoise-cli | Indirect via IPC | Git status/diff/log commands | cli → server → git |

---

## RemoteProvider Trait

```rust
#[async_trait]
pub trait RemoteProvider: Send + Sync {
    fn name(&self) -> &'static str;   // "github", "gitlab", etc.
    
    async fn list_prs(&self, repo: &str) -> Result<Vec<PullRequest>>;
    async fn get_pr(&self, repo: &str, number: u64) -> Result<PullRequest>;
    async fn create_pr(&self, repo: &str, pr: NewPr) -> Result<PullRequest>;
    async fn merge_pr(&self, repo: &str, number: u64, strategy: MergeStrategy) -> Result<()>;
    
    async fn list_issues(&self, repo: &str) -> Result<Vec<Issue>>;
    async fn create_issue(&self, repo: &str, issue: NewIssue) -> Result<Issue>;
    
    async fn list_checks(&self, repo: &str, ref_str: &str) -> Result<Vec<CheckRun>>;
}
```

Each provider (GitHub, GitLab, Gitea, Azure DevOps) implements this trait.

---

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| `git2` over shell `git` commands | Type-safe API, faster (no process spawn per call), cross-platform consistent output |
| `git worktree` over `git clone` | Shared object store = instant checkout, no duplicate data |
| Debounced file watcher | Prevents status refresh storm during bulk file operations (git stash, merge) |
| Provider trait for remote | Clean abstraction; adding a new Git provider doesn't change any business logic |
| WorktreeManager as single entry | Serializes all worktree operations → no race conditions on `.git/worktrees/` |
| Clone with callbacks | Enables progress reporting to the user during long clones |

---

*Git operations are I/O-bound and potentially long-running (clones). All git functions must be async and support cancellation via tokio. Use `Repository::open()` from a background thread if the callback requires it, as libgit2 is not fully async-safe.*
