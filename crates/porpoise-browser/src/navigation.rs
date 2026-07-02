use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationResult {
    pub url: String,
    pub title: String,
    pub status: NavigationStatus,
    pub page_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NavigationStatus {
    Loaded,
    Error(String),
    Timeout,
}
