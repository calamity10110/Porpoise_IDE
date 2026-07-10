# Graph Report - .  (2026-07-10)

## Corpus Check
- 16 files · ~3 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1279 nodes · 2329 edges · 82 communities (79 shown, 3 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 16|Community 16]]
- [[_COMMUNITY_Community 17|Community 17]]
- [[_COMMUNITY_Community 18|Community 18]]
- [[_COMMUNITY_Community 19|Community 19]]
- [[_COMMUNITY_Community 20|Community 20]]
- [[_COMMUNITY_Community 21|Community 21]]
- [[_COMMUNITY_Community 22|Community 22]]
- [[_COMMUNITY_Community 23|Community 23]]
- [[_COMMUNITY_Community 24|Community 24]]
- [[_COMMUNITY_Community 25|Community 25]]
- [[_COMMUNITY_Community 26|Community 26]]
- [[_COMMUNITY_Community 27|Community 27]]
- [[_COMMUNITY_Community 28|Community 28]]
- [[_COMMUNITY_Community 29|Community 29]]
- [[_COMMUNITY_Community 30|Community 30]]
- [[_COMMUNITY_Community 31|Community 31]]
- [[_COMMUNITY_Community 32|Community 32]]
- [[_COMMUNITY_Community 33|Community 33]]
- [[_COMMUNITY_Community 34|Community 34]]
- [[_COMMUNITY_Community 35|Community 35]]
- [[_COMMUNITY_Community 36|Community 36]]
- [[_COMMUNITY_Community 37|Community 37]]
- [[_COMMUNITY_Community 38|Community 38]]
- [[_COMMUNITY_Community 39|Community 39]]
- [[_COMMUNITY_Community 40|Community 40]]
- [[_COMMUNITY_Community 41|Community 41]]
- [[_COMMUNITY_Community 42|Community 42]]
- [[_COMMUNITY_Community 43|Community 43]]
- [[_COMMUNITY_Community 44|Community 44]]
- [[_COMMUNITY_Community 45|Community 45]]
- [[_COMMUNITY_Community 46|Community 46]]
- [[_COMMUNITY_Community 47|Community 47]]
- [[_COMMUNITY_Community 48|Community 48]]
- [[_COMMUNITY_Community 49|Community 49]]
- [[_COMMUNITY_Community 50|Community 50]]
- [[_COMMUNITY_Community 51|Community 51]]
- [[_COMMUNITY_Community 52|Community 52]]
- [[_COMMUNITY_Community 53|Community 53]]
- [[_COMMUNITY_Community 54|Community 54]]
- [[_COMMUNITY_Community 55|Community 55]]
- [[_COMMUNITY_Community 56|Community 56]]
- [[_COMMUNITY_Community 57|Community 57]]
- [[_COMMUNITY_Community 58|Community 58]]
- [[_COMMUNITY_Community 59|Community 59]]
- [[_COMMUNITY_Community 60|Community 60]]
- [[_COMMUNITY_Community 61|Community 61]]
- [[_COMMUNITY_Community 62|Community 62]]
- [[_COMMUNITY_Community 63|Community 63]]
- [[_COMMUNITY_Community 64|Community 64]]
- [[_COMMUNITY_Community 65|Community 65]]
- [[_COMMUNITY_Community 66|Community 66]]
- [[_COMMUNITY_Community 67|Community 67]]
- [[_COMMUNITY_Community 68|Community 68]]
- [[_COMMUNITY_Community 69|Community 69]]
- [[_COMMUNITY_Community 70|Community 70]]
- [[_COMMUNITY_Community 71|Community 71]]
- [[_COMMUNITY_Community 72|Community 72]]
- [[_COMMUNITY_Community 74|Community 74]]
- [[_COMMUNITY_Community 81|Community 81]]

## God Nodes (most connected - your core abstractions)
1. `DbPool` - 40 edges
2. `EventBus` - 26 edges
3. `AppState` - 23 edges
4. `Agent` - 21 edges
5. `ProcessManager` - 21 edges
6. `GitEngine` - 18 edges
7. `PtyManager` - 17 edges
8. `Daemon` - 17 edges
9. `PorpoiseError` - 15 edges
10. `SystemEvent` - 15 edges

## Surprising Connections (you probably didn't know these)
- `CodexAgent` --implements--> `Agent`  [EXTRACTED]
  crates/porpoise-agent/src/codex.rs → crates/porpoise-agent/src/traits.rs
- `CodexHandle` --implements--> `AgentHandle`  [EXTRACTED]
  crates/porpoise-agent/src/codex.rs → crates/porpoise-agent/src/traits.rs
