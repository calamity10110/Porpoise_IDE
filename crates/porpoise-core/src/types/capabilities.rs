use std::path::{Component, Path, PathBuf};

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

fn merged<T: Clone + PartialEq>(a: &[T], b: &[T]) -> Vec<T> {
    let mut out = a.to_vec();
    for item in b {
        if !out.contains(item) {
            out.push(item.clone());
        }
    }
    out
}

/// Lexically normalizes a path, resolving `.` and `..` components without
/// touching the filesystem. Leading `..` beyond the root is clamped, so a
/// path like `/tmp/../etc` normalizes to `/etc` and can never match a
/// grant at `/tmp`. Symbolic links are NOT resolved — enforcement on
/// untrusted paths must use [`CapabilityScope::matches_path_canonicalized`].
fn normalize_lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Strips the Windows verbatim prefix (`\\?\C:\...` / `\\?\UNC\...`) that
/// [`std::fs::canonicalize`] produces, so canonicalized candidates compare
/// equal to grants written as plain paths.
#[cfg(windows)]
fn strip_verbatim(p: &Path) -> PathBuf {
    let s = p.as_os_str().to_string_lossy();
    if let Some(rest) = s.strip_prefix("\\\\?\\UNC\\") {
        PathBuf::from(format!("\\\\{rest}"))
    } else if let Some(rest) = s.strip_prefix("\\\\?\\") {
        PathBuf::from(rest)
    } else {
        p.to_path_buf()
    }
}

/// True if the path is drive-relative (`C:foo`) — its meaning depends on
/// the per-drive current directory and cannot be matched safely. Such
/// paths are always rejected (fail closed).
#[cfg(windows)]
fn is_drive_relative(path: &Path) -> bool {
    matches!(
        path.components().next(),
        Some(Component::Prefix(prefix)) if {
            use std::path::Prefix::*;
            matches!(prefix.kind(), Disk(_))
        } && !path.is_absolute()
    )
}

/// Path comparison: byte-exact on Unix, case-insensitive on Windows where
/// the default filesystem is case-insensitive.
#[cfg(windows)]
fn os_eq(a: &std::ffi::OsStr, b: &std::ffi::OsStr) -> bool {
    a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
}

#[cfg(not(windows))]
fn os_eq(a: &std::ffi::OsStr, b: &std::ffi::OsStr) -> bool {
    a == b
}

/// Component-wise prefix-or-equality: `grant` matches `candidate` when every
/// component of `grant` equals the corresponding prefix of `candidate`.
/// Component-based, so a grant at `/tmp` covers `/tmp/sub` but never
/// `/tmpx`.
fn prefix_or_equal(candidate: &Path, grant: &Path) -> bool {
    let mut grant = grant.components();
    let mut cand = candidate.components();
    loop {
        match (grant.next(), cand.next()) {
            (None, _) => return true,
            (Some(g), Some(c)) => {
                if !os_eq(g.as_os_str(), c.as_os_str()) {
                    return false;
                }
            }
            (Some(_), None) => return false,
        }
    }
}

