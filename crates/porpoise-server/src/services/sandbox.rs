//! MicroVM sandbox backend — implementation of `SANDBOX_SPEC.md` v1.
//!
//! Owns the per-project sandbox lifecycle (provision / start / stop /
//! clean) with the spec's file layout under the server data dir:
//!
//! ```text
//! <data_dir>/sandboxes/<id>/workspace/     host project, mounted at /workspace in the guest
//! <data_dir>/sandboxes/<id>/vm.toml        backend="microvm", state, cpus=1, memory_mb=512
//! <data_dir>/sandboxes/<id>/microvm.json   base image, overlay, kernel, network policy, toolchain mount
//! <data_dir>/sandboxes/<id>/overlay.qcow2  per-sandbox delta — created at START, never at provision
//! <data_dir>/sandboxes/<id>/microvm.pid    QEMU pid file while running
//! <data_dir>/sandboxes/images/             shared agent-base.qcow2 + vmlinux
//! ```
//!
//! Nothing is faked, per the spec: [`MicroVmManager::start`] fails with
//! the exact list of missing prerequisites (KVM, QEMU binary, base image,
//! guest kernel) instead of pretending to boot.

use std::path::{Path, PathBuf};

use porpoise_core::error::{PorpoiseError, Result};
use serde::{Deserialize, Serialize};

/// Detection result for one sandbox backend: whether it can run on this
/// host, and the exact missing prerequisites when it cannot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendStatus {
    pub name: String,
    pub available: bool,
    pub missing: Vec<String>,
}

/// Per-project microVM configuration persisted as `vm.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MicroVmConfig {
    pub backend: String,
    pub state: String,
    pub cpus: u32,
    pub memory_mb: u32,
    pub workspace: PathBuf,
}

impl Default for MicroVmConfig {
    fn default() -> Self {
        Self {
            backend: "microvm".to_string(),
            state: "stopped".to_string(),
            cpus: 1,
            memory_mb: 512,
            workspace: PathBuf::new(),
        }
    }
}

/// Connection/mount metadata persisted as `microvm.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MicroVmJson {
    pub base_image: PathBuf,
    pub overlay: PathBuf,
    pub kernel: PathBuf,
    pub network_policy: String,
    pub toolchain_mount: String,
}

/// One provisioned microVM sandbox as shown by `list()` / `status()`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicroVmView {
    pub id: String,
    pub state: String,
    pub cpus: u32,
    pub memory_mb: u32,
}

/// Searches the host PATH for an executable (no `which` dependency).
fn find_on_path(binary: &str) -> bool {
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(binary);
        if candidate.is_file() {
            return true;
        }
        // Windows PATHEXT: qemu-system-x86_64.EXE resolves via .exe.
        #[cfg(windows)]
        if candidate.with_extension("exe").is_file() {
            return true;
        }
    }
    false
}

fn config_err(msg: impl Into<String>) -> PorpoiseError {
    PorpoiseError::Config(msg.into())
}

/// Per-project microVM sandbox lifecycle. See the module docs for the
/// file layout. All expensive checks (PATH scans, fs existence) run
/// synchronously — handlers on the relay already run on worker tasks,
/// and every lifecycle call is a short bounded operation.
pub struct MicroVmManager {
    root: PathBuf,
}

