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
rivulet list --unread
rivulet list --queued

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

Filtered item views:

```bash
rivulet list --unread
rivulet list --starred
rivulet list --queued
rivulet list --saved
rivulet list --archived
```

Only one item filter can be used at a time.

### `rivulet search <query>`

Search locally indexed item titles, authors, summaries, links, feed titles, and scraped article content.

```bash
rivulet search rust
rivulet search "borrow checker" --limit 10
rivulet search databases --unread
rivulet search auth --queued
```

Search supports the same item filters as `list`, and archived items are excluded unless `--archived` is passed.

### `rivulet auth`

Create and check persistent Chrome profiles for sites that require browser login.

```bash
rivulet auth add my-site --site https://example.com/login
rivulet auth check my-site --url https://example.com/account
rivulet auth list
rivulet scrape --auth-profile my-site --limit 10
```

`auth add` opens a visible Chrome window. Log in manually, then return to the terminal and press Enter. Rivulet stores profile metadata in SQLite, but it does not store passwords or raw cookie values; browser session material remains in the Chrome profile directory.

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
| `L` | Toggle queued/read-later status |
| `S` | Toggle saved status |
| `x` | Toggle archived status |
| `a` | Show all items |
| `u` | Show unread items |
| `f` | Show starred items |
| `l` | Show queued/read-later items |
| `v` | Show saved items |
| `X` | Show archived items |
| `o` | Open item link in browser |
| `R` | Refresh all feeds |
| `q` | Quit |

### Visual Indicators

- `●` - Unread item
- `★` - Starred item
- `Q` - Queued/read-later item
- `S` - Saved item
- `x` - Archived item
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

The TUI keeps stdout/stderr reserved for the terminal interface. TUI logs are
written to the Rivulet data directory instead:

- **macOS**: `~/Library/Application Support/rivulet/tui.log`
- **Linux**: `~/.local/share/rivulet/tui.log`
- **Windows**: `C:\Users\<user>\AppData\Roaming\rivulet\tui.log`

Override the TUI log file path with `RIVULET_TUI_LOG`:

```bash
RIVULET_TUI_LOG=/tmp/rivulet-tui.log RUST_LOG=rivulet=trace,chromiumoxide=debug rivulet tui
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
