use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "porpoise",
    version,
    about = "AI Orchestration IDE - Rust-native port of Orca"
)]
#[command(propagate_version = true)]
pub struct Cli {
    #[arg(global = true, long, help = "Output as JSON")]
    pub json: bool,

    #[arg(global = true, long, short, help = "Verbose output")]
    pub verbose: bool,

    #[arg(global = true, long, help = "Debug mode")]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage the porpoise daemon
    Daemon(DaemonArgs),
    /// Manage worktrees
    Worktree(WorktreeArgs),
    /// Manage terminals
    Terminal(TerminalArgs),
    /// Manage agents
    Agent(AgentArgs),
    /// Git operations
    Git(GitArgs),
    /// Browser automation
    Browser(BrowserArgs),
    /// SSH connections
    Ssh(SshArgs),
    /// Configuration
    Config(ConfigArgs),
    /// Skills/plugins
    Skill(SkillArgs),
    /// Store and retrieve credentials (encrypted local vault)
    Credential(CredentialArgs),
    /// Mobile companion pairing
    Mobile(MobileArgs),
    /// Launch the daemon and/or desktop app, health-check, print pairing QR
    Launch(LaunchArgs),
    /// Show system status
    Status,
    /// Show version
    Version,
}

#[derive(Args)]
pub struct LaunchArgs {
    /// Launch only the desktop app (daemon must already be running)
    #[arg(long)]
    pub app: bool,

    /// Launch only the daemon (no desktop app)
    #[arg(long)]
    pub server: bool,
}

#[derive(Args)]
pub struct CredentialArgs {
    #[command(subcommand)]
    pub action: CredentialAction,
}

#[derive(Subcommand)]
pub enum CredentialAction {
    /// Store or update a credential (encrypted local vault)
    Set {
        /// Name to store under (e.g. github-token)
        name: String,
        /// Credential type label
        #[arg(long, default_value = "api_key")]
        credential_type: String,
        /// Secret value; omitted = read one line from stdin
        #[arg(long)]
        value: Option<String>,
    },
    /// Print a stored credential's secret
    Get { name: String },
    /// List stored credential names and types (never secrets)
    List,
    /// Delete a stored credential
    Delete { name: String },
}

#[derive(Args)]
pub struct DaemonArgs {
    #[command(subcommand)]
    pub action: DaemonAction,
}

#[derive(Subcommand)]
pub enum DaemonAction {
    Start,
    Stop,
    Status,
}

#[derive(Args)]
pub struct WorktreeArgs {
    #[command(subcommand)]
    pub action: WorktreeAction,
}

#[derive(Subcommand)]
pub enum WorktreeAction {
    Create {
        name: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        agent: Option<String>,
        #[arg(long)]
        prompt: Option<String>,
    },
    List,
    Show {
        name: String,
    },
    Rm {
        name: String,
    },
    Prune {
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Args)]
pub struct TerminalArgs {
    #[command(subcommand)]
    pub action: TerminalAction,
}

#[derive(Subcommand)]
pub enum TerminalAction {
    Create {
        #[arg(long)]
        worktree: String,
        #[arg(long)]
        shell: Option<String>,
    },
    List {
        #[arg(long)]
        worktree: Option<String>,
    },
    Send {
        id: String,
        #[arg(long)]
        text: String,
        #[arg(long)]
        enter: bool,
    },
    Read {
        id: String,
    },
    Resize {
        id: String,
        rows: u16,
        cols: u16,
    },
    Close {
        id: String,
    },
}

#[derive(Args)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub action: AgentAction,
}

#[derive(Subcommand)]
pub enum AgentAction {
    List,
    Run {
        kind: String,
        #[arg(long)]
        worktree: String,
        #[arg(long)]
        prompt: String,
    },
    Stop {
        id: String,
    },
    Logs {
        id: String,
        #[arg(long)]
        lines: Option<u32>,
    },
    /// Start an agent run that streams AG-UI 1.0 protocol events.
    AguiRun {
        kind: String,
        #[arg(long)]
        worktree: String,
        #[arg(long)]
        prompt: String,
        /// Continue an existing conversation thread instead of starting a new one.
        #[arg(long)]
        thread: Option<String>,
    },
    /// Print a run's AG-UI events as SSE frames.
    AguiEvents {
        #[arg(long)]
        run_id: String,
        /// Poll events appended after this offset.
        #[arg(long, default_value_t = 0)]
        after: usize,
        /// Keep polling and printing until the run reaches its terminal event.
        #[arg(long)]
        follow: bool,
    },
    /// Interrupt a live AG-UI run (it ends with the cancelled outcome).
    AguiStop {
        #[arg(long)]
        run_id: String,
    },
}

#[derive(Args)]
pub struct GitArgs {
    #[command(subcommand)]
    pub action: GitAction,
}

#[derive(Subcommand)]
pub enum GitAction {
    Status {
        repo_path: String,
    },
    Diff {
        repo_path: String,
        #[arg(long)]
        staged: bool,
    },
    Log {
        repo_path: String,
        #[arg(long)]
        count: Option<u32>,
    },
    Clone {
        url: String,
        path: String,
    },
    Branch {
        repo_path: String,
    },
}

#[derive(Args)]
pub struct BrowserArgs {
    #[command(subcommand)]
    pub action: BrowserAction,
}

#[derive(Subcommand)]
pub enum BrowserAction {
    Open {
        url: String,
    },
    Snapshot {
        page_id: String,
    },
    Click {
        page_id: String,
        selector: String,
    },
    Fill {
        page_id: String,
        selector: String,
        value: String,
    },
}

#[derive(Args)]
pub struct SshArgs {
    #[command(subcommand)]
    pub action: SshAction,
}

#[derive(Subcommand)]
pub enum SshAction {
    Connect {
        host: String,
        user: Option<String>,
    },
    Worktree {
        session_id: String,
    },
    PortForward {
        session_id: String,
        local: u16,
        remote: u16,
    },
}

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    Get { key: String },
    Set { key: String, value: String },
    List,
    Edit,
}

#[derive(Args)]
pub struct SkillArgs {
    #[command(subcommand)]
    pub action: SkillAction,
}

#[derive(Args)]
pub struct MobileArgs {
    #[command(subcommand)]
    pub action: MobileAction,
}

#[derive(Subcommand)]
pub enum MobileAction {
    /// Print pairing info (host, port, token) for mobile app connection
    Qr,
}

#[derive(Subcommand)]
pub enum SkillAction {
    Search { query: String },
    Install { name: String },
    Uninstall { name: String },
    List,
}
