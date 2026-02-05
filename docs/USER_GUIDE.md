# Rivulet User Guide

Rivulet is a terminal-first, offline-first RSS/Atom feed reader built in Rust.

## Installation

```bash
cargo build --release
# Binary will be at target/release/rivulet
```

## Quick Start

```bash
# Add a feed
rivulet add https://blog.rust-lang.org/feed.xml

# List your feeds
rivulet list

# List all items
rivulet list --items

# Update all feeds
rivulet update

# Launch the TUI
rivulet tui
```

## CLI Commands

### `rivulet add <url>`

Add a new RSS or Atom feed. The feed is immediately fetched and items are stored locally.

```bash
rivulet add https://example.com/feed.xml
```

### `rivulet remove <url>`

Remove a feed and all its items from the database.

```bash
rivulet remove https://example.com/feed.xml
```

### `rivulet update`

Update all feeds. Rivulet respects HTTP conditional headers (ETag, If-Modified-Since) to avoid re-downloading unchanged feeds.

```bash
rivulet update
```

### `rivulet list`

List all subscribed feeds with their unread counts.

```bash
rivulet list
```

### `rivulet list --items`

List all items across all feeds, sorted by publication date (newest first).

```bash
rivulet list --items
```

### `rivulet tui`

Launch the interactive terminal user interface.

## TUI Interface

The TUI uses a three-pane vertical layout:

```
┌──────────────────────────┐
│ Feeds (compact list)     │
├──────────────────────────┤
│ Item List (40% height)   │
├──────────────────────────┤
│ Preview Pane (remaining) │
└──────────────────────────┘
```

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Tab` | Cycle to next pane |
| `Shift+Tab` | Cycle to previous pane |
| `Enter` | Select feed (loads its items) |
| `r` | Toggle read status |
| `s` | Toggle starred status |
| `o` | Open item link in browser |
| `R` | Refresh all feeds |
| `q` | Quit |

### Visual Indicators

- `●` - Unread item
- `★` - Starred item
- Dimmed text - Read item
- Cyan border - Active pane

## Data Storage

Rivulet stores its database at:
- **macOS**: `~/Library/Application Support/rivulet/rivulet.db`
- **Linux**: `~/.local/share/rivulet/rivulet.db`
- **Windows**: `C:\Users\<user>\AppData\Roaming\rivulet\rivulet.db`

## Logging

Enable debug logging with the `RUST_LOG` environment variable:

```bash
RUST_LOG=rivulet=debug rivulet update
RUST_LOG=rivulet=trace rivulet tui
```

## Supported Feed Formats

- RSS 0.9x, 1.0, 2.0
- Atom 0.3, 1.0
- JSON Feed 1.0

## Architecture

Rivulet follows a modular pipeline architecture:

```
Fetcher → Normalizer → Store → Index → UI
```

- **Fetcher**: HTTP client with ETag/conditional request support
- **Normalizer**: Converts RSS/Atom to unified domain models
- **Store**: SQLite persistence with foreign key support
- **TUI**: ratatui-based terminal interface

## Troubleshooting

### Feed not updating

Rivulet respects HTTP 304 Not Modified responses. If a feed hasn't changed, no new items will be fetched. This is expected behavior.

### Build errors on macOS

If you encounter linker errors for SQLite or iconv, ensure Homebrew packages are installed:

```bash
brew install sqlite libiconv
```

The project includes a `.cargo/config.toml` that sets the correct library paths for Homebrew on Apple Silicon.
