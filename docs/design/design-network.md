# Module Design: porpoise-network

> HTTP/WebSocket networking with rate limiting and proxy support.

## Purpose
Provides shared networking primitives (HTTP client, WebSocket client, rate limiter, proxy support) used by other crates for API calls and remote communication.

## Dependencies
- porpoise-core (types)
- reqwest, tokio-tungstenite

## Key Types
- `HttpClient` wrapper around reqwest with retry/backoff
- `WsClient` for WebSocket connections
- `RateLimiter` for API rate limit compliance
- Proxy configuration (HTTP, HTTPS, SOCKS5)

## Bridges
- porpoise-git uses network for remote provider API calls
- porpoise-ssh may use network for HTTP-based SSH setup
- porpoise-browser uses network for page loading