- `GeminiAgent` --implements--> `Agent`  [EXTRACTED]
  crates/porpoise-agent/src/gemini.rs → crates/porpoise-agent/src/traits.rs
- `do_spawn()` --calls--> `Agent`  [EXTRACTED]
  crates/porpoise-agent/src/gemini.rs → crates/porpoise-agent/src/traits.rs
- `do_spawn()` --references--> `AgentHandle`  [EXTRACTED]
  crates/porpoise-agent/src/gemini.rs → crates/porpoise-agent/src/traits.rs

## Import Cycles
- 1-file cycle: `crates/porpoise-core/src/error.rs -> crates/porpoise-core/src/error.rs`

## Communities (82 total, 3 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.05
Nodes (40): AgentRow, Option, Result, Self, String, Vec, ConfigEntry, Option (+32 more)

### Community 1 - "Community 1"
Cohesion: 0.07
Nodes (35): HealthChecker, Arc, Duration, Result, Self, ProcessHandle, DateTime, ProcessId (+27 more)

### Community 2 - "Community 2"
Cohesion: 0.06
Nodes (30): Channel, AuthMethod, Debug, Formatter, Option, Result, String, HostConfig (+22 more)

### Community 3 - "Community 3"
Cohesion: 0.06
Nodes (31): Connection, AppConfig, home_dir(), PathBuf, Result, PorpoiseError, Display, PathBuf (+23 more)

### Community 4 - "Community 4"
Cohesion: 0.09
Nodes (30): from_octo_issue(), from_octo_pr(), GitHubProvider, pr_state(), Option, Result, Self, Vec (+22 more)

### Community 5 - "Community 5"
Cohesion: 0.07
Nodes (25): AgentDetector, Option, String, Vec, test_detect_all(), generate_completions(), Error, Path (+17 more)

### Community 6 - "Community 6"
Cohesion: 0.12
Nodes (22): alloc_pty_impl(), pty_read_impl(), pty_resize_impl(), pty_write_impl(), PtyManager, PtySession, Arc, HashMap (+14 more)

### Community 7 - "Community 7"
Cohesion: 0.17
Nodes (23): Arc, ComputerActionType, resolve_template(), Result, Self, String, Value, Vec (+15 more)

### Community 8 - "Community 8"
Cohesion: 0.14
Nodes (22): AgentState, AppStateInner, AgentId, AppConfig, Arc, DateTime, HashMap, Option (+14 more)

### Community 9 - "Community 9"
Cohesion: 0.11
Nodes (15): Option, Path, PathBuf, Result, Self, Vec, ScrollbackPersister, Self (+7 more)

### Community 10 - "Community 10"
Cohesion: 0.11
Nodes (16): AtomicBool, ConnectivityMonitor, Arc, Default, Duration, Self, String, Vec (+8 more)

### Community 11 - "Community 11"
Cohesion: 0.21
Nodes (15): CredentialEntry, CredentialStore, CredentialVault, Result, Self, String, Vec, test_delete() (+7 more)

### Community 12 - "Community 12"
Cohesion: 0.16
Nodes (12): GitEngine, Option, Path, PathBuf, Result, Self, String, Vec (+4 more)

### Community 13 - "Community 13"
Cohesion: 0.16
Nodes (18): AgentConfig, AppConfig, BrowserConfig, CliConfig, ColorChoice, CoreConfig, DbConfig, GitConfig (+10 more)

### Community 14 - "Community 14"
Cohesion: 0.14
Nodes (13): ClaudeCodeAgent, ClaudeHandle, Box, Child, Default, Option, Result, Self (+5 more)

### Community 15 - "Community 15"
Cohesion: 0.14
Nodes (11): do_spawn(), GeminiAgent, GeminiHandle, Box, Child, Default, Option, Path (+3 more)

### Community 16 - "Community 16"
Cohesion: 0.14
Nodes (10): CodexAgent, CodexHandle, Box, Child, Default, Option, Path, Result (+2 more)

### Community 17 - "Community 17"
Cohesion: 0.15
Nodes (10): BrowserEngine, HeadlessBrowser, Result, Send, String, Sync, Vec, NavigationResult (+2 more)

### Community 18 - "Community 18"
Cohesion: 0.19
Nodes (21): AgentEvent, AgentStatusKind, BrowserEvent, GitEvent, NotificationSeverity, OutputKind, AgentId, Option (+13 more)

### Community 19 - "Community 19"
Cohesion: 0.23
Nodes (14): CorrelationId, ErrorCode, Handshake, ProtocolError, Request, Response, Default, Into (+6 more)

### Community 20 - "Community 20"
Cohesion: 0.14
Nodes (10): GenericAgent, GenericHandle, Box, Child, Into, Option, Path, Result (+2 more)

