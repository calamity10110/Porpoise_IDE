# Graph Report - C:/Users/vuanh/Downloads/porpoises/Porpoise_ide  (2026-09-13)

## Corpus Check
- Large corpus: 280 files · ~122,047 words. Semantic extraction will be expensive (many Claude tokens). Consider running on a subfolder, or use --no-semantic to run AST-only.

## Summary
- 1903 nodes · 4140 edges · 67 communities detected
- Extraction: 56% EXTRACTED · 44% INFERRED · 0% AMBIGUOUS · INFERRED: 1835 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_ESP32 Comms|ESP32 Comms]]
- [[_COMMUNITY_App Core|App Core]]
- [[_COMMUNITY_Agent Adapters|Agent Adapters]]
- [[_COMMUNITY_Adapter Registry|Adapter Registry]]
- [[_COMMUNITY_Browser & Skills|Browser & Skills]]
- [[_COMMUNITY_Mobile Shell|Mobile Shell]]
- [[_COMMUNITY_Agent Models|Agent Models]]
- [[_COMMUNITY_ESP32 WiFi|ESP32 WiFi]]
- [[_COMMUNITY_Process Pools|Process Pools]]
- [[_COMMUNITY_Notifications|Notifications]]
- [[_COMMUNITY_HTTP Server|HTTP Server]]
- [[_COMMUNITY_Design Canvas|Design Canvas]]
- [[_COMMUNITY_CLI Commands|CLI Commands]]
- [[_COMMUNITY_Behavior Orchestration|Behavior Orchestration]]
- [[_COMMUNITY_Extension Background|Extension Background]]
- [[_COMMUNITY_Runtime|Runtime]]
- [[_COMMUNITY_Agent|Agent]]
- [[_COMMUNITY_Core|Core]]
- [[_COMMUNITY_Cli|Cli]]
- [[_COMMUNITY_Core|Core]]
- [[_COMMUNITY_Skills|Skills]]
- [[_COMMUNITY_Extension|Extension]]
- [[_COMMUNITY_Docs|Docs]]
- [[_COMMUNITY_Runtime|Runtime]]
- [[_COMMUNITY_Templates|Templates]]
- [[_COMMUNITY_Templates|Templates]]
- [[_COMMUNITY_Mobile|Mobile]]
- [[_COMMUNITY_Templates|Templates]]
- [[_COMMUNITY_Docs|Docs]]
- [[_COMMUNITY_App|App]]
- [[_COMMUNITY_Core|Core]]
- [[_COMMUNITY_Git|Git]]
- [[_COMMUNITY_Design|Design]]
- [[_COMMUNITY_Templates|Templates]]
- [[_COMMUNITY_Templates|Templates]]
- [[_COMMUNITY_Templates|Templates]]
- [[_COMMUNITY_Agent|Agent]]
- [[_COMMUNITY_Design|Design]]
- [[_COMMUNITY_Core|Core]]
- [[_COMMUNITY_Network|Network]]
- [[_COMMUNITY_Skills|Skills]]
- [[_COMMUNITY_Templates|Templates]]
- [[_COMMUNITY_Agent|Agent]]
- [[_COMMUNITY_Core|Core]]
- [[_COMMUNITY_Core|Core]]
- [[_COMMUNITY_Core|Core]]
- [[_COMMUNITY_Terminal|Terminal]]
- [[_COMMUNITY_Templates|Templates]]
- [[_COMMUNITY_Opencode|Opencode]]
- [[_COMMUNITY_Templates|Templates]]
- [[_COMMUNITY_Templates|Templates]]
- [[_COMMUNITY_Db|Db]]
- [[_COMMUNITY_Runtime|Runtime]]
- [[_COMMUNITY_Templates|Templates]]
- [[_COMMUNITY_Templates|Templates]]
- [[_COMMUNITY_Templates|Templates]]
- [[_COMMUNITY_Agent|Agent]]
- [[_COMMUNITY_Browser|Browser]]
- [[_COMMUNITY_Templates|Templates]]
- [[_COMMUNITY_Templates|Templates]]
- [[_COMMUNITY_Docs|Docs]]
- [[_COMMUNITY_Extension|Extension]]
- [[_COMMUNITY_Core|Core]]
- [[_COMMUNITY_Core|Core]]
- [[_COMMUNITY_Runtime|Runtime]]
- [[_COMMUNITY_Templates|Templates]]
- [[_COMMUNITY_App|App]]

