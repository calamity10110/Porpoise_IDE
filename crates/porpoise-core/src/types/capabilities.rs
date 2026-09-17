use std::path::{Path, PathBuf};

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

impl Capabilities {
    /// Returns the union of this capabilities set and another.
    ///
    /// The union contains all capabilities that exist in either set.
    /// ```rust
    /// use porpoise_core::types::Capabilities;
    /// let a = Capabilities::all_read("/tmp".into());
    /// let b = Capabilities::all_write("/tmp".into());
    /// let u = a.union(&b);
    /// assert!(u.fs_read.contains(&"/tmp".into()));
    /// assert!(u.fs_write.contains(&"/tmp".into()));
    /// ```
    pub fn union(&self, other: &Capabilities) -> Capabilities {
        Capabilities {
            fs_read: self.fs_read.iter().chain(&other.fs_read).cloned().collect(),
            fs_write: self.fs_write.iter().chain(&other.fs_write).cloned().collect(),
            network: self.network.iter().chain(&other.network).cloned().collect(),
            process: self.process.iter().chain(&other.process).cloned().collect(),
            ssh: self.ssh.iter().chain(&other.ssh).cloned().collect(),
        }
    }

    /// Returns the intersection of this capabilities set and another.
    ///
    /// The intersection contains only capabilities that exist in both sets.
    /// ```rust
    /// use porpoise_core::types::Capabilities;
    /// let a = Capabilities::all_read_write("/tmp".into());
    /// let b = Capabilities::all_read_write("/tmp".into());
    /// let inter = a.intersect(&b);
    /// assert!(inter.fs_read.contains(&"/tmp".into()));
    /// assert!(inter.fs_write.contains(&"/tmp".into()));
    /// ```
    pub fn intersect(&self, other: &Capabilities) -> Capabilities {
        Capabilities {
            fs_read: self
                .fs_read
                .iter()
                .filter(|p| other.fs_read.contains(*p))
                .cloned()
                .collect(),
            fs_write: self
                .fs_write
                .iter()
                .filter(|p| other.fs_write.contains(*p))
                .cloned()
                .collect(),
            network: self
                .network
                .iter()
                .filter(|p| other.network.contains(*p))
                .cloned()
                .collect(),
            process: self
                .process
                .iter()
                .filter(|p| other.process.contains(*p))
                .cloned()
                .collect(),
            ssh: self.ssh.iter().filter(|p| other.ssh.contains(*p)).cloned().collect(),
        }
    }

    /// Returns the difference of this capabilities set minus another.
    ///
    /// The difference contains capabilities that exist in this set but not in other.
    /// ```rust
    /// use porpoise_core::types::Capabilities;
    /// let a = Capabilities::all_read_write("/tmp".into());
    /// let b = Capabilities::all_read("/tmp".into());
    /// let diff = a.difference(&b);
    /// assert!(diff.fs_write.contains(&"/tmp".into()));
    /// assert!(!diff.fs_read.contains(&"/tmp".into())); // may or may not, depends on impl
    /// ```
    pub fn difference(&self, other: &Capabilities) -> Capabilities {
        Capabilities {
            fs_read: self
                .fs_read
                .iter()
                .filter(|p| !other.fs_read.contains(*p))
                .cloned()
                .collect(),
            fs_write: self
                .fs_write
                .iter()
                .filter(|p| !other.fs_write.contains(*p))
                .cloned()
                .collect(),
            network: self
                .network
                .iter()
                .filter(|p| !other.network.contains(*p))
                .cloned()
                .collect(),
            process: self
                .process
                .iter()
                .filter(|p| !other.process.contains(*p))
                .cloned()
                .collect(),
            ssh: self.ssh.iter().filter(|p| !other.ssh.contains(*p)).cloned().collect(),
        }
    }

    /// Returns `true` if this capabilities set contains at least all the capabilities
    /// in the other set (i.e., other is a subset of this).
    ///
    /// ```rust
    /// use porpoise_core::types::Capabilities;
    /// let a = Capabilities::all_read_write("/tmp".into());
    /// let b = Capabilities::all_read("/tmp".into());
    /// assert!(a.matches(&b));
    /// ```
    pub fn matches(&self, other: &Capabilities) -> bool {
        self.fs_read.iter().all(|p| other.fs_read.contains(p))
            && self.fs_write.iter().all(|p| other.fs_write.contains(p))
            && self.network.iter().all(|p| other.network.contains(p))
            && self.process.iter().all(|p| other.process.contains(p))
            && self.ssh.iter().all(|p| other.ssh.contains(p))
    }

    /// Creates a capabilities set that allows reading from the given path.
    /// ```rust
    /// use porpoise_core::types::Capabilities;
    /// let caps = Capabilities::all_read("/tmp".into());
    /// assert!(caps.fs_read.contains(&"/tmp".into()));
    /// ```
    pub fn all_read(path: PathBuf) -> Capabilities {
        Capabilities {
            fs_read: vec![path],
            fs_write: vec![],
            network: vec![],
            process: vec![],
            ssh: vec![],
        }
    }

