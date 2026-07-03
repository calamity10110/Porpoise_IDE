use super::AppConfig;

impl AppConfig {
    /// Returns the default Unix domain socket path based on platform conventions.
    pub fn default_socket_path() -> Result<std::path::PathBuf, crate::error::PorpoiseError> {
        let data_dir = Self::default_data_dir()?;
        Ok(data_dir.join("porpoise.sock"))
    }

    /// Returns the default application data directory per platform:
    /// - Linux: `$XDG_DATA_HOME/porpoise` or `~/.local/share/porpoise`
    /// - macOS: `~/Library/Application Support/porpoise`
    /// - Windows: `%LOCALAPPDATA%/porpoise`
    pub fn default_data_dir() -> Result<std::path::PathBuf, crate::error::PorpoiseError> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
                Ok(std::path::PathBuf::from(xdg).join("porpoise"))
            } else {
                let home = home_dir()?;
                Ok(home.join(".local").join("share").join("porpoise"))
            }
        }

        #[cfg(target_os = "macos")]
        {
            let home = home_dir()?;
            Ok(home.join("Library").join("Application Support").join("porpoise"))
        }

        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("LOCALAPPDATA")
                .map_err(|_| crate::error::PorpoiseError::Config("LOCALAPPDATA not set".into()))?;
            Ok(std::path::PathBuf::from(appdata).join("porpoise"))
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            compile_error!("unsupported target OS");
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn home_dir() -> Result<std::path::PathBuf, crate::error::PorpoiseError> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| crate::error::PorpoiseError::Config("cannot determine home directory".into()))
        .map(std::path::PathBuf::from)
}