## God Nodes (most connected - your core abstractions)
1. `register_all()` - 45 edges
2. `Agent` - 23 edges
3. `NotificationService` - 22 edges
4. `call()` - 22 edges
5. `main()` - 20 edges
6. `build()` - 20 edges
7. `Terminal` - 19 edges
8. `run()` - 17 edges
9. `GitEngine` - 16 edges
10. `run_migrations()` - 15 edges

## Surprising Connections (you probably didn't know these)
- `build_tray_icon()` --calls--> `Icon`  [INFERRED]
  crates\porpoise-app\src\lib.rs → mobile\lib\screens\pair_screen.dart
- `make_store()` --calls--> `run_migrations()`  [INFERRED]
  crates\porpoise-agent\src\resume.rs → crates\porpoise-db\src\migration.rs
- `main()` --calls--> `build()`  [INFERRED]
  crates\porpoise-app\build.rs → templates\esp32-s3\src\orchestration\telemetry.rs
- `create_worktree()` --calls--> `call()`  [INFERRED]
  crates\porpoise-app\src\commands.rs → extension\chrome\background.js
- `run()` --calls--> `log_event()`  [INFERRED]
  crates\porpoise-app\src\lib.rs → crates\porpoise-app\src\commands.rs

## Communities

### Community 0 - "ESP32 Comms"
Cohesion: 0.02
Nodes (60): Text, WebSocketServer, begin_behavior_ota(), finalize(), finalize_behavior_ota(), OtaState, OtaTarget, process_behavior_chunk() (+52 more)

### Community 1 - "App Core"
Cohesion: 0.03
Nodes (78): annotate(), DiffSummary, build(), main(), ProcessManager, OrientationFilter, handle_open(), handle_snapshot() (+70 more)

### Community 2 - "Agent Adapters"
Cohesion: 0.02
Nodes (55): AiderAdapter, AiderOutputParser, ClaudeAdapter, ClaudeOutputParser, CodexAdapter, CodexOutputParser, OpenCodeAdapter, OpenCodeOutputParser (+47 more)

### Community 3 - "Adapter Registry"
Cohesion: 0.03
Nodes (52): AdapterRegistry, appendOutput, Terminal, TerminalProvider, update, saveConfig(), loadSettings(), AgentRow (+44 more)

### Community 4 - "Browser & Skills"
Cohesion: 0.05
Nodes (42): AccountSwitcher, AgentAccount, test_account(), test_add_and_list(), test_get_default(), test_persistence(), test_remove(), test_switch() (+34 more)

### Community 5 - "Mobile Shell"
Cohesion: 0.03
Nodes (71): build, HomeScreen, _HomeScreenState, main, MultiProvider, PorpoiseApp, Scaffold, AgentsScreen (+63 more)

### Community 6 - "Agent Models"
Cohesion: 0.05
Nodes (36): Agent, AgentProvider, appendOutput, update, OpenCodeAgent, OpenCodeHandle, test_build_run_args(), test_resolve_binary_default() (+28 more)

### Community 7 - "ESP32 WiFi"
Cohesion: 0.04
Nodes (33): WifiConfig, WifiError, WifiManager, WifiMode, WifiState, AppConfig, home_dir(), discover_config_path() (+25 more)

### Community 8 - "Process Pools"
Cohesion: 0.05
Nodes (24): ProcessPool, handle_detect(), handle_list(), handle_logs(), handle_run(), handle_stop(), AgentDetector, test_detect_all() (+16 more)

### Community 9 - "Notifications"
Cohesion: 0.06
Nodes (26): make_svc(), NotificationAction, NotificationChannel, NotificationPreferences, NotificationRecord, NotificationService, RichNotification, test_agent_completion_notification() (+18 more)

### Community 10 - "HTTP Server"
Cohesion: 0.05
Nodes (24): HttpError, HttpServer, Method, Request, Response, AuthSession, AuthState, AuthToken (+16 more)

### Community 11 - "Design Canvas"
Cohesion: 0.05
Nodes (30): auto_layout(), canvas_from_graph(), CanvasState, NodeSize, RenderedEdge, RenderedNode, Viewport, viewport_zoom_clamp() (+22 more)

### Community 12 - "CLI Commands"
Cohesion: 0.05
Nodes (41): call(), handle(), handle(), handle(), handle(), handle(), handle(), handle_qr() (+33 more)

