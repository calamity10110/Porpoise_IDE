use std::path::PathBuf;
use porpoise_core::config::discovery;

#[derive(Debug, Clone)]
pub struct CliSettings {
    pub output_format: super::output::OutputFormat,
    pub pager: bool,
    pub color: bool,
    pub config_path: PathBuf,
}

impl Default for CliSettings {
    fn default() -> Self {
        Self {
            output_format: super::output::OutputFormat::Plain,
            pager: true,
            color: true,
            config_path: discovery::discover_config_path()
                .unwrap_or_else(|_| PathBuf::from("config.toml")),
        }
    }
}
