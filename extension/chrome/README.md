# Porpoise Bridge — Chrome Extension

Browser bridge for Porpoise AI agents. Pair with your desktop daemon to inspect DOM, send pages to agents, and replay automations.

## Install (Developer Mode)

1. Open `chrome://extensions`
2. Enable **Developer mode** (top right)
3. Click **Load unpacked**
4. Select the `extension/chrome/` folder

## Pair with Daemon

1. Make sure your daemon is running with WSS enabled:
   ```bash
   PORPOISE_WS_PORT=9876 PORPOISE_WS_TLS=1 porpoise-server
   ```
2. Get the pairing token:
   ```bash
   porpoise mobile qr
   ```
3. Open the extension popup → enter host/port/token → click Connect
4. Or right-click any page → **Send page to Porpoise agent**

## Features

| Feature | Description |
|---------|-------------|
| **Send page to agent** | Sends URL + title to selected agent for analysis |
| **Inspect mode** | Click any page element → sends HTML, CSS, and selector to agent |
| **Automation playback** | Background service worker replays click/type/wait sequences |
| **Reconnect** | Exponential backoff reconnection (1s → 2s → 4s → ... → 30s) |
| **TLS support** | Connects via WSS with self-signed cert fingerprint verification |
| **Notifications** | Agent events forwarded to Chrome notifications |

## Files

```
extension/chrome/
├── manifest.json        # Manifest V3
├── background.js        # Service worker (WSS connection, message routing)
├── popup.html           # Popup UI
├── popup.js             # Popup logic
├── popup.css            # Popup styles
├── options.html         # Settings page
├── options.js           # Settings logic
├── content.js           # Content script (DOM inspection, automation)
├── icons/               # Extension icons
└── README.md            # This file
```

## Development

No build step needed — all files are vanilla JS with ES modules. Edit and reload the extension at `chrome://extensions/service-worker?load-unpacked`.

To debug the service worker:
1. Go to `chrome://extensions`
2. Find Porpoise Bridge
3. Click service worker link (inspects the background script)

## Privacy

- The extension only connects to the configured host/port
- No telemetry, no analytics, no third-party servers
- DOM data only sent when you explicitly trigger Inspect mode or "Send to Agent"
