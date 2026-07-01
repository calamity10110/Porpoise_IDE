use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub fs_read: Vec<PathBuf>,
    pub fs_write: Vec<PathBuf>,
    pub network: Vec<UrlPattern>,
    pub process: Vec<ProcessPattern>,
    pub ssh: Vec<HostPattern>,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            fs_read: vec![],
            fs_write: vec![],
            network: vec![],
            process: vec![],
            ssh: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlPattern {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessPattern {
    pub binary: String,
    pub args_allow: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostPattern {
    pub host: String,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub scope: CapabilityScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityScope {
    Filesystem { paths: Vec<PathBuf>, write: bool },
    Network { urls: Vec<String> },
    Process { allow: Vec<String> },
    Ssh { hosts: Vec<String> },
    All,
    None,
}
