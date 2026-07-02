use async_trait::async_trait;
use porpoise_core::error::Result;
use crate::navigation::NavigationResult;

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

pub struct HeadlessBrowser;

#[async_trait]
impl BrowserEngine for HeadlessBrowser {
    async fn navigate(&self, _url: &str) -> Result<NavigationResult> {
        Err(porpoise_core::error::PorpoiseError::Unimplemented("headless browser - compile with platform webview feature"))
    }
    async fn snapshot(&self) -> Result<Vec<u8>> {
        Err(porpoise_core::error::PorpoiseError::Unimplemented("headless browser snapshot"))
    }
    async fn click(&self, _selector: &str) -> Result<()> {
        Err(porpoise_core::error::PorpoiseError::Unimplemented("headless browser click"))
    }
    async fn fill(&self, _selector: &str, _value: &str) -> Result<()> {
        Err(porpoise_core::error::PorpoiseError::Unimplemented("headless browser fill"))
    }
    async fn get_html(&self, _selector: &str) -> Result<String> {
        Err(porpoise_core::error::PorpoiseError::Unimplemented("headless browser get_html"))
    }
    async fn back(&self) -> Result<()> {
        Err(porpoise_core::error::PorpoiseError::Unimplemented("headless browser back"))
    }
    async fn forward(&self) -> Result<()> {
        Err(porpoise_core::error::PorpoiseError::Unimplemented("headless browser forward"))
    }
    async fn reload(&self) -> Result<()> {
        Err(porpoise_core::error::PorpoiseError::Unimplemented("headless browser reload"))
    }
    fn page_id(&self) -> String { "headless".into() }
}
