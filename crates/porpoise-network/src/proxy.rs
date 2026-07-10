pub struct ProxyConfig {
    pub http: Option<String>,
    pub https: Option<String>,
    pub socks5: Option<String>,
    pub no_proxy: Vec<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        let no_proxy = std::env::var("NO_PROXY")
            .or_else(|_| std::env::var("no_proxy"))
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            http: std::env::var("HTTP_PROXY")
                .or_else(|_| std::env::var("http_proxy"))
                .ok(),
            https: std::env::var("HTTPS_PROXY")
                .or_else(|_| std::env::var("https_proxy"))
                .ok(),
            socks5: None,
            no_proxy,
        }
    }
}

impl ProxyConfig {
    pub fn to_reqwest_proxy(&self) -> Option<reqwest::Proxy> {
        if let Some(ref https) = self.https {
            let mut proxy = reqwest::Proxy::https(https).ok()?;
            for n in &self.no_proxy {
                proxy = proxy.no_proxy(reqwest::NoProxy::from_string(n));
            }
            return Some(proxy);
        }
        if let Some(ref http) = self.http {
            let proxy = reqwest::Proxy::http(http).ok()?;
            return Some(proxy);
        }
        None
    }
}
