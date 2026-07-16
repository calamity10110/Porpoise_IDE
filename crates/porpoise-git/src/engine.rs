use std::path::Path;

use git2::{DiffOptions, Repository, StatusOptions};
use porpoise_core::error::{PorpoiseError, Result};

use crate::types::{BranchInfo, Change, ChangeStatus, CommitEntry};

pub struct GitEngine {
    pub(crate) repo_path: std::path::PathBuf,
    pub(crate) repo: Repository,
}

impl GitEngine {
    pub fn open(path: &Path) -> Result<Self> {
        let repo = Repository::open(path).map_err(|e| PorpoiseError::Git(format!("open repo: {e}")))?;
        Ok(Self {
            repo_path: path.to_path_buf(),
            repo,
        })
    }

    pub fn clone(url: &str, path: &Path) -> Result<Self> {
        let repo = Repository::clone(url, path).map_err(|e| PorpoiseError::Git(format!("clone {url}: {e}")))?;
        Ok(Self {
            repo_path: path.to_path_buf(),
            repo,
        })
    }

    pub fn init(path: &Path) -> Result<Self> {
        let repo = Repository::init(path).map_err(|e| PorpoiseError::Git(format!("init: {e}")))?;
        Ok(Self {
            repo_path: path.to_path_buf(),
            repo,
        })
    }

    pub fn status(&self) -> Result<Vec<Change>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_unmodified(false);
        let statuses = self
            .repo
            .statuses(Some(&mut opts))
            .map_err(|e| PorpoiseError::Git(format!("status: {e}")))?;

