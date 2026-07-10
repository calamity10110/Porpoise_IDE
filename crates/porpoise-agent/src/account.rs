use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use porpoise_core::error::{PorpoiseError, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::types::AgentKind;

/// A named account for an agent provider (e.g., personal vs work GitHub).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAccount {
    pub id: String,
    pub name: String,
    pub agent_kind: String,
    pub api_key_env: Option<String>,
    pub config_path: Option<PathBuf>,
    pub is_default: bool,
}

/// Manages multiple accounts per agent kind for switching.
pub struct AccountSwitcher {
    accounts: Arc<RwLock<HashMap<String, AgentAccount>>>,
    active: Arc<RwLock<HashMap<String, String>>>,
    storage_path: PathBuf,
}

impl AccountSwitcher {
    pub fn new(storage_dir: &Path) -> Result<Self> {
        let storage_path = storage_dir.join("accounts.json");
        let accounts = Self::load_accounts(&storage_path)?;
        Ok(Self {
            accounts: Arc::new(RwLock::new(accounts)),
            active: Arc::new(RwLock::new(HashMap::new())),
            storage_path,
        })
    }

    pub async fn add_account(&self, account: AgentAccount) -> Result<()> {
        let mut accounts = self.accounts.write().await;
        if account.is_default {
            for a in accounts.values_mut() {
                if a.agent_kind == account.agent_kind {
                    a.is_default = false;
                }
            }
        }
        accounts.insert(account.id.clone(), account);
        self.save_accounts(&accounts)
    }

    pub async fn remove_account(&self, id: &str) -> Result<()> {
        let mut accounts = self.accounts.write().await;
        accounts.remove(id);
        self.save_accounts(&accounts)
    }

    pub async fn list_accounts(&self) -> Vec<AgentAccount> {
        self.accounts.read().await.values().cloned().collect()
    }

    pub async fn list_for_kind(&self, kind: &AgentKind) -> Vec<AgentAccount> {
        let kind_str = kind.to_string();
        self.accounts
            .read()
            .await
            .values()
            .filter(|a| a.agent_kind == kind_str)
            .cloned()
            .collect()
    }

    pub async fn switch(&self, kind: &AgentKind, account_id: &str) -> Result<AgentAccount> {
        let accounts = self.accounts.read().await;
        let account = accounts
            .get(account_id)
            .ok_or_else(|| PorpoiseError::Agent(format!("account '{account_id}' not found")))?;

        if account.agent_kind != kind.to_string() {
            return Err(PorpoiseError::Agent(format!(
                "account '{}' is for kind '{}', not '{}'",
                account_id, account.agent_kind, kind
            )));
        }

        self.active
            .write()
            .await
            .insert(kind.to_string(), account_id.to_string());

        Ok(account.clone())
    }

    pub async fn get_active(&self, kind: &AgentKind) -> Option<AgentAccount> {
        let active = self.active.read().await;
        let kind_str = kind.to_string();
        let id = active.get(&kind_str).cloned();
        if let Some(id) = id {
            self.accounts.read().await.get(&id).cloned()
        } else {
            None
        }
    }

    pub async fn get_default(&self, kind: &AgentKind) -> Option<AgentAccount> {
        let kind_str = kind.to_string();
        self.accounts
            .read()
            .await
            .values()
            .find(|a| a.agent_kind == kind_str && a.is_default)
            .cloned()
    }

    pub async fn get_or_default(&self, kind: &AgentKind) -> Option<AgentAccount> {
        if let Some(acc) = self.get_active(kind).await {
            Some(acc)
        } else {
            self.get_default(kind).await
        }
    }

    fn load_accounts(path: &Path) -> Result<HashMap<String, AgentAccount>> {
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let content = std::fs::read_to_string(path).map_err(|e| PorpoiseError::Agent(format!("read accounts: {e}")))?;
        let accounts: Vec<AgentAccount> =
            serde_json::from_str(&content).map_err(|e| PorpoiseError::Agent(format!("parse accounts: {e}")))?;
        Ok(accounts.into_iter().map(|a| (a.id.clone(), a)).collect())
    }

    fn save_accounts(&self, accounts: &HashMap<String, AgentAccount>) -> Result<()> {
        if let Some(parent) = self.storage_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| PorpoiseError::Agent(format!("create accounts dir: {e}")))?;
        }
        let list: Vec<&AgentAccount> = accounts.values().collect();
        let json = serde_json::to_string_pretty(&list)
            .map_err(|e| PorpoiseError::Agent(format!("serialize accounts: {e}")))?;
        std::fs::write(&self.storage_path, json).map_err(|e| PorpoiseError::Agent(format!("write accounts: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn test_account(id: &str, kind: &str, default: bool) -> AgentAccount {
        AgentAccount {
            id: id.into(),
            name: format!("{id} account"),
            agent_kind: kind.into(),
            api_key_env: None,
            config_path: None,
            is_default: default,
        }
    }

    #[tokio::test]
    async fn test_add_and_list() {
        let dir = TempDir::new().unwrap();
        let switcher = AccountSwitcher::new(dir.path()).unwrap();

        switcher
            .add_account(test_account("personal", "claude", true))
            .await
            .unwrap();
        switcher
            .add_account(test_account("work", "claude", false))
            .await
            .unwrap();

        let accounts = switcher.list_accounts().await;
        assert_eq!(accounts.len(), 2);
    }

    #[tokio::test]
    async fn test_switch() {
        let dir = TempDir::new().unwrap();
        let switcher = AccountSwitcher::new(dir.path()).unwrap();

        switcher
            .add_account(test_account("personal", "claude", true))
            .await
            .unwrap();
        switcher
            .add_account(test_account("work", "claude", false))
            .await
            .unwrap();

        let active = switcher.get_active(&AgentKind::ClaudeCode).await;
        assert!(active.is_none());

        switcher.switch(&AgentKind::ClaudeCode, "work").await.unwrap();

        let active = switcher.get_active(&AgentKind::ClaudeCode).await.unwrap();
        assert_eq!(active.id, "work");
    }

    #[tokio::test]
    async fn test_get_default() {
        let dir = TempDir::new().unwrap();
        let switcher = AccountSwitcher::new(dir.path()).unwrap();

        switcher
            .add_account(test_account("personal", "claude", true))
            .await
            .unwrap();
        switcher
            .add_account(test_account("work", "claude", false))
            .await
            .unwrap();

        let default = switcher.get_default(&AgentKind::ClaudeCode).await.unwrap();
        assert_eq!(default.id, "personal");

        let or_default = switcher.get_or_default(&AgentKind::ClaudeCode).await.unwrap();
        assert_eq!(or_default.id, "personal");
    }

    #[tokio::test]
    async fn test_remove() {
        let dir = TempDir::new().unwrap();
        let switcher = AccountSwitcher::new(dir.path()).unwrap();

        switcher
            .add_account(test_account("personal", "claude", true))
            .await
            .unwrap();
        switcher.remove_account("personal").await.unwrap();
        assert_eq!(switcher.list_accounts().await.len(), 0);
    }

    #[tokio::test]
    async fn test_persistence() {
        let dir = TempDir::new().unwrap();

        {
            let switcher = AccountSwitcher::new(dir.path()).unwrap();
            switcher
                .add_account(test_account("personal", "claude", true))
                .await
                .unwrap();
        }

        let switcher = AccountSwitcher::new(dir.path()).unwrap();
        assert_eq!(switcher.list_accounts().await.len(), 1);
    }
}
