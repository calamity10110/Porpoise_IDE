# Troubleshooting

## Build Issues

### "linker `cc` not found"
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# macOS
xcode-select --install
```

### "failed to run custom build command for `openssl-sys`"
```bash
# Ubuntu/Debian
sudo apt-get install pkg-config libssl-dev

# macOS
brew install openssl
```

### "could not find `native-tls`"
```bash
# Ubuntu/Debian
sudo apt-get install pkg-config libssl-dev

# Windows
# Install OpenSSL from https://slproweb.com/products/Win32OpenSSL.html
```

## Runtime Issues

### "Address already in use"
```bash
# Find process using port
lsof -i :9876
# Kill it
kill -9 <PID>
```

### "Permission denied" on serial port
```bash
# Linux
sudo usermod -aG dialout $USER
# Logout and login again

# macOS
sudo dseditgroup -o edit -a $USER -t user wheel
```

## ESP32 Issues

### "Failed to connect to ESP32"
1. Check USB cable (data, not charge-only)
2. Hold BOOT button during flash
3. Try different USB port
4. Install CP2102/CH340 drivers

### "Flash size mismatch"
```bash
# Check flash size
esptool.py flash_id
# Update partitions.csv if needed
```

## Docker Issues

### "Cannot connect to Docker daemon"
```bash
# Start Docker
sudo systemctl start docker
# Add user to docker group
sudo usermod -aG docker $USER
```

## Getting Help

- [GitHub Issues](https://github.com/porpoise-ide/porpoise/issues)
- [Discord](https://discord.gg/porpoise)
- [Documentation](https://docs.porpoise.dev)