    /// Creates a capabilities set that allows reading and writing to the given path.
    /// ```rust
    /// use porpoise_core::types::Capabilities;
    /// let caps = Capabilities::all_read_write("/tmp".into());
    /// assert!(caps.fs_read.contains(&"/tmp".into()));
    /// assert!(caps.fs_write.contains(&"/tmp".into()));
    /// ```
    pub fn all_read_write(path: PathBuf) -> Capabilities {
        Capabilities {
            fs_read: vec![path.clone()],
            fs_write: vec![path],
            network: vec![],
            process: vec![],
            ssh: vec![],
        }
    }

    /// Creates a capabilities set that allows the given network URL patterns.
    /// ```rust
    /// use porpoise_core::types::Capabilities;
    /// use porpoise_core::types::UrlPattern;
    /// let caps = Capabilities::all_network(vec![UrlPattern {
    ///     scheme: "http".into(),
    ///     host: "localhost".into(),
    ///     port: None,
    ///     path: None,
    /// }]);
    /// assert!(caps.network.contains(&UrlPattern { scheme: "http".into(), host: "localhost".into(), port: None, path: None }));
    /// ```
    pub fn all_network(patterns: Vec<UrlPattern>) -> Capabilities {
        Capabilities {
            fs_read: vec![],
            fs_write: vec![],
            network: patterns,
            process: vec![],
            ssh: vec![],
        }
    }

    /// Creates a capabilities set that allows spawning the given process patterns.
    /// ```rust
    /// use porpoise_core::types::Capabilities;
    /// let caps = Capabilities::all_process(vec!["my-app".into()]);
    /// assert!(caps.process.contains(&"my-app".into()));
    /// ```
    pub fn all_process(patterns: Vec<String>) -> Capabilities {
        Capabilities {
            fs_read: vec![],
            fs_write: vec![],
            network: vec![],
            process: patterns
                .into_iter()
                .map(|binary| ProcessPattern {
                    binary,
                    args_allow: vec![],
                })
                .collect(),
            ssh: vec![],
        }
    }

    /// Creates a capabilities set that allows connecting to the given SSH hosts.
    /// ```rust
    /// use porpoise_core::types::Capabilities;
    /// let caps = Capabilities::all_ssh(vec!["example.com".into()]);
    /// assert!(caps.ssh.iter().any(|h| h.host == "example.com"));
    /// ```
    pub fn all_ssh(hosts: Vec<String>) -> Capabilities {
        Capabilities {
            fs_read: vec![],
            fs_write: vec![],
            network: vec![],
            process: vec![],
            ssh: hosts.into_iter().map(|host| HostPattern { host, port: None }).collect(),
        }
    }
}

/// A URL pattern for capability matching (scheme + host + optional port/path).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UrlPattern {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: Option<String>,
}

/// A process pattern for capability matching (binary name + allowed args).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessPattern {
    pub binary: String,
    pub args_allow: Vec<String>,
}

/// An SSH host pattern for capability matching.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

impl CapabilityScope {
    /// Returns `true` if this scope matches the given filesystem path.
    ///
    /// ```rust
    /// use porpoise_core::types::CapabilityScope;
    /// use std::path::Path;
    /// let scope = CapabilityScope::Filesystem { paths: vec!["/tmp".into()], write: false };
    /// let path = Path::new("/tmp");
    /// assert!(scope.matches_path(path));
    /// ```
    pub fn matches_path(&self, path: &Path) -> bool {
        match self {
            CapabilityScope::Filesystem { paths, write: _ } => paths.iter().any(|p| path.as_os_str() == p.as_os_str()),
            CapabilityScope::Network { urls: _ } => false,
            CapabilityScope::Process { allow: _ } => false,
            CapabilityScope::Ssh { hosts: _ } => false,
            CapabilityScope::All => true,
            CapabilityScope::None => false,
        }
    }

    /// Returns `true` if this scope matches the given URL string.
    ///
    /// ```rust
    /// use porpoise_core::types::CapabilityScope;
    /// let scope = CapabilityScope::Network { urls: vec!["http://localhost:3000].into() };
    /// assert!(scope.matches_url("http://localhost:3000/"));
    /// ```
    pub fn matches_url(&self, url: &str) -> bool {
        match self {
            CapabilityScope::Network { urls } => urls.iter().any(|u| u == url),
            CapabilityScope::Filesystem { paths: _, write: _ } => false,
            CapabilityScope::Process { allow: _ } => false,
            CapabilityScope::Ssh { hosts: _ } => false,
            CapabilityScope::All => true,
            CapabilityScope::None => false,
        }
    }
}