### Community 13 - "Behavior Orchestration"
Cohesion: 0.07
Nodes (24): apply_update(), BehaviorConfig, BehaviorError, ComponentConfig, ConfigValue, default_waveshare_349(), default_waveshare_349_touch(), load_from_flash() (+16 more)

### Community 14 - "Extension Background"
Cohesion: 0.06
Nodes (20): connect(), disconnect(), getWsUrl(), loadConfig(), scheduleReconnect(), updateBadge(), handle_connect(), handle_port_forward() (+12 more)

### Community 15 - "Runtime"
Cohesion: 0.1
Nodes (15): alloc_pty_impl(), lookup(), pty_read_impl(), pty_resize_impl(), pty_write_impl(), PtyHandle, PtyManager, PtySession (+7 more)

### Community 16 - "Agent"
Cohesion: 0.13
Nodes (14): AudioConfig, AudioInput, AudioOutput, SampleFormat, build_capability_linker(), SandboxedRuntime, test_no_imports_succeeds(), test_with_import_denied() (+6 more)

### Community 17 - "Core"
Cohesion: 0.12
Nodes (13): update, Worktree, WorktreeProvider, EventBus, test_event_bus_clone(), test_multiple_subscribers(), test_publish_subscribe(), HookServer (+5 more)

### Community 18 - "Cli"
Cohesion: 0.09
Nodes (22): AgentAction, AgentArgs, BrowserAction, BrowserArgs, Cli, Commands, ConfigAction, ConfigArgs (+14 more)

### Community 19 - "Core"
Cohesion: 0.1
Nodes (11): AgentConfig, AppConfig, BrowserConfig, CliConfig, ColorChoice, CoreConfig, DbConfig, GitConfig (+3 more)

### Community 20 - "Skills"
Cohesion: 0.22
Nodes (7): HookContext, HookRegistry, HookResults, HookType, test_hook_error_collection(), test_register_and_fire(), test_unregister()

### Community 21 - "Extension"
Cohesion: 0.17
Nodes (12): injectContentScript(), createOverlay(), getComputedSnapshot(), getSelector(), onKeyDown(), onMouseClick(), playSequence(), setInspectMode() (+4 more)

### Community 22 - "Docs"
Cohesion: 0.16
Nodes (18): Porpoise — AGENTS.md, Porpoise Architecture Guide, Contributing to Porpoise, Porpoise Deployment Guide, Porpoise Development Plan, Installing Porpoise Mobile Companion on Android, Porpoise Installation Guide, Installing Porpoise on Windows (+10 more)

### Community 23 - "Runtime"
Cohesion: 0.22
Nodes (7): create_rotating_log(), Inner, LogRotationConfig, RotatingLogFile, test_max_files_enforced(), test_no_rotation_under_limit(), test_write_and_rotate()

### Community 24 - "Templates"
Cohesion: 0.12
Nodes (1): CustomBoard

### Community 25 - "Templates"
Cohesion: 0.29
Nodes (13): Get-BoardFeature(), Get-SerialPort(), Invoke-Behavior(), Invoke-Build(), Invoke-Clean(), Invoke-Flash(), Invoke-Monitor(), Invoke-Ota() (+5 more)

### Community 26 - "Mobile"
Cohesion: 0.14
Nodes (13): disconnect, dispose, _doConnect, Exception, _generateId, _onDone, _onError, _onMessage (+5 more)

### Community 27 - "Templates"
Cohesion: 0.15
Nodes (1): WaveshareLcd349

### Community 28 - "Docs"
Cohesion: 0.14
Nodes (14): Module Design: porpoise-agent, Module Design: porpoise-app, Module Design: porpoise-browser, Module Design: porpoise-cli, Module Design: porpoise-core, Module Design: porpoise-db, Module Design: porpoise-git, Module Design: porpoise-network (+6 more)

### Community 29 - "App"
Cohesion: 0.14
Nodes (14): 128X128@2X, 128X128, 32X32, Icon, Square107X107Logo, Square142X142Logo, Square150X150Logo, Square284X284Logo (+6 more)

### Community 30 - "Core"
Cohesion: 0.15
Nodes (12): AgentEvent, AgentStatusKind, BrowserEvent, GitEvent, NotificationSeverity, OutputKind, SshEvent, SystemEvent (+4 more)

### Community 32 - "Git"
Cohesion: 0.17
Nodes (11): BranchInfo, Change, ChangeStatus, CommitEntry, Issue, IssueState, PrState, PullRequest (+3 more)

