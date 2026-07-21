# Installing Porpoise Mobile Companion on Android

The Porpoise mobile companion is a Flutter app that pairs with your desktop daemon over WSS (WebSocket Secure). It lets you monitor agents, browse worktrees, and interact with terminals from your phone.

## Prerequisites

- Android 7.0+ (API 21+)
- Porpoise desktop daemon running on your local network (or same machine)
- GitHub account with access to the repository

## Download the APK

The APK is built automatically by GitHub Actions whenever a version tag is pushed:

1. Go to the repository's **Actions** tab
2. Select the **Android** workflow
3. Find the latest run (or trigger a new one with "Run workflow")
4. Download the **porpoise-mobile-apk** artifact
5. Extract `app-release.apk` from the zip

## Manual Build (for developers)

```bash
cd mobile
flutter create --platforms=android .
flutter pub get
flutter build apk --release
```

## Install on Device

1. Copy `app-release.apk` to your Android device
2. Open the file and tap "Install"
3. If prompted, enable "Install from unknown sources" for your file manager
4. Open the Porpoise Mobile app

## Pair with Desktop

1. On your desktop, start the daemon with mobile support:
   ```bash
   PORPOISE_WS_PORT=9876 PORPOISE_WS_TLS=1 porpoise-server
   ```
2. Get the pairing info:
   ```bash
   porpoise mobile qr
   ```
   This prints a JSON object with:
   - `url`: WebSocket URL (e.g., `wss://localhost:9876`)
   - `host`: daemon hostname/IP
   - `port`: WebSocket port
   - `token`: 64-char hex auth token
   - `tls_fingerprint`: SHA-256 fingerprint of the daemon's self-signed cert
3. On your phone, enter the host, port, and token
4. Tap **Connect**

> **Note**: The daemon uses a self-signed certificate on first start.
> Verify the TLS fingerprint matches between the daemon output and your phone.

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Connection refused | Ensure daemon is running and `PORPOISE_WS_PORT` is set |
| TLS error | Check that `PORPOISE_WS_TLS=1` is set and fingerprint matches |
| Token rejected | Restart daemon to generate a new token, or verify `PORPOISE_WS_TOKEN` |
| APK install blocked | Settings → Security → Install from unknown sources → enable for File Manager |

## Security Notes

- All traffic between the mobile app and daemon is encrypted via WSS (TLS 1.3)
- The daemon token authenticates the mobile client
- The self-signed certificate fingerprint is verified by the mobile app (TOFU model)
- HTTP/WS (cleartext) connections are disabled by default — set `useTls: false` in the Flutter app if needed