### Community 21 - "Community 21"
Cohesion: 0.21
Nodes (10): Client, HttpClient, F, Proxy, Result, Self, String, T (+2 more)

### Community 22 - "Community 22"
Cohesion: 0.14
Nodes (10): Path, RemoteProvider, Send, Sync, ColorScheme, Default, Self, String (+2 more)

### Community 23 - "Community 23"
Cohesion: 0.20
Nodes (10): HookServer, Self, EventBus, Into, Receiver, Self, Sender, test_event_bus_clone() (+2 more)

### Community 24 - "Community 24"
Cohesion: 0.18
Nodes (9): HashMap, Option, Self, TerminalId, Vec, TerminalLayout, SessionId, TerminalId (+1 more)

### Community 25 - "Community 25"
Cohesion: 0.19
Nodes (14): Vec, AgentInfo, AgentKind, AgentManifest, AgentOutput, AgentStatus, OutputType, AgentId (+6 more)

### Community 26 - "Community 26"
Cohesion: 0.21
Nodes (8): Daemon, AppConfig, DateTime, Option, PathBuf, Result, Self, Utc

### Community 27 - "Community 27"
Cohesion: 0.18
Nodes (8): Default, HashMap, Option, Self, String, Vec, SkillManifest, SkillRegistry

### Community 28 - "Community 28"
Cohesion: 0.12
Nodes (15): MaterialPageRoute, PorpoiseApp, AgentsScreen, build, _iconFor, _showSpawnDialog, _statusDot, build (+7 more)

### Community 29 - "Community 29"
Cohesion: 0.23
Nodes (11): AgentEntry, AgentPool, AgentId, Arc, Box, HashMap, Option, Path (+3 more)

### Community 30 - "Community 30"
Cohesion: 0.20
Nodes (12): AppState, handle_get(), Result, Value, handle_create(), Result, Value, handle_create() (+4 more)

### Community 31 - "Community 31"
Cohesion: 0.19
Nodes (11): CompiledModule, Result, Self, Vec, WasmInstance, WasmRuntime, Engine, Instance (+3 more)

### Community 32 - "Community 32"
Cohesion: 0.25
Nodes (12): CliSettings, Default, OutputFormat, PathBuf, Self, discover_config_path(), home_dir(), load_config() (+4 more)

### Community 33 - "Community 33"
Cohesion: 0.26
Nodes (8): AutomationAction, ClickButton, ComputerUse, Result, Self, String, Value, Enigo

### Community 34 - "Community 34"
Cohesion: 0.21
Nodes (8): NamedPipeListener, NamedPipeTransport, F, Path, Result, Self, String, NamedPipeClient

### Community 35 - "Community 35"
Cohesion: 0.15
Nodes (13): build, createState, HomeScreen, _HomeScreenState, main, _screens, _selectedIndex, screens/agents_screen.dart (+5 more)

### Community 36 - "Community 36"
Cohesion: 0.15
Nodes (12): bool get, dart:async, dart:convert, _authToken, call, _channel, connect, disconnect (+4 more)

### Community 37 - "Community 37"
Cohesion: 0.28
Nodes (7): OutputParser, ParsedOutput, Option, String, Vec, test_parse_newline(), test_parse_plain_text()

### Community 38 - "Community 38"
Cohesion: 0.33
Nodes (12): from_bincode(), from_json(), Result, String, T, Vec, test_bincode_roundtrip(), test_json_pretty_has_newlines() (+4 more)

### Community 39 - "Community 39"
Cohesion: 0.19
Nodes (8): Router, HashMap, Self, String, register_all(), DateTime, Utc, HandlerFn

### Community 40 - "Community 40"
Cohesion: 0.23
Nodes (6): RateLimitConfig, RateLimiter, Default, Self, test_rate_limiter_consume(), Instant

### Community 41 - "Community 41"
Cohesion: 0.33
Nodes (9): read_frame(), RelayServer, Arc, PathBuf, Result, Self, UnixStream, write_frame() (+1 more)

### Community 42 - "Community 42"
Cohesion: 0.20
Nodes (9): ChangeNotifier, AgentProvider, TerminalProvider, WorktreeProvider, build, build, ../models/terminal.dart, ../models/worktree.dart (+1 more)

### Community 43 - "Community 43"
Cohesion: 0.22
Nodes (7): Platform, Display, Formatter, Result, Self, test_current_platform_is_unix(), test_display()

### Community 44 - "Community 44"
Cohesion: 0.42
Nodes (10): Capabilities, Capability, CapabilityScope, HostPattern, ProcessPattern, Option, PathBuf, String (+2 more)

### Community 45 - "Community 45"
Cohesion: 0.18
Nodes (10): Agent, _agents, appendOutput, fromJson, id, kind, _lastOutput, status (+2 more)