impl MicroVmManager {
    /// Manager rooted at `<data_dir>/sandboxes`.
    pub fn with_data_dir(data_dir: &Path) -> Self {
        Self {
            root: data_dir.join("sandboxes"),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn project_dir(&self, id: &str) -> PathBuf {
        self.root.join(id)
    }

    fn vm_toml_path(&self, id: &str) -> PathBuf {
        self.project_dir(id).join("vm.toml")
    }

    fn microvm_json_path(&self, id: &str) -> PathBuf {
        self.project_dir(id).join("microvm.json")
    }

    fn overlay_path(&self, id: &str) -> PathBuf {
        self.project_dir(id).join("overlay.qcow2")
    }

    fn pid_path(&self, id: &str) -> PathBuf {
        self.project_dir(id).join("microvm.pid")
    }

    fn base_image_path(&self) -> PathBuf {
        self.root.join("images").join("agent-base.qcow2")
    }

    fn kernel_path(&self) -> PathBuf {
        self.root.join("images").join("vmlinux")
    }

    fn validate_id(id: &str) -> Result<()> {
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err(config_err(format!(
                "invalid sandbox id '{id}': use letters, digits, '-' or '_'"
            )));
        }
        Ok(())
    }

    /// Exact list of missing prerequisites for booting a microVM on this
    /// host. Empty means the execution path can run.
    pub fn missing_prerequisites(&self) -> Vec<String> {
        let mut missing = Vec::new();
        #[cfg(unix)]
        if !Path::new("/dev/kvm").exists() {
            missing.push("KVM accelerator (/dev/kvm)".to_string());
        }
        #[cfg(windows)]
        missing.push("KVM accelerator (/dev/kvm) — Linux hosts only".to_string());
        if !find_on_path("qemu-system-x86_64") {
            missing.push("QEMU binary (qemu-system-x86_64 on PATH)".to_string());
        }
        let base = self.base_image_path();
        if !base.exists() {
            missing.push(format!("base image ({})", base.display()));
        }
        let kernel = self.kernel_path();
        if !kernel.exists() {
            missing.push(format!("guest kernel ({})", kernel.display()));
        }
        missing
    }

    /// Provision a sandbox: workspace + `vm.toml` + `microvm.json`. No
    /// hypervisor is required at provision time, so this works on any
    /// host. The overlay is NOT created here — it is created at start.
    /// `workspace_source` optionally seeds the workspace (shallow copy).
    pub fn provision(&self, id: &str, workspace_source: Option<&Path>) -> Result<MicroVmConfig> {
        Self::validate_id(id)?;
        let dir = self.project_dir(id);
        if dir.exists() {
            return Err(config_err(format!(
                "sandbox '{id}' already exists at {}",
                dir.display()
            )));
        }
        let ws = dir.join("workspace");
        std::fs::create_dir_all(&ws).map_err(|e| config_err(format!("create workspace {}: {e}", ws.display())))?;
        if let Some(src) = workspace_source
            && src.is_dir()
        {
            for entry in std::fs::read_dir(src).map_err(|e| config_err(format!("read {}: {e}", src.display())))? {
                let entry = entry.map_err(|e| config_err(format!("readdir: {e}")))?;
                if entry
                    .file_type()
                    .map_err(|e| config_err(format!("filetype: {e}")))?
                    .is_file()
                {
                    let dest = ws.join(entry.file_name());
                    std::fs::copy(entry.path(), &dest)
                        .map_err(|e| config_err(format!("copy {}: {e}", entry.path().display())))?;
                }
            }
        }

        let config = MicroVmConfig {
            workspace: ws,
            ..Default::default()
        };
        std::fs::write(
            self.vm_toml_path(id),
            toml::to_string_pretty(&config).map_err(|e| config_err(format!("serialize vm.toml: {e}")))?,
        )
        .map_err(|e| config_err(format!("write vm.toml: {e}")))?;

        let json = MicroVmJson {
            base_image: self.base_image_path(),
            overlay: self.overlay_path(id),
            kernel: self.kernel_path(),
            network_policy: "deny".to_string(),
            toolchain_mount: "ro".to_string(),
        };
        std::fs::write(
            self.microvm_json_path(id),
            serde_json::to_string_pretty(&json).map_err(|e| config_err(format!("serialize microvm.json: {e}")))?,
        )
        .map_err(|e| config_err(format!("write microvm.json: {e}")))?;

        Ok(config)
    }

    fn load_config(&self, id: &str) -> Result<MicroVmConfig> {
        let raw = std::fs::read_to_string(self.vm_toml_path(id))
            .map_err(|_| config_err(format!("sandbox '{id}' is not provisioned")))?;
        toml::from_str(&raw).map_err(|e| config_err(format!("parse vm.toml: {e}")))
    }

    fn write_state(&self, id: &str, state: &str) -> Result<()> {
        let mut config = self.load_config(id)?;
        config.state = state.to_string();
        std::fs::write(
            self.vm_toml_path(id),
            toml::to_string_pretty(&config).map_err(|e| config_err(format!("serialize vm.toml: {e}")))?,
        )
        .map_err(|e| config_err(format!("write vm.toml: {e}")))?;
        Ok(())
    }

    /// Boot the microVM. Fails honestly — with every missing piece listed —
    /// unless KVM, the QEMU binary, the base image, and the kernel are all
    /// present. When they are, creates the overlay (if absent) and spawns
    /// QEMU `microvm` daemonised with a pid file.
    pub fn start(&self, id: &str) -> Result<u32> {
        Self::validate_id(id)?;
        let missing = self.missing_prerequisites();
        if !missing.is_empty() {
            return Err(config_err(format!(
                "microvm cannot start on this host; missing: {}",
                missing.join(", ")
            )));
        }
        let config = self.load_config(id)?;
        let overlay = self.overlay_path(id);
        if !overlay.exists() {
            let status = std::process::Command::new("qemu-img")
                .args([
                    "create",
                    "-f",
                    "qcow2",
                    "-b",
                    &self.base_image_path().display().to_string(),
                    "-F",
                    "qcow2",
                    &overlay.display().to_string(),
                ])
                .status()
                .map_err(|e| config_err(format!("spawn qemu-img: {e}")))?;
            if !status.success() {
                return Err(config_err(format!("qemu-img overlay creation failed ({status})")));
            }
        }
        let mut cmd = std::process::Command::new("qemu-system-x86_64");
        cmd.args([
            "-machine",
            "microvm",
            "-kernel",
            &self.kernel_path().display().to_string(),
            "-drive",
            &format!("file={},format=qcow2,if=none,id=root", overlay.display()),
            "-m",
            &format!("{}M", config.memory_mb),
            "-smp",
            &config.cpus.to_string(),
            "-daemonize",
            "-pidfile",
            &self.pid_path(id).display().to_string(),
        ]);
        let status = cmd
            .status()
            .map_err(|e| config_err(format!("spawn qemu-system-x86_64: {e}")))?;
        if !status.success() {
            return Err(config_err(format!("QEMU exited with {status}; VM not booted")));
        }
        let pid = std::fs::read_to_string(self.pid_path(id))
            .map_err(|e| config_err(format!("read pid file: {e}")))?
            .trim()
            .parse::<u32>()
            .map_err(|e| config_err(format!("parse pid file: {e}")))?;
        self.write_state(id, "running")?;
        Ok(pid)
    }

    /// Stop the sandbox: kill the pid-file process when one exists (unix
    /// `kill`, Windows `taskkill`), remove the pid file, mark stopped. A
    /// missing pid file is not an error — the VM is already down.
    pub fn stop(&self, id: &str) -> Result<bool> {
        Self::validate_id(id)?;
        let pid_path = self.pid_path(id);
        let mut killed = false;
        if let Ok(raw) = std::fs::read_to_string(&pid_path)
            && let Ok(pid) = raw.trim().parse::<u32>()
        {
            #[cfg(unix)]
            {
                killed = std::process::Command::new("kill")
                    .args(["-TERM", &pid.to_string()])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
            }
            #[cfg(windows)]
            {
                killed = std::process::Command::new("taskkill")
                    .args(["/PID".to_string(), pid.to_string(), "/F".to_string()])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
            }
        }
        let _ = std::fs::remove_file(&pid_path);
        if self.vm_toml_path(id).exists() {
            self.write_state(id, "stopped")?;
        }
        Ok(killed)
    }

    /// Return the sandbox to pristine base: delete overlay + pid file,
    /// keep workspace and config files.
    pub fn clean(&self, id: &str) -> Result<()> {
        Self::validate_id(id)?;
        let _ = std::fs::remove_file(self.overlay_path(id));
        let _ = std::fs::remove_file(self.pid_path(id));
        if self.vm_toml_path(id).exists() {
            self.write_state(id, "stopped")?;
        }
        Ok(())
    }

    /// Delete the whole sandbox directory (workspace included). Explicitly
    /// destructive — callers confirm.
    pub fn delete(&self, id: &str) -> Result<()> {
        Self::validate_id(id)?;
        let dir = self.project_dir(id);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(|e| config_err(format!("remove {}: {e}", dir.display())))?;
        }
        Ok(())
    }