        let mut changes = Vec::new();
        for entry in statuses.iter() {
            let path = entry.path().map_or_else(|| Path::new(""), Path::new);
            let git_status = entry.status();

            let status = if git_status.is_index_new() || git_status.is_wt_new() {
                ChangeStatus::New
            } else if git_status.is_index_modified() || git_status.is_wt_modified() {
                ChangeStatus::Modified
            } else if git_status.is_index_deleted() || git_status.is_wt_deleted() {
                ChangeStatus::Deleted
            } else if git_status.is_index_renamed() || git_status.is_wt_renamed() {
                ChangeStatus::Renamed
            } else if git_status.is_index_typechange() || git_status.is_wt_typechange() {
                ChangeStatus::TypeChange
            } else if git_status.is_conflicted() {
                ChangeStatus::Conflict
            } else {
                continue;
            };

            changes.push(Change {
                path: path.to_path_buf(),
                status,
            });
        }
        Ok(changes)
    }

    pub fn diff(&self, staged: bool) -> Result<String> {
        if staged {
            let tree = self.repo.head().ok().and_then(|h| h.peel_to_tree().ok());
            let mut opts = DiffOptions::new();
            let diff = self
                .repo
                .diff_tree_to_index(tree.as_ref(), None, Some(&mut opts))
                .map_err(|e| PorpoiseError::Git(format!("diff staged: {e}")))?;
            Self::format_diff(&diff)
        } else {
            let mut opts = DiffOptions::new();
            let diff = self
                .repo
                .diff_tree_to_workdir(None, Some(&mut opts))
                .map_err(|e| PorpoiseError::Git(format!("diff workdir: {e}")))?;
            Self::format_diff(&diff)
        }
    }

    fn format_diff(diff: &git2::Diff) -> Result<String> {
        let mut out = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let prefix = match line.origin() {
                '+' => "+",
                '-' => "-",
                ' ' => " ",
                _ => return true,
            };
            if let Ok(content) = std::str::from_utf8(line.content()) {
                out.push_str(prefix);
                out.push_str(content);
            }
            true
        })
        .map_err(|e| PorpoiseError::Git(format!("diff print: {e}")))?;
        Ok(out)
    }

    pub fn log(&self, count: usize) -> Result<Vec<CommitEntry>> {
        let mut revwalk = self
            .repo
            .revwalk()
            .map_err(|e| PorpoiseError::Git(format!("revwalk: {e}")))?;
        revwalk
            .push_head()
            .map_err(|e| PorpoiseError::Git(format!("push head: {e}")))?;
        revwalk
            .set_sorting(git2::Sort::TIME)
            .map_err(|e| PorpoiseError::Git(format!("sort: {e}")))?;

        let mut entries = Vec::new();
        for (i, oid) in revwalk.enumerate() {
            if i >= count {
                break;
            }
            let oid = oid.map_err(|e| PorpoiseError::Git(format!("oid: {e}")))?;
            let commit = self
                .repo
                .find_commit(oid)
                .map_err(|e| PorpoiseError::Git(format!("find commit: {e}")))?;
            entries.push(CommitEntry {
                id: oid.to_string(),
                author: commit.author().name().unwrap_or("unknown").to_string(),
                message: commit.message().unwrap_or("").trim().to_string(),
                timestamp: commit.time().seconds(),
            });
        }
        Ok(entries)
    }

    pub fn branch_list(&self) -> Result<Vec<BranchInfo>> {
        let mut branches = Vec::new();
        let head = self.repo.head().ok().map(|h| h.shorthand().unwrap_or("").to_string());
        self.repo
            .branches(None)
            .map_err(|e| PorpoiseError::Git(format!("branches: {e}")))?
            .filter_map(|b| b.ok())
            .for_each(|(branch, _kind)| {
                let name = branch.name().ok().flatten().unwrap_or("unknown").to_string();
                let is_head = head.as_deref() == Some(&name);
                branches.push(BranchInfo { name, is_head });
            });
        Ok(branches)
    }

    pub fn branch_create(&self, name: &str) -> Result<()> {
        let head_commit = self
            .repo
            .head()
            .map_err(|e| PorpoiseError::Git(format!("head: {e}")))?
            .peel_to_commit()
            .map_err(|e| PorpoiseError::Git(format!("peel: {e}")))?;
        self.repo
            .branch(name, &head_commit, false)
            .map_err(|e| PorpoiseError::Git(format!("create branch '{name}': {e}")))?;
        Ok(())
    }

    pub fn branch_checkout(&self, name: &str) -> Result<()> {
        let obj = self
            .repo
            .revparse_single(name)
            .map_err(|_| PorpoiseError::GitNoSuchBranch(name.into()))?;
        self.repo
            .checkout_tree(&obj, None)
            .map_err(|e| PorpoiseError::Git(format!("checkout tree: {e}")))?;
        self.repo
            .set_head(&format!("refs/heads/{name}"))
            .map_err(|e| PorpoiseError::Git(format!("set head: {e}")))?;
        Ok(())
    }

    pub fn fetch(&self, remote: &str) -> Result<()> {
        let mut rm = self
            .repo
            .find_remote(remote)
            .map_err(|e| PorpoiseError::Git(format!("remote '{remote}': {e}")))?;
        rm.fetch(&[] as &[&str], None, None as Option<&str>)
            .map_err(|e| PorpoiseError::Git(format!("fetch: {e}")))?;
        Ok(())
    }

    pub fn push(&self, remote: &str, branch: &str) -> Result<()> {
        let mut rm = self
            .repo
            .find_remote(remote)
            .map_err(|e| PorpoiseError::Git(format!("remote '{remote}': {e}")))?;
        let refspec = format!("refs/heads/{branch}:refs/heads/{branch}");
        rm.push(&[&refspec], None)
            .map_err(|e| PorpoiseError::Git(format!("push: {e}")))?;
        Ok(())
    }

    pub fn stash_save(&self, message: Option<&str>) -> Result<()> {
        let msg = message.unwrap_or("porpoise stash");
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(&self.repo_path)
            .args(["stash", "push", "-m", msg])
            .output()
            .map_err(|e| PorpoiseError::Git(format!("git stash: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("No local changes") {
                return Err(PorpoiseError::Git(format!("git stash: {stderr}")));
            }
        }
        Ok(())
    }

    pub fn stash_pop(&self) -> Result<()> {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(&self.repo_path)
            .args(["stash", "pop"])
            .output()
            .map_err(|e| PorpoiseError::Git(format!("git stash pop: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("No stash entries found") {
                return Err(PorpoiseError::Git(format!("git stash pop: {stderr}")));
            }
        }
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.repo_path
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_init_and_open() {
        let dir = TempDir::new().unwrap();
        let engine = GitEngine::init(dir.path()).unwrap();
        let opened = GitEngine::open(dir.path()).unwrap();
        assert_eq!(engine.path(), opened.path());
    }

    #[test]
    fn test_branch_list_empty() {
        let dir = TempDir::new().unwrap();
        let engine = GitEngine::init(dir.path()).unwrap();
        // A freshly init'd repo may or may not have a branch depending on git version/config
        let branches = engine.branch_list().unwrap_or_default();
        assert!(!branches.iter().any(|b| b.is_head), "new repo has no checked-out branches");
    }

    #[test]
    fn test_clone_invalid_url_fails() {
        let dir = TempDir::new().unwrap();
        let result = GitEngine::clone("https://invalid.url/repo.git", dir.path());
        assert!(result.is_err());
    }
}
