# Module Design: porpoise-app

> Desktop application shell (Tauri-based).

## Purpose
Provides the native desktop GUI for Porpoise. Built on Tauri, it wraps the CLI and server functionality in a graphical interface: worktree sidebar, terminal panel, embedded browser, settings.

## Dependencies
- porpoise-cli (command execution)
- porpoise-server (embeds as library)

## Key Features
- Worktree sidebar tree view with drag-and-drop
- Terminal panel with split panes
- Monaco/markdown editor for file editing
- Settings UI for config management
- System tray with agent status
- Keyboard shortcuts per platform
- Native menus (macOS menu bar, Windows taskbar)

## Bridges
- App calls CLI commands via porpoise_cli internally
- Connects to porpoise-server via IPC for real-time state
- Uses Tauri's notification API for agent completion events