### Community 46 - "Community 46"
Cohesion: 0.18
Nodes (10): appendOutput, cols, fromJson, id, output, rows, sessionId, Terminal (+2 more)

### Community 47 - "Community 47"
Cohesion: 0.24
Nodes (10): build, createState, _hostController, _pair, PairScreen, _PairScreenState, _portController, WsService (+2 more)

### Community 48 - "Community 48"
Cohesion: 0.22
Nodes (7): ProxyConfig, Default, Option, Proxy, Self, String, Vec

### Community 49 - "Community 49"
Cohesion: 0.38
Nodes (6): RelayClient, Mutex, PathBuf, Result, Self, Value

### Community 50 - "Community 50"
Cohesion: 0.38
Nodes (6): Frame, Result, Self, Vec, test_roundtrip(), FrameFlags

### Community 51 - "Community 51"
Cohesion: 0.33
Nodes (7): Cli, DaemonAction, DaemonArgs, handle(), OutputFormat, Result, String

### Community 52 - "Community 52"
Cohesion: 0.31
Nodes (5): Path, Result, Self, UnixStream, UnixSocketTransport

### Community 53 - "Community 53"
Cohesion: 0.22
Nodes (8): List, branch, fromJson, name, path, update, Worktree, _worktrees

### Community 54 - "Community 54"
Cohesion: 0.36
Nodes (7): AgentAction, AgentArgs, Option, handle(), OutputFormat, Result, String

### Community 55 - "Community 55"
Cohesion: 0.36
Nodes (7): BrowserAction, BrowserArgs, String, handle(), OutputFormat, Result, String

### Community 56 - "Community 56"
Cohesion: 0.50
Nodes (7): Commands, handle_command(), page_output(), OutputFormat, Result, String, should_page()

### Community 57 - "Community 57"
Cohesion: 0.36
Nodes (6): TerminalAction, TerminalArgs, handle(), OutputFormat, Result, String

### Community 59 - "Community 59"
Cohesion: 0.43
Nodes (6): ConfigAction, ConfigArgs, handle(), OutputFormat, Result, String

### Community 60 - "Community 60"
Cohesion: 0.43
Nodes (6): GitAction, GitArgs, handle(), OutputFormat, Result, String

### Community 61 - "Community 61"
Cohesion: 0.43
Nodes (6): SkillAction, SkillArgs, handle(), OutputFormat, Result, String

### Community 62 - "Community 62"
Cohesion: 0.43
Nodes (6): SshAction, SshArgs, handle(), OutputFormat, Result, String

### Community 63 - "Community 63"
Cohesion: 0.43
Nodes (6): WorktreeAction, WorktreeArgs, handle(), OutputFormat, Result, String

### Community 64 - "Community 64"
Cohesion: 0.57
Nodes (4): OutputFormat, plain_line(), String, Value

### Community 65 - "Community 65"
Cohesion: 0.40
Nodes (5): handle_health(), DateTime, Result, Utc, Value

### Community 66 - "Community 66"
Cohesion: 0.40
Nodes (4): Create, Delete, Read, Update

### Community 67 - "Community 67"
Cohesion: 0.70
Nodes (4): handle_detect(), handle_list(), Result, Value

### Community 68 - "Community 68"
Cohesion: 0.70
Nodes (4): handle_clone(), handle_status(), Result, Value

### Community 69 - "Community 69"
Cohesion: 0.50
Nodes (3): EventHandler, Send, Sync

### Community 70 - "Community 70"
Cohesion: 0.67
Nodes (3): handle_open(), Result, Value

### Community 71 - "Community 71"
Cohesion: 0.67
Nodes (3): handle_list(), Result, Value

### Community 72 - "Community 72"
Cohesion: 0.67
Nodes (3): handle_connect(), Result, Value

## Knowledge Gaps
- **54 isolated node(s):** `OutputFormat`, `Create`, `Read`, `Update`, `Delete` (+49 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **3 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `PorpoiseError` connect `Community 3` to `Community 0`, `Community 5`?**
  _High betweenness centrality (0.148) - this node is a cross-community bridge._
- **Why does `DbPool` connect `Community 0` to `Community 26`, `Community 3`?**
  _High betweenness centrality (0.065) - this node is a cross-community bridge._
- **What connects `OutputFormat`, `Create`, `Read` to the rest of the system?**
  _54 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.0532724505327245 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.06745098039215686 - nodes in this community are weakly interconnected._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.06382978723404255 - nodes in this community are weakly interconnected._
- **Should `Community 3` be split into smaller, more focused modules?**
  _Cohesion score 0.05939716312056738 - nodes in this community are weakly interconnected._