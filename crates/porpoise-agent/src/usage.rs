use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::Utc;
use porpoise_core::{
    error::{PorpoiseError, Result},
    types::id::AgentId,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub agent_id: String,
    pub session_id: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub estimated_cost_usd: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageSummary {
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_tokens: u64,
    pub total_estimated_cost_usd: f64,
    pub session_count: usize,
}

/// Tracks token usage and estimated costs for agent sessions.
pub struct TokenUsageMonitor {
    entries: Arc<RwLock<Vec<TokenUsage>>>,
    storage_path: PathBuf,
    cost_per_1k_prompt: f64,
    cost_per_1k_completion: f64,
}

impl TokenUsageMonitor {
    pub fn new(storage_dir: &Path) -> Result<Self> {
        let storage_path = storage_dir.join("token_usage.jsonl");
        let entries = Self::load_entries(&storage_path)?;
        Ok(Self {
            entries: Arc::new(RwLock::new(entries)),
            storage_path,
            cost_per_1k_prompt: 0.003,
            cost_per_1k_completion: 0.015,
        })
    }

    pub fn with_pricing(mut self, cost_per_1k_prompt: f64, cost_per_1k_completion: f64) -> Self {
        self.cost_per_1k_prompt = cost_per_1k_prompt;
        self.cost_per_1k_completion = cost_per_1k_completion;
        self
    }

    pub async fn record(
        &self,
        agent_id: AgentId,
        session_id: &str,
        prompt_tokens: u64,
        completion_tokens: u64,
    ) -> Result<TokenUsage> {
        let total = prompt_tokens + completion_tokens;
        let cost = (prompt_tokens as f64 / 1000.0) * self.cost_per_1k_prompt
            + (completion_tokens as f64 / 1000.0) * self.cost_per_1k_completion;

        let entry = TokenUsage {
            agent_id: agent_id.to_string(),
            session_id: session_id.to_string(),
            prompt_tokens,
            completion_tokens,
            total_tokens: total,
            estimated_cost_usd: (cost * 1_000_000.0).round() / 1_000_000.0,
            timestamp: Utc::now().to_rfc3339(),
        };

        let mut entries = self.entries.write().await;
        entries.push(entry.clone());
        self.append_entry(&entry)?;
        Ok(entry)
    }

    pub async fn summary(&self) -> UsageSummary {
        let entries = self.entries.read().await;
        let mut summary = UsageSummary::default();
        let mut sessions: HashMap<String, ()> = HashMap::new();

        for entry in entries.iter() {
            summary.total_prompt_tokens += entry.prompt_tokens;
            summary.total_completion_tokens += entry.completion_tokens;
            summary.total_tokens += entry.total_tokens;
            summary.total_estimated_cost_usd += entry.estimated_cost_usd;
            sessions.insert(entry.session_id.clone(), ());
        }

        summary.session_count = sessions.len();
        summary.total_estimated_cost_usd = (summary.total_estimated_cost_usd * 1_000_000.0).round() / 1_000_000.0;
        summary
    }

    pub async fn summary_for_agent(&self, agent_id: &AgentId) -> UsageSummary {
        let entries = self.entries.read().await;
        let agent_str = agent_id.to_string();
        let mut summary = UsageSummary::default();
        let mut sessions: HashMap<String, ()> = HashMap::new();

        for entry in entries.iter().filter(|e| e.agent_id == agent_str) {
            summary.total_prompt_tokens += entry.prompt_tokens;
            summary.total_completion_tokens += entry.completion_tokens;
            summary.total_tokens += entry.total_tokens;
            summary.total_estimated_cost_usd += entry.estimated_cost_usd;
            sessions.insert(entry.session_id.clone(), ());
        }

        summary.session_count = sessions.len();
        summary
    }

    pub async fn summary_for_session(&self, session_id: &str) -> UsageSummary {
        let entries = self.entries.read().await;
        let mut summary = UsageSummary::default();

        for entry in entries.iter().filter(|e| e.session_id == session_id) {
            summary.total_prompt_tokens += entry.prompt_tokens;
            summary.total_completion_tokens += entry.completion_tokens;
            summary.total_tokens += entry.total_tokens;
            summary.total_estimated_cost_usd += entry.estimated_cost_usd;
        }

        summary.session_count = 1;
        summary
    }

    pub async fn list_recent(&self, limit: usize) -> Vec<TokenUsage> {
        let entries = self.entries.read().await;
        entries.iter().rev().take(limit).cloned().collect()
    }

    pub async fn clear(&self) {
        self.entries.write().await.clear();
        let _ = std::fs::remove_file(&self.storage_path);
    }

    fn load_entries(path: &Path) -> Result<Vec<TokenUsage>> {
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(path).map_err(|e| PorpoiseError::Agent(format!("read usage: {e}")))?;
        let mut entries = Vec::new();
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<TokenUsage>(line) {
                Ok(entry) => entries.push(entry),
                Err(_) => continue,
            }
        }
        Ok(entries)
    }

    fn append_entry(&self, entry: &TokenUsage) -> Result<()> {
        use std::io::Write;
        if let Some(parent) = self.storage_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| PorpoiseError::Agent(format!("create usage dir: {e}")))?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.storage_path)
            .map_err(|e| PorpoiseError::Agent(format!("open usage file: {e}")))?;
        let json = serde_json::to_string(entry).map_err(|e| PorpoiseError::Agent(format!("serialize usage: {e}")))?;
        writeln!(file, "{json}").map_err(|e| PorpoiseError::Agent(format!("write usage: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test_record_and_summary() {
        let dir = TempDir::new().unwrap();
        let monitor = TokenUsageMonitor::new(dir.path()).unwrap();

        let agent = AgentId::new();
        monitor.record(agent, "sess1", 1000, 500).await.unwrap();
        monitor.record(agent, "sess1", 2000, 1000).await.unwrap();

        let summary = monitor.summary().await;
        assert_eq!(summary.total_prompt_tokens, 3000);
        assert_eq!(summary.total_completion_tokens, 1500);
        assert_eq!(summary.total_tokens, 4500);
        assert_eq!(summary.session_count, 1);
        assert!(summary.total_estimated_cost_usd > 0.0);
    }

    #[tokio::test]
    async fn test_per_agent_summary() {
        let dir = TempDir::new().unwrap();
        let monitor = TokenUsageMonitor::new(dir.path()).unwrap();

        let a1 = AgentId::new();
        let a2 = AgentId::new();
        monitor.record(a1, "s1", 100, 50).await.unwrap();
        monitor.record(a2, "s2", 200, 100).await.unwrap();

        let s1 = monitor.summary_for_agent(&a1).await;
        assert_eq!(s1.total_tokens, 150);

        let s2 = monitor.summary_for_agent(&a2).await;
        assert_eq!(s2.total_tokens, 300);
    }

    #[tokio::test]
    async fn test_persistence() {
        let dir = TempDir::new().unwrap();

        {
            let monitor = TokenUsageMonitor::new(dir.path()).unwrap();
            let agent = AgentId::new();
            monitor.record(agent, "s1", 500, 200).await.unwrap();
        }

        let monitor = TokenUsageMonitor::new(dir.path()).unwrap();
        let summary = monitor.summary().await;
        assert_eq!(summary.total_tokens, 700);
    }
}
