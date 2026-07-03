use std::path::PathBuf;

use crate::error::Result;
use super::AppConfig;

/// Discovers the configuration file path using this priority:
/// 1. `PORPOISE_CONFIG` environment variable
/// 2. XDG config directory (Linux/macOS) or `%APPDATA%` (Windows)
pub fn discover_config_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("PORPOISE_CONFIG") {
        return Ok(PathBuf::from(path));
    }

    let base = platform_config_dir()?;
    Ok(base.join("porpoise").join("config.toml"))
}

fn platform_config_dir() -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            Ok(PathBuf::from(xdg))
        } else {
            let home = home_dir()?;
            Ok(home.join(".config"))
        }
    }

    #[cfg(target_os = "macos")]
    {
        let home = home_dir()?;
        Ok(home.join(".config"))
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| crate::error::PorpoiseError::Config("APPDATA not set".into()))?;
        Ok(PathBuf::from(appdata))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        compile_error!("unsupported target OS");
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn home_dir() -> Result<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| crate::error::PorpoiseError::Config("cannot determine home directory".into()))
        .map(PathBuf::from)
}

/// Loads configuration from TOML file + environment variables with `PORPOISE_` prefix.
///
/// Falls back to `AppConfig::default()` if no config file exists.
pub fn load_config() -> Result<AppConfig> {
    let path = discover_config_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let mut cfg = config::Config::builder();
    cfg = cfg.add_source(config::File::from(path.as_path()).required(false));
    cfg = cfg.add_source(config::Environment::with_prefix("PORPOISE"));

    let settings = cfg
        .build()
        .map_err(|e| crate::error::PorpoiseError::ConfigParse(e.to_string()))?;

    let app_config: AppConfig = settings
        .try_deserialize()
        .map_err(|e| crate::error::PorpoiseError::ConfigParse(e.to_string()))?;

    Ok(app_config)
}