### Community 33 - "Design"
Cohesion: 0.35
Nodes (10): agent_props_count(), make_node(), merge_has_no_props(), PropDef, props_for_kind(), PropType, validate_bad_select(), validate_missing_required() (+2 more)

### Community 34 - "Templates"
Cohesion: 0.18
Nodes (1): WaveshareLcd349Touch

### Community 35 - "Templates"
Cohesion: 0.18
Nodes (10): CapabilitiesMsg, CommandMsg, CommandResultMsg, Envelope, HelloMsg, MsgType, OtaChunkMsg, PeripheralInfoMsg (+2 more)

### Community 36 - "Templates"
Cohesion: 0.18
Nodes (7): KeyboardPeripheral, KeyboardReport, KeyEvent, Modifiers, MouseButtons, MousePeripheral, MouseReport

### Community 37 - "Agent"
Cohesion: 0.2
Nodes (8): AdapterCapabilities, AgentAdapter, DiffPatch, InputMode, OutputParser, ParsedEvent, RawOutput, ToolCall

### Community 38 - "Design"
Cohesion: 0.29
Nodes (6): agent_component_has_two_outputs(), ComponentDef, end_has_no_outputs(), merge_accepts_multi_input(), Port, start_has_no_inputs()

### Community 39 - "Core"
Cohesion: 0.36
Nodes (7): from_bincode(), from_json(), test_bincode_roundtrip(), test_json_pretty_has_newlines(), test_json_roundtrip(), to_bincode(), to_json_pretty()

### Community 40 - "Network"
Cohesion: 0.31
Nodes (3): RateLimitConfig, RateLimiter, test_rate_limiter_consume()

### Community 41 - "Skills"
Cohesion: 0.25
Nodes (2): SkillManifest, SkillRegistry

### Community 42 - "Templates"
Cohesion: 0.22
Nodes (8): BoardConfig, DisplayInterface, DvpConfig, I2cConfig, I2sConfig, PeripheralMap, SpiConfig, TouchConfig

### Community 43 - "Agent"
Cohesion: 0.25
Nodes (6): AgentInfo, AgentKind, AgentManifest, AgentOutput, AgentStatus, OutputType

### Community 44 - "Core"
Cohesion: 0.25
Nodes (7): AnnotatedDiff, DiffLine, FilePatch, FileStatus, Hunk, LineAnnotation, LineKind

### Community 45 - "Core"
Cohesion: 0.38
Nodes (3): Platform, test_current_platform_is_unix(), test_display()

### Community 46 - "Core"
Cohesion: 0.29
Nodes (6): Capabilities, Capability, CapabilityScope, HostPattern, ProcessPattern, UrlPattern

### Community 47 - "Terminal"
Cohesion: 0.29
Nodes (5): ColorScheme, OutputLine, SplitDirection, TerminalConfig, TerminalPane

### Community 48 - "Templates"
Cohesion: 0.29
Nodes (6): CommandResponse, RemoteCommand, TelemetryFrame, WsError, WsMessage, WsState

### Community 49 - "Opencode"
Cohesion: 0.33
Nodes (3): OpenCodeConfig, OpenCodeOutputFormat, OpenCodeServerConfig

### Community 50 - "Templates"
Cohesion: 0.33
Nodes (4): CameraInfo, CameraPeripheral, PixelFormat, Resolution

### Community 51 - "Templates"
Cohesion: 0.33
Nodes (5): DataRate, ImuChip, ImuPeripheral, ImuReading, Vec3

### Community 52 - "Db"
Cohesion: 0.4
Nodes (4): Create, Delete, Read, Update

### Community 53 - "Runtime"
Cohesion: 0.4
Nodes (4): ProcessCommand, ProcessEntry, ProcessKind, ProcessStatus

### Community 54 - "Templates"
Cohesion: 0.4
Nodes (4): ColorFormat, DisplayInfo, DisplayPeripheral, Orientation

### Community 55 - "Templates"
Cohesion: 0.4
Nodes (1): GestureRecognizer

### Community 56 - "Templates"
Cohesion: 0.5
Nodes (3): TouchEvent, TouchInfo, TouchPeripheral

### Community 57 - "Agent"
Cohesion: 0.67
Nodes (2): Agent, AgentHandle

### Community 58 - "Browser"
Cohesion: 0.67
Nodes (2): NavigationResult, NavigationStatus

### Community 59 - "Templates"
Cohesion: 0.67
Nodes (2): DeviceCapabilities, DeviceIdentity