    /// Current state of one sandbox; `None` when not provisioned.
    pub fn status(&self, id: &str) -> Option<MicroVmView> {
        let config = self.load_config(id).ok()?;
        Some(MicroVmView {
            id: id.to_string(),
            state: config.state,
            cpus: config.cpus,
            memory_mb: config.memory_mb,
        })
    }

    /// All provisioned sandboxes, sorted by id.
    pub fn list(&self) -> Result<Vec<MicroVmView>> {
        let mut out = Vec::new();
        if !self.root.exists() {
            return Ok(out);
        }
        let entries =
            std::fs::read_dir(&self.root).map_err(|e| config_err(format!("read {}: {e}", self.root.display())))?;
        for entry in entries {
            let entry = entry.map_err(|e| config_err(format!("readdir: {e}")))?;
            if entry
                .file_type()
                .map_err(|e| config_err(format!("filetype: {e}")))?
                .is_dir()
                && entry.file_name().to_string_lossy() != "images"
            {
                let id = entry.file_name().to_string_lossy().to_string();
                if let Some(view) = self.status(&id) {
                    out.push(view);
                }
            }
        }
        out.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(out)
    }
}

/// Detect sandbox backends on this host (SANDBOX_SPEC "Host view"): the
/// microvm execution path plus the container fallback. Process spawns are
/// avoided where an fs check suffices; `docker --version` is the one
/// exception.
pub fn detect_backends(data_dir: &Path) -> Vec<BackendStatus> {
    let mut out = Vec::new();

    let vm = MicroVmManager::with_data_dir(data_dir);
    let missing = vm.missing_prerequisites();
    out.push(if missing.is_empty() {
        BackendStatus {
            name: "microvm".to_string(),
            available: true,
            missing: Vec::new(),
        }
    } else {
        BackendStatus {
            name: "microvm".to_string(),
            available: false,
            missing,
        }
    });

    let docker_available = std::process::Command::new("docker")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    out.push(if docker_available {
        BackendStatus {
            name: "docker".to_string(),
            available: true,
            missing: Vec::new(),
        }
    } else {
        BackendStatus {
            name: "docker".to_string(),
            available: false,
            missing: vec!["docker not found on PATH or daemon not reachable".to_string()],
        }
    });

    out.push(BackendStatus {
        name: "process".to_string(),
        available: true,
        missing: Vec::new(),
    });

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manager() -> (tempfile::TempDir, MicroVmManager) {
        let dir = tempfile::tempdir().unwrap();
        let mgr = MicroVmManager::with_data_dir(dir.path());
        (dir, mgr)
    }

    #[test]
    fn provision_writes_vm_toml_and_microvm_json_without_overlay() {
        let (_dir, mgr) = manager();
        let config = mgr.provision("demo", None).unwrap();

        assert_eq!(config.backend, "microvm");
        assert_eq!(config.state, "stopped");
        assert_eq!(config.cpus, 1, "spec default: 1 vCPU");
        assert_eq!(config.memory_mb, 512, "spec default: 512 MB");

        let vm_toml = std::fs::read_to_string(mgr.vm_toml_path("demo")).unwrap();
        assert!(vm_toml.contains("backend = \"microvm\""));
        assert!(vm_toml.contains("cpus = 1"));
        assert!(vm_toml.contains("memory_mb = 512"));

        let json: MicroVmJson =
            serde_json::from_str(&std::fs::read_to_string(mgr.microvm_json_path("demo")).unwrap()).unwrap();
        assert_eq!(json.network_policy, "deny", "spec: default deny");
        assert_eq!(json.toolchain_mount, "ro", "spec: toolchain mounted ro");
        assert_eq!(json.overlay, mgr.overlay_path("demo"));

        // Overlay is created at start, never at provision (spec).
        assert!(!mgr.overlay_path("demo").exists());
        assert!(mgr.project_dir("demo").join("workspace").is_dir());
    }

    #[test]
    fn provision_rejects_bad_ids_and_duplicates() {
        let (_dir, mgr) = manager();
        assert!(mgr.provision("../escape", None).is_err());
        assert!(mgr.provision("", None).is_err());
        mgr.provision("demo", None).unwrap();
        assert!(mgr.provision("demo", None).is_err());
    }

    #[test]
    fn provision_copies_workspace_files() {
        let (_dir, mgr) = manager();
        let src = tempfile::tempdir().unwrap();
        std::fs::write(src.path().join("main.rs"), "fn main() {}\n").unwrap();
        mgr.provision("demo", Some(src.path())).unwrap();
        let copied = mgr.project_dir("demo").join("workspace").join("main.rs");
        assert_eq!(std::fs::read_to_string(copied).unwrap(), "fn main() {}\n");
    }

    #[test]
    fn start_fails_honestly_listing_missing_prerequisites() {
        // Imageless root: even on a KVM host with QEMU installed, base
        // image and kernel are missing — start must fail with the exact
        // items and never pretend to boot.
        let (_dir, mgr) = manager();
        mgr.provision("demo", None).unwrap();
        let err = mgr.start("demo").unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("missing"), "got: {msg}");
        assert!(msg.contains("base image"), "got: {msg}");
        assert!(msg.contains("guest kernel"), "got: {msg}");
        assert_eq!(mgr.status("demo").unwrap().state, "stopped");
    }

    #[test]
    fn detection_reports_microvm_with_missing_items_on_imageless_host() {
        let (dir, mgr) = manager();
        assert!(mgr.missing_prerequisites().iter().any(|m| m.contains("base image")));
        let backends = detect_backends(dir.path());
        assert!(backends.iter().any(|b| b.name == "microvm" && !b.available));
        assert!(backends.iter().any(|b| b.name == "process" && b.available));
    }

    #[test]
    fn clean_removes_overlay_and_pid_keeps_workspace_and_config() {
        let (_dir, mgr) = manager();
        mgr.provision("demo", None).unwrap();
        std::fs::write(mgr.overlay_path("demo"), "qcow2-bytes").unwrap();
        std::fs::write(mgr.pid_path("demo"), "12345").unwrap();

        mgr.clean("demo").unwrap();

        assert!(!mgr.overlay_path("demo").exists());
        assert!(!mgr.pid_path("demo").exists());
        assert!(mgr.project_dir("demo").join("workspace").is_dir());
        assert!(mgr.vm_toml_path("demo").exists());
        assert!(mgr.microvm_json_path("demo").exists());
        assert_eq!(mgr.status("demo").unwrap().state, "stopped");
    }

    #[test]
    fn stop_without_pid_file_is_not_an_error() {
        let (_dir, mgr) = manager();
        mgr.provision("demo", None).unwrap();
        assert!(!mgr.stop("demo").unwrap());
        assert_eq!(mgr.status("demo").unwrap().state, "stopped");
    }

    #[test]
    fn status_and_list_cover_provisioned_sandboxes_only() {
        let (_dir, mgr) = manager();
        assert!(mgr.list().unwrap().is_empty());
        assert!(mgr.status("nope").is_none());
        mgr.provision("b-second", None).unwrap();
        mgr.provision("a-first", None).unwrap();
        let list = mgr.list().unwrap();
        assert_eq!(
            list.iter().map(|v| v.id.as_str()).collect::<Vec<_>>(),
            vec!["a-first", "b-second"]
        );
    }

    #[test]
    fn delete_removes_everything() {
        let (_dir, mgr) = manager();
        mgr.provision("demo", None).unwrap();
        mgr.delete("demo").unwrap();
        assert!(!mgr.project_dir("demo").exists());
        assert!(mgr.list().unwrap().is_empty());
    }
}
