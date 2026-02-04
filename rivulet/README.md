# Rivulet

A terminal-first, offline-first RSS/Atom feed reader with full-text content scraping.

## Features

- **Terminal UI** - Three-pane interface with vim-style navigation
- **Offline-first** - All content stored locally in SQLite
- **Content Scraping** - Fetch full articles using headless Chrome
- **Background Updates** - Built-in daemon for automatic feed updates
- **OPML Import** - Import feeds from other readers
- **Configurable** - Customize colors, keybindings, and scraper settings

## Quick Start

```bash
# Build
cargo build --release

# Add a feed
./target/release/rivulet add https://blog.rust-lang.org/feed.xml

# Update feeds
./target/release/rivulet update

# Launch TUI
./target/release/rivulet tui
```

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
rivulet add <URL>         # Add a new feed
rivulet remove <URL>      # Remove a feed
rivulet import <FILE>     # Import OPML file
rivulet update            # Update all feeds
rivulet list              # List feeds
rivulet list --items      # List all items
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

# Enable automatic background scraping in config
# ~/.config/rivulet/config.toml
[scraper]
enabled = true
```

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
| `o` | Open in browser |
| `R` | Refresh feeds |
| `m` | Maximize preview |
| `q` | Quit |

## Requirements

- Rust 1.70+
- Chrome/Chromium (optional, for content scraping)

## License

MIT