### Community 60 - "Templates"
Cohesion: 0.67
Nodes (2): Gesture, Orientation

### Community 61 - "Docs"
Cohesion: 0.67
Nodes (3): porpoise-core: Core Type System, porpoise-relay: IPC Protocol Design, porpoise-runtime: Process & PTY Management

### Community 62 - "Extension"
Cohesion: 0.67
Nodes (3): Icon128, Icon16, Icon48

### Community 63 - "Core"
Cohesion: 1.0
Nodes (1): Command

### Community 64 - "Core"
Cohesion: 1.0
Nodes (1): EventHandler

### Community 65 - "Runtime"
Cohesion: 1.0
Nodes (1): ProcessHandle

### Community 67 - "Templates"
Cohesion: 1.0
Nodes (1): CustomPeripheral

### Community 68 - "App"
Cohesion: 1.0
Nodes (2): Index, Workflow

## Knowledge Gaps
- **345 isolated node(s):** `AgentAccount`, `AgentEntry`, `SessionRecord`, `SessionStatus`, `Agent` (+340 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Templates`** (16 nodes): `CustomBoard`, `.audio_config()`, `.camera_config()`, `.display_config()`, `.has_camera()`, `.has_display()`, `.has_imu()`, `.has_keyboard()`, `.has_mic()`, `.has_mouse()`, `.has_speaker()`, `.has_touch()`, `.imu_config()`, `.name()`, `.touch_config()`, `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Templates`** (14 nodes): `mod.rs`, `WaveshareLcd349`, `.cpu_frequency_mhz()`, `.default()`, `.description()`, `.display_config()`, `.display_resolution()`, `.flash_size()`, `.imu_i2c_config()`, `.name()`, `.new()`, `.peripheral_map()`, `.psram_size()`, `.touch_config()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Templates`** (11 nodes): `mod.rs`, `WaveshareLcd349Touch`, `.cpu_frequency_mhz()`, `.description()`, `.display_config()`, `.display_resolution()`, `.flash_size()`, `.imu_i2c_config()`, `.peripheral_map()`, `.psram_size()`, `.touch_config()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Skills`** (9 nodes): `registry.rs`, `SkillManifest`, `SkillRegistry`, `.default()`, `.disable()`, `.enable()`, `.list()`, `.new()`, `.register()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Templates`** (5 nodes): `GestureRecognizer`, `.feed()`, `.last()`, `.new()`, `gesture.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Agent`** (3 nodes): `traits.rs`, `Agent`, `AgentHandle`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Browser`** (3 nodes): `navigation.rs`, `NavigationResult`, `NavigationStatus`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Templates`** (3 nodes): `DeviceCapabilities`, `DeviceIdentity`, `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Templates`** (3 nodes): `Gesture`, `Orientation`, `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Core`** (2 nodes): `command.rs`, `Command`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Core`** (2 nodes): `event_handler.rs`, `EventHandler`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Runtime`** (2 nodes): `handle.rs`, `ProcessHandle`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Templates`** (2 nodes): `CustomPeripheral`, `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `App`** (2 nodes): `Index`, `Workflow`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Text` connect `ESP32 Comms` to `Notifications`, `Agent Adapters`, `Browser & Skills`, `Mobile Shell`?**
  _High betweenness centrality (0.053) - this node is a cross-community bridge._
- **Are the 44 inferred relationships involving `register_all()` (e.g. with `.start()` and `.clone()`) actually correct?**
  _`register_all()` has 44 INFERRED edges - model-reasoned connections that need verification._
- **Are the 22 inferred relationships involving `Agent` (e.g. with `.switch()` and `.load_accounts()`) actually correct?**
  _`Agent` has 22 INFERRED edges - model-reasoned connections that need verification._
- **Are the 21 inferred relationships involving `call()` (e.g. with `list_worktrees()` and `create_worktree()`) actually correct?**
  _`call()` has 21 INFERRED edges - model-reasoned connections that need verification._
- **Are the 16 inferred relationships involving `main()` (e.g. with `.run()` and `.parse()`) actually correct?**
  _`main()` has 16 INFERRED edges - model-reasoned connections that need verification._
- **What connects `AgentAccount`, `AgentEntry`, `SessionRecord` to the rest of the system?**
  _345 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `ESP32 Comms` be split into smaller, more focused modules?**
  _Cohesion score 0.02 - nodes in this community are weakly interconnected._