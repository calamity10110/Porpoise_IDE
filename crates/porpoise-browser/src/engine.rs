use std::sync::Arc;

use async_trait::async_trait;
use porpoise_core::error::{PorpoiseError, Result};
use tokio::sync::RwLock;

use crate::navigation::{NavigationResult, NavigationStatus};

#[async_trait]
pub trait BrowserEngine: Send + Sync {
    async fn navigate(&self, url: &str) -> Result<NavigationResult>;
    async fn snapshot(&self) -> Result<Vec<u8>>;
    async fn click(&self, selector: &str) -> Result<()>;
    async fn fill(&self, selector: &str, value: &str) -> Result<()>;
    async fn get_html(&self, selector: &str) -> Result<String>;
    async fn back(&self) -> Result<()>;
    async fn forward(&self) -> Result<()>;
    async fn reload(&self) -> Result<()>;
    fn page_id(&self) -> String;
}

/// Headless browser that fetches pages via HTTP.
///
/// Provides basic navigate/get_html/snapshot functionality.
/// DOM interactions (click/fill) are no-ops with warnings — no JS engine.
pub struct HeadlessBrowser {
    state: Arc<RwLock<BrowserState>>,
}

struct BrowserState {
    current_url: Option<String>,
    page_html: String,
    page_title: String,
    page_id: String,
    history: Vec<String>,
    history_index: usize,
    client: reqwest::Client,
}

impl HeadlessBrowser {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(BrowserState {
                current_url: None,
                page_html: String::new(),
                page_title: String::new(),
                page_id: gen_page_id(),
                history: Vec::new(),
                history_index: 0,
                client: reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .user_agent("Porpoise/0.1")
                    .build()
                    .expect("reqwest client builder should succeed"),
            })),
        }
    }

    async fn fetch_page(client: &reqwest::Client, url: &str) -> Result<(String, String)> {
        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e| PorpoiseError::Browser(format!("fetch failed: {e}")))?;
        let html = resp
            .text()
            .await
            .map_err(|e| PorpoiseError::Browser(format!("read body: {e}")))?;
        let title = extract_title(&html);
        Ok((html, title))
    }
}

impl Default for HeadlessBrowser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BrowserEngine for HeadlessBrowser {
    async fn navigate(&self, url: &str) -> Result<NavigationResult> {
        let client = self.state.read().await.client.clone();

        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e| PorpoiseError::Browser(format!("fetch failed: {e}")))?;

        let status_code = resp.status();
        let html = resp
            .text()
            .await
            .map_err(|e| PorpoiseError::Browser(format!("read body: {e}")))?;
        let title = extract_title(&html);
        let page_id = gen_page_id();

        let mut st = self.state.write().await;

        let truncate_at = st.history_index + 1;
        if truncate_at < st.history.len() {
            st.history.truncate(truncate_at);
        }
        st.history.push(url.to_string());
        st.history_index = st.history.len() - 1;

        st.current_url = Some(url.to_string());
        st.page_html = html;
        st.page_title = title.clone();
        st.page_id = page_id.clone();

        Ok(NavigationResult {
            url: url.to_string(),
            title,
            status: if status_code.is_success() {
                NavigationStatus::Loaded
            } else {
                NavigationStatus::Error(format!("HTTP {}", status_code.as_u16()))
            },
            page_id,
        })
    }

    async fn snapshot(&self) -> Result<Vec<u8>> {
        let st = self.state.read().await;
        Ok(st.page_html.as_bytes().to_vec())
    }

    async fn click(&self, _selector: &str) -> Result<()> {
        tracing::warn!("HeadlessBrowser::click is a no-op — no DOM engine available");
        Ok(())
    }

    async fn fill(&self, _selector: &str, _value: &str) -> Result<()> {
        tracing::warn!("HeadlessBrowser::fill is a no-op — no DOM engine available");
        Ok(())
    }

    async fn get_html(&self, _selector: &str) -> Result<String> {
        let st = self.state.read().await;
        Ok(st.page_html.clone())
    }

    async fn back(&self) -> Result<()> {
        let (url, client) = {
            let st = self.state.read().await;
            if st.history_index > 0 {
                (Some(st.history[st.history_index - 1].clone()), st.client.clone())
            } else {
                (None, st.client.clone())
            }
        };
        let url = url.ok_or_else(|| PorpoiseError::Browser("no history to go back to".into()))?;

        let (html, title) = Self::fetch_page(&client, &url).await?;

        let mut st = self.state.write().await;
        st.history_index -= 1;
        st.current_url = Some(url);
        st.page_html = html;
        st.page_title = title;
        st.page_id = gen_page_id();
        Ok(())
    }

    async fn forward(&self) -> Result<()> {
        let (url, client) = {
            let st = self.state.read().await;
            if st.history_index + 1 < st.history.len() {
                (Some(st.history[st.history_index + 1].clone()), st.client.clone())
            } else {
                (None, st.client.clone())
            }
        };
        let url = url.ok_or_else(|| PorpoiseError::Browser("no forward history".into()))?;

        let (html, title) = Self::fetch_page(&client, &url).await?;

        let mut st = self.state.write().await;
        st.history_index += 1;
        st.current_url = Some(url);
        st.page_html = html;
        st.page_title = title;
        st.page_id = gen_page_id();
        Ok(())
    }

    async fn reload(&self) -> Result<()> {
        let (url, client) = {
            let st = self.state.read().await;
            (st.current_url.clone(), st.client.clone())
        };
        let url = url.ok_or_else(|| PorpoiseError::Browser("no page to reload".into()))?;

        let (html, title) = Self::fetch_page(&client, &url).await?;

        let mut st = self.state.write().await;
        st.page_html = html;
        st.page_title = title;
        Ok(())
    }

    fn page_id(&self) -> String {
        self.state
            .try_read()
            .map(|st| st.page_id.clone())
            .unwrap_or_else(|_| "unknown".to_string())
    }
}

fn gen_page_id() -> String {
    format!(
        "page_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    )
}

fn extract_title(html: &str) -> String {
    let lower = html.to_lowercase();
    if let Some(start) = lower.find("<title>")
        && let Some(end) = lower[start..].find("</title>")
    {
        let title_start = start + "<title>".len();
        return html[title_start..start + end].trim().to_string();
    }
    "Untitled".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title() {
        assert_eq!(
            extract_title("<html><head><title>Hello World</title></head>"),
            "Hello World"
        );
        assert_eq!(extract_title("<TITLE>Test</TITLE>"), "Test");
        assert_eq!(extract_title("<html>no title</html>"), "Untitled");
    }

    #[test]
    fn test_browser_creation() {
        let browser = HeadlessBrowser::new();
        assert!(!browser.page_id().is_empty());
    }
}
