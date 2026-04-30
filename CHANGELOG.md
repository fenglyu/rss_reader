# Changelog

All notable changes to Rivulet will be documented in this file.

## [Unreleased]

### Added

- **Configuration file support** (`~/.config/rivulet/config.toml`)
  - Customizable colors (named colors and hex codes like `#FF0000`)
  - Customizable keybindings (e.g., `Ctrl+c`, `Shift+Tab`, `j`)
  - Auto-generates default config with comments on first run
  - Graceful fallback to defaults for missing/invalid config

- **Delete feed in TUI**
  - Press `d` or `Delete` key in Feeds pane to delete selected feed
  - Confirmation prompt: `Delete "Feed Name"? (y/n)`
  - Configurable via `delete_feed` keybinding

- **Background daemon for automatic updates**
  - `rivulet daemon start` - Start background updater
  - `rivulet daemon stop` - Stop the daemon
  - `rivulet daemon status` - Check if daemon is running
  - Configurable update interval (`--interval 1h`, `30m`, `6h`, `1d`)
  - Optional logging to file (`--log path/to/file.log`)
  - PID file prevents multiple instances
  - Cross-platform support (macOS, Linux, Windows)

- **Scheduling documentation** (`docs/scheduling.md`)
  - Built-in daemon usage guide
  - System scheduler setup (launchd, systemd, cron, Task Scheduler)
  - Auto-start on login instructions for all platforms

- **Latest tab and refresh batches**
  - `Alt+1` opens a SQLite-backed Latest tab for recent items
  - `Alt+2` opens the Reader tab
  - Refresh runs record inserted item IDs so newly fetched items can be marked `NEW`
  - Reader feed navigation is available as a collapsible left rail via `\`

### Changed

- TUI now uses colors and keybindings from config file
- Status bar updated to show `d:Delete` hint
- TUI layout now uses top-level Latest/Reader tabs instead of the old vertical three-pane stack

## [0.1.0] - Initial Release

### Added

- Core RSS/Atom feed reader functionality
- Terminal UI with three-pane layout (Feeds, Items, Preview)
- Feed management: add, remove, update, list
- OPML import with parallel fetching
- Item state tracking (read/unread, starred)
- Keyboard navigation (vim-style j/k, arrow keys)
- Open articles in browser
- SQLite storage with migrations
- Conditional HTTP requests (ETag, Last-Modified)
- S3 storage support
