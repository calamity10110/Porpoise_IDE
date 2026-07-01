# Module Design: porpoise-browser

> Embedded browser engine — WKWebView, webkit2gtk, WebView2.

## Purpose
Provides an embedded web browser panel for agent use. Supports navigation, screenshots, and a design mode where agents can inspect page elements.

## Dependencies
- porpoise-core (types, events)
- Platform-specific webview crates

## Key Types
- `BrowserEngine` trait: Platform-specific impls
- DesignMode: element inspector -> screenshot pipeline

## Bridges
- Server IPC exposes browser methods to CLI/App
- Screenshots passed to agent via file system

## Platform Matrix
| Platform | Engine |
|----------|--------|
| macOS | WKWebView |
| Linux | webkit2gtk |
| Windows | WebView2 |
