use std::time::Duration;
use porpoise_core::error::{PorpoiseError, Result};

pub struct HttpClient {
    client: reqwest::Client,
    max_retries: u32,
    base_delay_ms: u64,
}

impl HttpClient {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("porpoise/0.1.0")
            .build()
            .map_err(|e| PorpoiseError::Network(format!("http client: {e}")))?;
        Ok(Self { client, max_retries: 3, base_delay_ms: 500 })
    }

    pub fn with_proxy(mut self, proxy: reqwest::Proxy) -> Result<Self> {
        let client = reqwest::Client::builder()
            .proxy(proxy)
            .timeout(Duration::from_secs(30))
            .user_agent("porpoise/0.1.0")
            .build()
            .map_err(|e| PorpoiseError::Network(format!("http client: {e}")))?;
        self.client = client;
        Ok(self)
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(secs))
            .user_agent("porpoise/0.1.0")
            .build()
            .unwrap_or(self.client);
        self.client = client;
        self
    }

    pub async fn get(&self, url: &str) -> Result<String> {
        self.request_with_retry(|| async {
            let resp = self.client.get(url).send().await
                .map_err(|e| PorpoiseError::Network(format!("GET {url}: {e}")))?;
            let status = resp.status();
            let body = resp.text().await
                .map_err(|e| PorpoiseError::Network(format!("read body: {e}")))?;
            if status.is_success() { Ok(body) }
            else { Err(PorpoiseError::Http { status: status.as_u16(), message: body }) }
        }).await
    }

    pub async fn post(&self, url: &str, body: &str) -> Result<String> {
        self.request_with_retry(|| async {
            let resp = self.client.post(url)
                .header("content-type", "application/json")
                .body(body.to_string())
                .send().await
                .map_err(|e| PorpoiseError::Network(format!("POST {url}: {e}")))?;
            let status = resp.status();
            let body = resp.text().await
                .map_err(|e| PorpoiseError::Network(format!("read body: {e}")))?;
            if status.is_success() { Ok(body) }
            else { Err(PorpoiseError::Http { status: status.as_u16(), message: body }) }
        }).await
    }

    pub async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let text = self.get(url).await?;
        serde_json::from_str(&text)
            .map_err(|e| PorpoiseError::Network(format!("json parse: {e}")))
    }

    async fn request_with_retry<F, Fut>(&self, f: F) -> Result<String>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<String>>,
    {
        let mut last_err = None;
        for attempt in 0..=self.max_retries {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt < self.max_retries {
                        let delay = self.base_delay_ms * (1u64 << attempt);
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                    }
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| PorpoiseError::Network("request failed".into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_new() {
        let client = HttpClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_http_client_with_timeout() {
        let _client = HttpClient::new().unwrap().with_timeout(10);
    }
}
