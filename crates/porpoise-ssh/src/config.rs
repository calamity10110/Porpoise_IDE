use std::path::PathBuf;

pub fn parse_ssh_config(path: &PathBuf) -> Vec<HostConfig> {
    let mut hosts = Vec::new();
    if !path.exists() {
        return hosts;
    }
    let content = std::fs::read_to_string(path).ok();
    if let Some(text) = content {
        let mut current = HostConfig::default();
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if line.to_lowercase().starts_with("host ") {
                if !current.host.is_empty() {
                    hosts.push(std::mem::take(&mut current));
                }
                current.host = line[5..].trim().to_string();
                continue;
            }
            if let Some((key, val)) = line.split_once(char::is_whitespace).filter(|(k, _)| !k.is_empty()) {
                match key {
                    "HostName" => current.host_name = val.trim().to_string(),
                    "Port" => current.port = val.trim().parse().ok().unwrap_or(22),
                    "User" => current.user = val.trim().to_string(),
                    "IdentityFile" => current.identity_file = Some(PathBuf::from(val.trim())),
                    _ => {}
                }
            }
        }
        if !current.host.is_empty() {
            hosts.push(current);
        }
    }
    hosts
}

#[derive(Debug, Clone, Default)]
pub struct HostConfig {
    pub host: String,
    pub host_name: String,
    pub port: u16,
    pub user: String,
    pub identity_file: Option<PathBuf>,
}
