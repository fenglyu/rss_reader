# Rivulet

A terminal-first, offline-first RSS/Atom feed reader with full-text content scraping.

## Features

- **Terminal UI** - Three-pane interface with vim-style navigation
- **Offline-first** - All content stored locally in SQLite
- **Content Scraping** - Fetch full articles using headless Chrome
- **Background Updates** - Built-in daemon for automatic feed updates
- **OPML Import** - Import feeds from other readers
- **Reading Workflow** - Unread/starred/queued/saved/archived item views
- **Configurable** - Customize colors, keybindings, and scraper settings

## Installation

```bash
# From source
cargo install --path .

# Or build manually
cargo build --release
```

## Quick Start

```bash
# Initialize config (optional - auto-created on first run)
rivulet init

# Add a feed
rivulet add https://blog.rust-lang.org/feed.xml

# Update feeds
rivulet update

# Launch TUI
rivulet tui
```

## Config & Data Locations

| Platform | Config | Database |
|----------|--------|----------|
| macOS | `~/.config/rivulet/config.toml` | `~/Library/Application Support/rivulet/rivulet.db` |
| Linux | `~/.config/rivulet/config.toml` | `~/.local/share/rivulet/rivulet.db` |
| Windows | `%APPDATA%\rivulet\config.toml` | `%APPDATA%\rivulet\rivulet.db` |

## Screenshots

```
┌─────────────────┬────────────────────────────────┐
│ Feeds           │ Items                          │
│                 │                                │
│ > Rust Blog (5) │ ● Announcing Rust 1.75         │
│   HN (42)       │   This Week in Rust            │
│   Beej's Blog   │ ● Rust RPN Calculator          │
│                 │                                │
├─────────────────┴────────────────────────────────┤
│ Preview                                          │
│                                                  │
│ # Rust RPN Calculator                            │
│                                                  │
│ Implementing an RPN Calculator in Rust...        │
│                                                  │
└──────────────────────────────────────────────────┘
```

## Commands

```bash
rivulet init              # Initialize config file
rivulet add <URL>         # Add a new feed
rivulet remove <URL>      # Remove a feed
rivulet import <FILE>     # Import OPML file
rivulet update            # Update all feeds
rivulet list              # List feeds
rivulet list --items      # List all items
rivulet list --unread     # List unread items
rivulet list --queued     # List read-later queue
rivulet search rust       # Search local titles, summaries, and scraped content
rivulet auth add nyt --site https://www.nytimes.com  # Create/login to a Chrome auth profile
rivulet scrape --auth-profile nyt --limit 10         # Scrape with saved Chrome session
rivulet tui               # Launch terminal UI
rivulet scrape            # Scrape full article content
rivulet daemon start      # Start background updater
rivulet daemon stop       # Stop daemon
rivulet daemon status     # Check daemon status
```

## Content Scraping

Many RSS feeds only include summaries. Rivulet can scrape full articles:

```bash
# Manual scraping
rivulet scrape --feed "https://beej.us/blog/rss.xml" --limit 10

# Scrape paid/private sites using a saved authenticated Chrome profile
rivulet auth add my-site --site "https://example.com/login"
rivulet auth check my-site --url "https://example.com/account"
rivulet scrape --auth-profile my-site --limit 10

# Enable automatic background scraping in config
# ~/.config/rivulet/config.toml
[scraper]
enabled = true
```

Auth profiles launch a visible Chrome window for manual login. Rivulet stores only profile metadata in SQLite; session cookies remain in the Chrome user data directory under Rivulet's data directory unless you pass `--profile-dir`.

## Configuration

Copy the sample config:

```bash
cp config.sample.toml ~/.config/rivulet/config.toml
```

Customize colors, keybindings, and scraper settings.

## Documentation

- [User Guide](docs/user-guide.md) - Complete usage documentation
- [Scraper](docs/scraper.md) - Content scraping configuration
- [Scheduling](docs/scheduling.md) - Automatic updates setup

## TUI Keybindings

| Key | Action |
|-----|--------|
| `j`/`k` | Move down/up |
| `Tab` | Next pane |
| `Enter` | Select |
| `r` | Toggle read |
| `s` | Toggle star |
| `L` | Toggle queued/read-later |
| `S` | Toggle saved |
| `x` | Toggle archived |
| `a`/`u`/`f`/`l`/`v`/`X` | View all/unread/starred/queued/saved/archived |
| `o` | Open in browser |
| `R` | Refresh feeds |
| `m` | Maximize preview |
| `q` | Quit |

## Requirements

- Rust 1.70+
- Chrome/Chromium (optional, for content scraping)

## License

MIT