impl Capabilities {
    /// Returns the union of this capabilities set and another.
    ///
    /// The union contains all capabilities that exist in either set.
    /// ```rust
    /// use porpoise_core::types::Capabilities;
    /// let a = Capabilities::all_read("/tmp".into());
    /// let b = Capabilities::all_read_write("/tmp".into());
    /// let u = a.union(&b);
    /// assert!(u.fs_read.contains(&"/tmp".into()));
    /// assert!(u.fs_write.contains(&"/tmp".into()));
    /// ```
    pub fn union(&self, other: &Capabilities) -> Capabilities {
        Capabilities {
            fs_read: merged(&self.fs_read, &other.fs_read),
            fs_write: merged(&self.fs_write, &other.fs_write),
            network: merged(&self.network, &other.network),
            process: merged(&self.process, &other.process),
            ssh: merged(&self.ssh, &other.ssh),
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
    /// assert!(!diff.fs_read.contains(&"/tmp".into()));
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
        other.fs_read.iter().all(|p| self.fs_read.contains(p))
            && other.fs_write.iter().all(|p| self.fs_write.contains(p))
            && other.network.iter().all(|p| self.network.contains(p))
            && other.process.iter().all(|p| self.process.contains(p))
            && other.ssh.iter().all(|p| self.ssh.contains(p))
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
    /// assert!(caps.process.iter().any(|p| p.binary == "my-app"));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_deduplicates_entries() {
        let a = Capabilities::all_read("/tmp".into());
        let b = Capabilities::all_read_write("/tmp".into());
        let u = a.union(&b);
        assert_eq!(u.fs_read.len(), 1);
        assert_eq!(u.fs_write.len(), 1);
    }

    #[test]
    fn matches_path_rejects_traversal() {
        let scope = CapabilityScope::Filesystem {
            paths: vec!["/tmp".into()],
            write: false,
        };
        assert!(scope.matches_path(std::path::Path::new("/tmp")));
        assert!(scope.matches_path(std::path::Path::new("/tmp/./x/..")));
        assert!(!scope.matches_path(std::path::Path::new("/tmp/../etc")));
        assert!(!scope.matches_path(std::path::Path::new("/etc")));
    }

    #[test]
    fn matches_path_is_prefix_not_string_prefix() {
        let scope = CapabilityScope::Filesystem {
            paths: vec!["/tmp".into()],
            write: false,
        };
        // Inside the grant: children at any depth match.
        assert!(scope.matches_path(std::path::Path::new("/tmp/sub/file.rs")));
        assert!(scope.matches_path(std::path::Path::new("/tmp/a/b/c/d")));
        // Siblings whose name extends the grant must NOT match.
        assert!(!scope.matches_path(std::path::Path::new("/tmpx")));
        // But a directory literally named "sub.." IS inside the grant.
        assert!(scope.matches_path(std::path::Path::new("/tmp/sub..")));
        // Traversal out then into a sibling is still outside.
        assert!(!scope.matches_path(std::path::Path::new("/tmp/../tmpx/secret")));
    }

    #[cfg(windows)]
    #[test]
    fn matches_path_windows_is_case_insensitive() {
        let scope = CapabilityScope::Filesystem {
            paths: vec!["C:\\Work".into()],
            write: false,
        };
        assert!(scope.matches_path(std::path::Path::new("c:\\work\\proj\\main.rs")));
        assert!(scope.matches_path(std::path::Path::new("C:\\WORK")));
        assert!(!scope.matches_path(std::path::Path::new("c:\\workx")));
    }

    #[cfg(windows)]
    #[test]
    fn matches_path_rejects_drive_relative_paths() {
        // "C:foo" resolves against the per-drive current directory — its
        // meaning is process state, so it must never match a grant.
        let scope = CapabilityScope::Filesystem {
            paths: vec!["C:\\Work".into()],
            write: false,
        };
        assert!(!scope.matches_path(std::path::Path::new("C:foo.txt")));
        assert!(!scope.matches_path(std::path::Path::new("C:Work\\project")));
        assert!(scope.matches_path(std::path::Path::new("C:\\Work\\project")));
    }

    #[test]
    fn matches_path_canonicalized_rejects_unverifiable_paths() {
        let scope = CapabilityScope::Filesystem {
            paths: vec!["/tmp".into()],
            write: false,
        };
        // A path that does not exist cannot be verified — fail closed.
        assert!(!scope.matches_path_canonicalized(std::path::Path::new("/definitely/not/here/x")));
    }

    #[test]
    fn matches_path_canonicalized_resolves_real_dir() {
        let dir = tempfile::tempdir().unwrap();
        let scope = CapabilityScope::Filesystem {
            paths: vec![dir.path().to_path_buf()],
            write: false,
        };
        let inner = dir.path().join("sub");
        std::fs::create_dir_all(&inner).unwrap();
        std::fs::write(inner.join("f.txt"), b"x").unwrap();
        assert!(scope.matches_path_canonicalized(&inner));
        assert!(scope.matches_path_canonicalized(&inner.join("f.txt")));
        // Outside the grant, even when close.
        let outside = dir.path().parent().unwrap().to_path_buf();
        if outside != dir.path() {
            assert!(!scope.matches_path_canonicalized(&outside));
        }
    }

    #[cfg(unix)]
    #[test]
    fn canonicalization_catches_symlink_escape() {
        let dir = tempfile::tempdir().unwrap();
        let grant_dir = dir.path().join("grant");
        let outside_dir = dir.path().join("outside");
        std::fs::create_dir_all(&grant_dir).unwrap();
        std::fs::create_dir_all(&outside_dir).unwrap();
        let link = grant_dir.join("escape");
        std::os::unix::fs::symlink(&outside_dir, &link).unwrap();

        let scope = CapabilityScope::Filesystem {
            paths: vec![grant_dir],
            write: false,
        };
        // Lexical check is blind to the symlink (documented limitation)…
        assert!(scope.matches_path(&link));
        // …canonicalization resolves it and must reject the escape.
        assert!(!scope.matches_path_canonicalized(&link));
        assert!(!scope.matches_path_canonicalized(&link.join("file")));
    }

    #[cfg(unix)]
    #[test]
    fn canonicalization_allows_symlink_staying_inside_grant() {
        let dir = tempfile::tempdir().unwrap();
        let grant_dir = dir.path().join("grant");
        std::fs::create_dir_all(grant_dir.join("real")).unwrap();
        let link = grant_dir.join("alias");
        std::os::unix::fs::symlink(grant_dir.join("real"), &link).unwrap();

        let scope = CapabilityScope::Filesystem {
            paths: vec![grant_dir],
            write: false,
        };
        assert!(scope.matches_path_canonicalized(&link));
    }

    #[cfg(windows)]
    #[test]
    fn canonicalized_candidates_survive_verbatim_stripping() {
        // std::fs::canonicalize returns \\?\C:\... verbatim paths on
        // Windows; a plain-path grant must still match.
        let dir = tempfile::tempdir().unwrap();
        let scope = CapabilityScope::Filesystem {
            paths: vec![dir.path().to_path_buf()],
            write: false,
        };
        let probe = dir.path().join("probe.txt");
        std::fs::write(&probe, b"x").unwrap();
        let canonical = std::fs::canonicalize(&probe).unwrap();
        assert!(canonical.as_os_str().to_string_lossy().starts_with("\\\\?\\"));
        assert!(scope.matches_path_canonicalized(&probe));
    }

    #[test]
    fn matches_url_ignores_trailing_slash() {
        let scope = CapabilityScope::Network {
            urls: vec!["http://localhost:3000/api".to_string()],
        };
        assert!(scope.matches_url("http://localhost:3000/api"));
        assert!(scope.matches_url("http://localhost:3000/api/"));
        let slash_grant = CapabilityScope::Network {
            urls: vec!["http://localhost:3000/api/".to_string()],
        };
        assert!(slash_grant.matches_url("http://localhost:3000/api"));
        // Still exact otherwise: path and host are not interchangeable.
        assert!(!scope.matches_url("http://localhost:3000"));
        assert!(!scope.matches_url("http://localhost:3000/api/v2"));
    }

    #[test]
    fn normalize_lexical_clamps_runaway_parent_dirs() {
        assert_eq!(
            normalize_lexical(std::path::Path::new("/tmp/../../etc")),
            std::path::PathBuf::from("/etc")
        );
        assert_eq!(
            normalize_lexical(std::path::Path::new("a/./b/../c")),
            std::path::PathBuf::from("a/c")
        );
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
    /// Matching is **component-wise prefix or equality** on lexically
    /// normalized paths: a grant at `/tmp` covers `/tmp/sub/file`, never
    /// `/tmpx` (sibling) and never `/tmp/../etc` (traversal clamps outside
    /// the grant). Comparison is case-insensitive on Windows. Drive-relative
    /// candidates (`C:foo`) are always rejected — their meaning depends on
    /// the per-drive current directory.
    ///
    /// This function does NOT resolve symlinks. Enforcing on untrusted
    /// paths requires [`Self::matches_path_canonicalized`].
    /// ```rust
    /// use porpoise_core::types::CapabilityScope;
    /// use std::path::Path;
    /// let scope = CapabilityScope::Filesystem { paths: vec!["/tmp".into()], write: false };
    /// assert!(scope.matches_path(Path::new("/tmp")));
    /// assert!(scope.matches_path(Path::new("/tmp/./x/..")));
    /// assert!(scope.matches_path(Path::new("/tmp/sub/file")));
    /// assert!(!scope.matches_path(Path::new("/tmp/../etc")));
    /// assert!(!scope.matches_path(Path::new("/tmpx")));
    /// ```
    pub fn matches_path(&self, path: &Path) -> bool {
        match self {
            CapabilityScope::Filesystem { paths, write: _ } => {
                #[cfg(windows)]
                {
                    if is_drive_relative(path) {
                        return false;
                    }
                }
                let candidate = normalize_lexical(path);
                paths.iter().any(|grant| {
                    #[cfg(windows)]
                    let grant = strip_verbatim(&normalize_lexical(grant));
                    #[cfg(not(windows))]
                    let grant = normalize_lexical(grant);
                    prefix_or_equal(&candidate, &grant)
                })
            }
            CapabilityScope::Network { urls: _ } => false,
            CapabilityScope::Process { allow: _ } => false,
            CapabilityScope::Ssh { hosts: _ } => false,
            CapabilityScope::All => true,
            CapabilityScope::None => false,
        }
    }

    /// Enforcement-boundary check: canonicalizes the candidate (resolving
    /// symlinks and relative segments against the real filesystem) and then
    /// applies [`Self::matches_path`]. A path that cannot be canonicalized
    /// (missing, permission denied) is **rejected** — fail closed. Windows
    /// verbatim prefixes (`\\?\`) are stripped so canonicalized candidates
    /// compare equal to plain-path grants.
    ///
    /// Grants should be created from canonical paths (e.g. the output of
    /// [`std::fs::canonicalize`] at grant time); a grant that itself points
    /// at a symlinked location is matched lexically here, not resolved.
    /// ```rust
    /// use porpoise_core::types::CapabilityScope;
    /// use std::path::Path;
    /// let scope = CapabilityScope::Filesystem { paths: vec!["/tmp".into()], write: false };
    /// assert!(!scope.matches_path_canonicalized(Path::new("/definitely/not/here")));
    /// ```
    pub fn matches_path_canonicalized(&self, path: &Path) -> bool {
        match std::fs::canonicalize(path) {
            Ok(real) => {
                #[cfg(windows)]
                let real = strip_verbatim(&real);
                self.matches_path(&real)
            }
            Err(_) => false,
        }
    }

    /// Returns `true` if this scope matches the given URL string.
    ///
    /// Trailing slashes are insignificant: a grant for `http://h:1/api`
    /// matches `http://h:1/api/` and vice versa. Everything else is exact.
    /// ```rust
    /// use porpoise_core::types::CapabilityScope;
    /// let scope = CapabilityScope::Network { urls: vec!["http://localhost:3000".to_string()] };
    /// assert!(scope.matches_url("http://localhost:3000"));
    /// assert!(scope.matches_url("http://localhost:3000/"));
    /// assert!(!scope.matches_url("http://example.com"));
    /// ```
    pub fn matches_url(&self, url: &str) -> bool {
        fn trim_slash(u: &str) -> &str {
            u.strip_suffix('/').unwrap_or(u)
        }
        let candidate = trim_slash(url);
        match self {
            CapabilityScope::Network { urls } => urls.iter().any(|u| trim_slash(u) == candidate),
            CapabilityScope::Filesystem { paths: _, write: _ } => false,
            CapabilityScope::Process { allow: _ } => false,
            CapabilityScope::Ssh { hosts: _ } => false,
            CapabilityScope::All => true,
            CapabilityScope::None => false,
        }
    }
}
