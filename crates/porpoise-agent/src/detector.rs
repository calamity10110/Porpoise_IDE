use std::env;
use std::process::Command;
use crate::types::{AgentKind, AgentManifest};

pub struct AgentDetector;

impl AgentDetector {
    pub fn detect_all() -> Vec<AgentManifest> {
        let agents = vec![
            ("Claude Code", "claude", AgentKind::ClaudeCode, &["--version"] as &[&str]),
            ("OpenAI Codex", "codex", AgentKind::Codex, &["--version"]),
            ("Google Gemini", "gemini", AgentKind::Gemini, &["--version"]),
        ];

        agents.into_iter().map(|(name, binary, kind, version_args)| {
            let detected = Self::binary_in_path(binary);
            let version = if detected {
                Self::get_version(binary, version_args)
            } else {
                None
            };
            AgentManifest { name: name.to_string(), binary: binary.to_string(), kind, detected, version }
        }).collect()
    }

    pub fn binary_in_path(binary: &str) -> bool {
        env::var_os("PATH")
            .and_then(|path| {
                env::split_paths(&path).find_map(|dir| {
                    let full = dir.join(binary);
                    if full.is_file() { Some(true) } else {
                        let full_exe = dir.join(format!("{}.exe", binary));
                        if full_exe.is_file() { Some(true) } else { None }
                    }
                })
            })
            .is_some()
    }

    fn get_version(binary: &str, args: &[&str]) -> Option<String> {
        Command::new(binary).args(args).output().ok()
            .and_then(|out| {
                if out.status.success() {
                    String::from_utf8(out.stdout).ok()
                        .or_else(|| String::from_utf8(out.stderr).ok())
                } else {
                    None
                }
            })
            .map(|s| s.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_all() {
        let manifests = AgentDetector::detect_all();
        assert!(manifests.len() == 3);
        for m in &manifests {
            assert!(!m.name.is_empty());
            assert!(!m.binary.is_empty());
        }
    }
}
