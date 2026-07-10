use std::path::Path;

use git2::Repository;
use porpoise_core::error::{PorpoiseError, Result};

use crate::types::WorktreeInfo;

pub struct WorktreeManager;

impl WorktreeManager {
    pub fn create(repo_path: &Path, name: &str, target_path: &Path) -> Result<WorktreeInfo> {
        let repo = Repository::open(repo_path).map_err(|e| PorpoiseError::Git(format!("open repo: {e}")))?;
        let opts = git2::WorktreeAddOptions::new();
        repo.worktree(name, target_path, Some(&opts))
            .map_err(|e| PorpoiseError::Git(format!("create worktree '{name}': {e}")))?;
        let branch = repo
            .head()
            .ok()
            .and_then(|h| h.shorthand().map(String::from))
            .unwrap_or_else(|| "unknown".to_string());
        Ok(WorktreeInfo {
            name: name.to_string(),
            path: target_path.to_path_buf(),
            branch,
            is_bare: false,
        })
    }

    pub fn list(repo_path: &Path) -> Result<Vec<WorktreeInfo>> {
        let repo = Repository::open(repo_path).map_err(|e| PorpoiseError::Git(format!("open repo: {e}")))?;
        let names = repo
            .worktrees()
            .map_err(|e| PorpoiseError::Git(format!("list worktrees: {e}")))?;
        let mut result = Vec::new();
        for name in names.iter().flatten() {
            if let Ok(worktree) = repo.find_worktree(name) {
                let branch = worktree.name().unwrap_or("unknown").to_string();
                let path = worktree.path().to_path_buf();
                result.push(WorktreeInfo {
                    name: name.to_string(),
                    path,
                    branch,
                    is_bare: false,
                });
            }
        }
        Ok(result)
    }

    pub fn remove(repo_path: &Path, name: &str) -> Result<()> {
        let repo = Repository::open(repo_path).map_err(|e| PorpoiseError::Git(format!("open repo: {e}")))?;
        let worktree = repo
            .find_worktree(name)
            .map_err(|e| PorpoiseError::Git(format!("find worktree '{name}': {e}")))?;
        let mut opts = git2::WorktreePruneOptions::new();
        opts.valid(true);
        worktree
            .prune(Some(&mut opts))
            .map_err(|e| PorpoiseError::Git(format!("prune worktree '{name}': {e}")))?;
        Ok(())
    }

    pub fn prune(repo_path: &Path) -> Result<()> {
        let repo = Repository::open(repo_path).map_err(|e| PorpoiseError::Git(format!("open repo: {e}")))?;
        let names = repo
            .worktrees()
            .map_err(|e| PorpoiseError::Git(format!("list worktrees: {e}")))?;
        for name in names.iter().flatten() {
            if let Ok(worktree) = repo.find_worktree(name) {
                let _opts = git2::WorktreeAddOptions::new();
                if !worktree.path().exists() {
                    let mut p_opts = git2::WorktreePruneOptions::new();
                    p_opts.valid(true);
                    worktree.prune(Some(&mut p_opts)).ok();
                }
            }
        }
        Ok(())
    }

    pub fn prune_all(repo_path: &Path) -> Result<()> {
        let repo = Repository::open(repo_path).map_err(|e| PorpoiseError::Git(format!("open repo: {e}")))?;
        let names = repo
            .worktrees()
            .map_err(|e| PorpoiseError::Git(format!("list worktrees: {e}")))?;
        for name in names.iter().flatten() {
            if let Ok(worktree) = repo.find_worktree(name) {
                let mut opts = git2::WorktreePruneOptions::new();
                opts.valid(true);
                worktree.prune(Some(&mut opts)).ok();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_list_empty_repo() {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "test").ok();
        config.set_str("user.email", "test@test.com").ok();
        let result = WorktreeManager::list(dir.path());
        assert!(result.is_ok());
    }
}
