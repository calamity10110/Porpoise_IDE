use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// OpenCode-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeConfig {
    /// Path to the opencode binary. If None, looks up "opencode" on PATH.
    pub binary_path: Option<PathBuf>,
    /// Default model to use (format: "provider/model").
    pub default_model: Option<String>,
    /// Default agent to use.
    pub default_agent: Option<String>,
    /// Whether to run without external plugins.
    pub pure_mode: bool,
    /// Log level for opencode subprocess.
    pub log_level: Option<String>,
    /// Whether to show thinking blocks.
    pub show_thinking: bool,
    /// Auto-approve permissions (dangerous).
    pub dangerously_skip_permissions: bool,
    /// Default format for output.
    pub output_format: OpenCodeOutputFormat,
    /// Timeout for individual runs in seconds.
    pub run_timeout_secs: u64,
    /// Additional CLI arguments passed to opencode.
    pub extra_args: Vec<String>,
}

impl Default for OpenCodeConfig {
    fn default() -> Self {
        Self {
            binary_path: None,
            default_model: None,
            default_agent: None,
            pure_mode: false,
            log_level: None,
            show_thinking: false,
            dangerously_skip_permissions: false,
            output_format: OpenCodeOutputFormat::Json,
            run_timeout_secs: 300,
            extra_args: vec![],
        }
    }
}

/// Output format for OpenCode runs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OpenCodeOutputFormat {
    /// Human-readable formatted output.
    Default,
    /// Raw JSON events (for machine parsing).
    Json,
}

impl std::fmt::Display for OpenCodeOutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::Json => write!(f, "json"),
        }
    }
}

/// Server connection config for attaching to a running OpenCode instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeServerConfig {
    /// URL of the running OpenCode server (e.g., "http://localhost:4096").
    pub url: String,
    /// Optional basic auth username.
    pub username: Option<String>,
    /// Optional basic auth password.
    pub password: Option<String>,
}
