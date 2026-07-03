# Module Design: porpoise-network

> HTTP/WebSocket networking with rate limiting, proxy support, and connectivity monitoring.

## Purpose
Provides shared networking primitives (HTTP client, WebSocket client, rate limiter, proxy support, connectivity monitoring) used by other crates for API calls and remote communication.

## Dependencies
- porpoise-core (types)
- reqwest, tokio-tungstenite

## Key Types
- `HttpClient` wrapper around reqwest with retry/backoff
- `WsClient` for WebSocket connections (connect/send/recv/close)
- `RateLimiter` token bucket for API rate limit compliance
- `ProxyConfig` for HTTP/HTTPS proxy auto-detection from env vars
- `ConnectivityMonitor` periodic TCP connectivity checks

## Implemented Features
| Feature | Status | Details |
|---------|--------|---------|
| HTTP client | ✅ | reqwest wrapper with retry/backoff |
| WebSocket client | ✅ | tokio-tungstenite connect/send/recv/close |
| Rate limiter | ✅ | Token bucket algorithm |
| Proxy configuration | ✅ | HTTP_PROXY/HTTPS_PROXY env var detection |
| Connectivity monitor | ✅ | Periodic TCP checks to 8.8.8.8:53, 1.1.1.1:53 |

## Bridges
- porpoise-git uses network for remote provider API calls
- porpoise-ssh may use network for HTTP-based SSH setup
- porpoise-browser uses network for page loading
