# Rivulet User Guide

Rivulet is a terminal-first, offline-first RSS/Atom feed reader with full-text content scraping.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Managing Feeds](#managing-feeds)
- [Reading Articles](#reading-articles)
- [Terminal UI (TUI)](#terminal-ui-tui)
- [Content Scraping](#content-scraping)
- [Configuration](#configuration)
- [Background Updates](#background-updates)
- [Data Storage](#data-storage)
- [Keyboard Shortcuts](#keyboard-shortcuts)

---

## Installation

### From Source

```bash
git clone https://github.com/user/rivulet
cd rivulet
cargo build --release
```

The binary will be at `./target/release/rivulet`.

### Add to PATH

```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$PATH:/path/to/rivulet/target/release"
```

### Requirements

- **Rust** 1.70+ (for building)
- **Chrome/Chromium** (optional, for content scraping)

---

## Quick Start

```bash
# Add your first feed
rivulet add https://blog.rust-lang.org/feed.xml

# List your feeds
rivulet list

# Update all feeds
rivulet update

# Launch the terminal UI
rivulet tui
```

---

## Managing Feeds

### Adding Feeds

```bash
# Add a single feed
rivulet add https://example.com/rss.xml

# The feed is fetched immediately and items are stored
```

### Removing Feeds

```bash
# Remove by URL
rivulet remove https://example.com/rss.xml
```

### Importing from OPML

Export your feeds from another reader as OPML, then:

```bash
# Import all feeds from OPML file
rivulet import feeds.opml
```

### Listing Feeds

```bash
# List all feeds with unread counts
rivulet list

# Example output:
# Rust Blog (5 unread)
#   https://blog.rust-lang.org/feed.xml
# Hacker News (42 unread)
#   https://news.ycombinator.com/rss
```

### Updating Feeds

```bash
# Update all feeds (fetch new items)
rivulet update

# With more parallel workers (default: 10)
rivulet update --workers 20
```

---

## Reading Articles

### Listing Items

```bash
# List all items across all feeds
rivulet list --items

# Example output:
# ● 2024-01-15 Announcing Rust 1.75
#   2024-01-14 This Week in Rust 523
# ● 2024-01-13 New crate: tokio-console
#
# ● = unread, blank = read
```

### In the TUI

1. Run `rivulet tui`
2. Navigate feeds with `j`/`k` or arrow keys
3. Press `Enter` to view items for a feed
4. Press `Tab` to switch to items pane
5. Navigate items and view preview
6. Press `o` to open in browser

---

## Terminal UI (TUI)

Launch with:

```bash
rivulet tui
```

### Layout

```
┌─────────────────┬────────────────────────────────┐
│ Feeds           │ Items                          │
│                 │                                │
│ > Rust Blog (5) │ ● Announcing Rust 1.75         │
│   HN (42)       │   This Week in Rust            │
│   Lobsters (8)  │ ● New async features           │
│                 │                                │
├─────────────────┴────────────────────────────────┤
│ Preview                                          │
│                                                  │
│ # Announcing Rust 1.75                           │
│                                                  │
│ The Rust team is happy to announce...            │
│                                                  │
└──────────────────────────────────────────────────┘
```

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `n` / `PageDown` | Next page |
| `p` / `PageUp` | Previous page |
| `Tab` | Next pane |
| `Shift+Tab` | Previous pane |
| `Enter` | Select feed / expand |
| `m` | Toggle maximize preview |

### Actions

| Key | Action |
|-----|--------|
| `r` | Toggle read/unread |
| `s` | Toggle starred |
| `o` | Open in browser |
| `R` | Refresh all feeds |
| `d` / `Delete` | Delete feed (with confirmation) |
| `q` / `Ctrl+c` | Quit |

### Status Bar

The bottom bar shows:
- Current status messages
- "Refreshing..." during updates
- Delete confirmation prompts

---

## Content Scraping

Many RSS feeds only include summaries. Rivulet can scrape full article content using headless Chrome.

### Automatic Scraping

Enable in config (`~/.config/rivulet/config.toml`):

```toml
[scraper]
enabled = true
```

Items are automatically scraped when:
- Adding new feeds
- Updating feeds
- Importing OPML
- Refreshing in TUI

### Manual Scraping

```bash
# Scrape items missing content
rivulet scrape

# Scrape specific feed
rivulet scrape --feed "https://beej.us/blog/rss.xml"

# Options
rivulet scrape --limit 20        # Max items to scrape
rivulet scrape --concurrency 5   # Parallel browser pages
rivulet scrape --visible         # Show browser (debug)
```

### How It Works

1. Identifies items with missing/short content
2. Launches headless Chrome
3. Navigates to article URL
4. Extracts main content using CSS selectors
5. Stores full HTML in database

See [scraper.md](scraper.md) for detailed configuration.

---

## Configuration

Configuration file: `~/.config/rivulet/config.toml`

A default config is created on first run. Copy the sample:

```bash
cp config.sample.toml ~/.config/rivulet/config.toml
```

### Colors

```toml
[colors]
# Border colors
active_border = "Cyan"
inactive_border = "DarkGray"

# Selection highlight
selection_bg_active = "Cyan"
selection_fg_active = "Black"
selection_bg_inactive = "DarkGray"
selection_fg_inactive = "White"

# Item colors
read_item = "DarkGray"
unread_item = "White"

# Metadata in preview
metadata_author = "Yellow"
metadata_date = "Yellow"
metadata_link = "Blue"

# Status bar
status_fg = "White"
status_bg = "DarkGray"
```

Color values can be:
- Named: `Black`, `Red`, `Green`, `Yellow`, `Blue`, `Magenta`, `Cyan`, `White`, `Gray`, `DarkGray`, `LightRed`, etc.
- Hex: `"#FF5500"` or `"#F50"`

### Keybindings

```toml
[keybindings]
# Navigation
quit = ["q", "Ctrl+c"]
move_up = ["k", "Up"]
move_down = ["j", "Down"]
next_page = ["n", "PageDown"]
prev_page = ["p", "PageUp"]
next_pane = ["Tab"]
prev_pane = ["BackTab", "Shift+Tab"]

# Actions
select = ["Enter"]
toggle_read = ["r"]
toggle_star = ["s"]
open_in_browser = ["o"]
refresh = ["R"]
toggle_maximize = ["m"]
delete_feed = ["d", "Delete"]
```

Key formats:
- Single char: `"a"`, `"A"`, `"1"`
- Special keys: `Enter`, `Tab`, `Backspace`, `Delete`, `Home`, `End`, `PageUp`, `PageDown`, `Up`, `Down`, `Left`, `Right`, `Esc`, `Space`, `F1`-`F12`
- Modifiers: `"Ctrl+c"`, `"Shift+Tab"`, `"Alt+Enter"`

### Scraper

```toml
[scraper]
enabled = true
headless = true
min_content_length = 200
timeout_secs = 30
wait_after_load_ms = 1000
max_concurrency = 3
block_images = true
block_stylesheets = true

content_selectors = [
    "article",
    "main",
    ".post-content",
    ".entry-content",
]

remove_selectors = [
    "nav",
    "header",
    "footer",
    ".sidebar",
    ".ads",
    "script",
    "style",
]
```

---

## Background Updates

### Built-in Daemon

```bash
# Start background updates (every hour)
rivulet daemon start

# Custom interval
rivulet daemon start --interval 30m   # 30 minutes
rivulet daemon start --interval 6h    # 6 hours
rivulet daemon start --interval 1d    # Daily

# With logging
rivulet daemon start --log ~/.local/log/rivulet.log

# Check status
rivulet daemon status

# Stop daemon
rivulet daemon stop
```

### Auto-start

See [scheduling.md](scheduling.md) for:
- macOS (launchd)
- Linux (systemd)
- Windows (Task Scheduler)

---

## Data Storage

### Database Location

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/rivulet/rivulet.db` |
| Linux | `~/.local/share/rivulet/rivulet.db` |
| Windows | `%APPDATA%\rivulet\rivulet.db` |

### Config Location

| Platform | Path |
|----------|------|
| macOS | `~/.config/rivulet/config.toml` |
| Linux | `~/.config/rivulet/config.toml` |
| Windows | `%APPDATA%\rivulet\config.toml` |

### Database Schema

SQLite database with tables:
- `feeds` - Feed metadata (URL, title, ETag, last fetch time)
- `items` - Articles (title, link, content, summary, author, dates)
- `item_state` - Read/starred status per item

### Backup

```bash
# Backup database
cp ~/Library/Application\ Support/rivulet/rivulet.db ~/backup/

# Restore
cp ~/backup/rivulet.db ~/Library/Application\ Support/rivulet/
```

---

## Keyboard Shortcuts

### TUI Quick Reference

```
Navigation
──────────────────────────
j/↓         Move down
k/↑         Move up
n/PageDown  Next page
p/PageUp    Previous page
Tab         Next pane
Shift+Tab   Previous pane
Enter       Select/expand
m           Maximize preview

Actions
──────────────────────────
r           Toggle read
s           Toggle star
o           Open in browser
R           Refresh feeds
d/Delete    Delete feed
q/Ctrl+c    Quit
```

---

## Command Reference

```
rivulet add <URL>              Add a new feed
rivulet remove <URL>           Remove a feed
rivulet import <FILE>          Import OPML file
rivulet update                 Update all feeds
rivulet list                   List feeds
rivulet list --items           List all items
rivulet tui                    Launch terminal UI
rivulet scrape                 Scrape full content
rivulet daemon start           Start background updater
rivulet daemon stop            Stop background updater
rivulet daemon status          Check daemon status

Global Options
──────────────────────────
-w, --workers <N>              Parallel fetch workers (default: 10)
-h, --help                     Show help
-V, --version                  Show version
```

---

## Tips & Tricks

### Efficient Reading Workflow

1. Run `rivulet daemon start` for automatic updates
2. Open `rivulet tui`
3. Press `Tab` to go to items
4. Use `j`/`k` to browse
5. Press `o` to open interesting articles
6. Articles auto-mark as read when opened

### Handling Many Feeds

```bash
# Use more workers for faster updates
rivulet update --workers 20

# Lower scraper concurrency to reduce memory
# In config.toml:
# [scraper]
# max_concurrency = 2
```

### Offline Reading

All content is stored locally. Once scraped, articles are available offline:

1. Update feeds while online: `rivulet update`
2. Wait for scraping to complete
3. Read offline in TUI

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug rivulet update

# Scraper debugging
RUST_LOG=info rivulet scrape --visible

# Check database directly
sqlite3 ~/Library/Application\ Support/rivulet/rivulet.db \
  "SELECT title, length(content) FROM items LIMIT 10;"
```
