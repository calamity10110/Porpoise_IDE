pub mod defaults;
pub mod discovery;

use std::path::PathBuf;

use serde::Deserialize;

/// Top-level application configuration, deserialized from TOML + env vars.
///
/// Each sub-config field corresponds to a `[section]` in the config file or
/// `PORPOISE_SECTION_KEY` environment variable. All sub-configs implement
/// `Default` so missing sections are handled gracefully.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub core: CoreConfig,
    #[serde(default)]
    pub cli: CliConfig,
    #[serde(default)]
    pub db: DbConfig,
    #[serde(default)]
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub ssh: SshConfig,
    #[serde(default)]
    pub browser: BrowserConfig,
}

/// Core application settings.
#[derive(Debug, Clone, Deserialize)]
pub struct CoreConfig {
    pub data_dir: Option<PathBuf>,
    pub event_bus_capacity: usize,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            data_dir: None,
            event_bus_capacity: 1024,
        }
    }
}

/// CLI-specific settings (output format, pager, color).
#[derive(Debug, Clone, Deserialize)]
pub struct CliConfig {
    pub default_output_format: OutputFormat,
    pub pager: bool,
    pub color: ColorChoice,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            default_output_format: OutputFormat::Plain,
            pager: true,
            color: ColorChoice::Auto,
        }
    }
}

/// Supported CLI output formats.
#[derive(Debug, Clone, Deserialize)]
pub enum OutputFormat {
    Plain,
    Json,
    JsonPretty,
    Yaml,
}

/// Color output preference.
#[derive(Debug, Clone, Deserialize)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

/// Database connection settings.
#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
    pub url: Option<String>,
    pub pool_size: u32,
    pub wal_mode: bool,
    pub busy_timeout_ms: u64,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: None,
            pool_size: 4,
            wal_mode: true,
            busy_timeout_ms: 5000,
        }
    }
}

/// Runtime/process manager settings.
#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
    pub max_concurrent_processes: u32,
    pub process_timeout_secs: u64,
    pub health_check_interval_secs: u64,
    pub pty_buffer_size: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_concurrent_processes: 32,
            process_timeout_secs: 86400,
            health_check_interval_secs: 30,
            pty_buffer_size: 65536,
        }
    }
}

/// Agent integration settings.
#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub default_shell: String,
    pub max_concurrent_agents: u32,
    pub agent_timeout_secs: u64,
    pub allowed_agents: Vec<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            default_shell: "bash".into(),
            max_concurrent_agents: 8,
            agent_timeout_secs: 86400,
            allowed_agents: vec![],
        }
    }
}

/// Git integration settings.
#[derive(Debug, Clone, Deserialize)]
pub struct GitConfig {
    pub default_branch: String,
    pub fetch_on_open: bool,
    pub auto_prune_worktrees: bool,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            default_branch: "main".into(),
            fetch_on_open: true,
            auto_prune_worktrees: true,
        }
    }
}

/// SSH connection settings.
#[derive(Debug, Clone, Deserialize)]
pub struct SshConfig {
    pub connect_timeout_secs: u64,
    pub keepalive_interval_secs: u64,
    pub max_retries: u32,
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            connect_timeout_secs: 10,
            keepalive_interval_secs: 30,
            max_retries: 3,
        }
    }
}

/// Embedded browser settings.
#[derive(Debug, Clone, Deserialize)]
pub struct BrowserConfig {
    pub default_width: u32,
    pub default_height: u32,
    pub user_agent: Option<String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            default_width: 1280,
            default_height: 720,
            user_agent: None,
        }
    }
}
