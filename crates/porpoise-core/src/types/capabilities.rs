use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Set of granted capabilities for a plugin or component.
///
/// Each field declares what the component is allowed to access. Empty vecs
/// mean no access in that domain. Used by the WASM plugin sandbox (Phase 7).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Capabilities {
    /// Filesystem paths the component can read.
    pub fs_read: Vec<PathBuf>,
    /// Filesystem paths the component can write.
    pub fs_write: Vec<PathBuf>,
    /// Network URL patterns the component can access.
    pub network: Vec<UrlPattern>,
    /// Process patterns the component can spawn.
    pub process: Vec<ProcessPattern>,
    /// SSH hosts the component can connect to.
    pub ssh: Vec<HostPattern>,
}

/// A URL pattern for capability matching (scheme + host + optional port/path).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlPattern {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: Option<String>,
}

/// A process pattern for capability matching (binary name + allowed args).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessPattern {
    pub binary: String,
    pub args_allow: Vec<String>,
}

/// An SSH host pattern for capability matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostPattern {
    pub host: String,
    pub port: Option<u16>,
}

/// A named capability with its scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub scope: CapabilityScope,
}

/// The scope of a capability — what resources it grants access to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityScope {
    /// Access to specific filesystem paths, optionally with write permission.
    Filesystem { paths: Vec<PathBuf>, write: bool },
    /// Access to specific URL patterns.
    Network { urls: Vec<String> },
    /// Permission to spawn specific processes.
    Process { allow: Vec<String> },
    /// Permission to connect to specific SSH hosts.
    Ssh { hosts: Vec<String> },
    /// Unrestricted access (use sparingly).
    All,
    /// No access.
    None,
}
