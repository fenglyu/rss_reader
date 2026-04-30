# Rivulet

A terminal-first, offline-first RSS / Atom feed reader written in Rust, with full-text content scraping, a vim-style TUI, and a built-in background daemon.

```
┌─[ Latest ]──[ Reader ]───────────────────────────────────────────┐
│ Feeds (12)         │ Items: All (87)   │ Preview                 │
│                    │                   │                         │
│ > Rust Blog (5)    │ ● 04/29 Announcing│ # Rust 1.86.0 released  │
│   Hacker News (42) │   04/27 This Week │                         │
│   Beej's Blog      │ ● 04/22 Rust RPN..│ The Rust team has...    │
├────────────────────┴───────────────────┴─────────────────────────┤
│ [/]:Tabs  \:Feeds  j/k:Nav  R:Refresh  Ctrl+W h/l:Focus  q:Quit  │
└──────────────────────────────────────────────────────────────────┘
```

## Features

- **Three-pane TUI** with vim-style navigation (`j`/`k`/`g`/`G`, `Ctrl+W h`/`l` to jump panes)
- **Offline-first** — feeds, items, item state, and scraped content all live in a local SQLite database
- **Full-article scraping** — headless Chrome via `chromiumoxide`, with optional authenticated profiles for paid/private sites
- **Two reading surfaces** — *Latest* (recently refreshed across all feeds) and *Reader* (drill into a single feed)
- **Reading workflow** — read / unread / starred / queued / saved / archived per item
- **Background daemon** — auto-refresh on a schedule
- **OPML import** — bring your subscriptions over from any other reader
- **Configurable** — colors, keybindings, scraper selectors, and refresh windows in a single TOML

## Install

### From source

```bash
git clone <repo-url> rivulet
cd rivulet
cargo install --path .
```

### Build only

```bash
cargo build --release
# binary → target/release/rivulet
```

Requirements:
- Rust 1.70+
- Google Chrome or Chromium on `$PATH` (only required for content scraping)

## Quick start

```bash
# Add a feed
rivulet add https://blog.rust-lang.org/feed.xml

# Or import an OPML export from another reader
rivulet import feeds.opml

# Pull the latest items from every feed
rivulet update

# Launch the TUI
rivulet tui
```

The first run creates `~/.config/rivulet/config.toml` with the default keybindings, colors, and scraper settings.

## Keyboard shortcuts

The TUI is keyboard-driven. Every binding below is configurable in `config.toml` under `[keybindings]` — see [`config.sample.toml`](config.sample.toml).

### Pane & tab navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down (next item) |
| `k` / `↑` | Move up (previous item) |
| `g` | Jump to top of the current list / preview |
| `G` / `%` | Jump to bottom |
| `n` / `PageDown` | Page down (10 items / 10 lines) |
| `p` / `PageUp` | Page up |
| `Tab` | Cycle focus to the next pane (Feeds → Items → Preview) |
| `Shift+Tab` | Cycle focus to the previous pane |
| `←` / `Left` / `h` | Focus pane to the **left** (loads the highlighted feed if needed) |
| `→` / `Right` / `l` | Focus pane to the **right** (loads the highlighted feed if needed) |
| `Ctrl+W` then `h` | Focus the pane to the **left** (vim window chord) |
| `Ctrl+W` then `l` | Focus the pane to the **right** |
| `Ctrl+W` then `w` / `Tab` | Cycle focus to the next pane |
| `Ctrl+W` then `W` / `Shift+Tab` | Cycle focus to the previous pane |
| `Ctrl+W` then `Esc` | Cancel a pending window chord |
| `Alt+1` / `[` | Switch to the **Latest** tab |
| `Alt+2` / `]` | Switch to the **Reader** tab (also opens the feed rail) |
| `\` | Expand / collapse the feed rail in Reader |
| `m` | Toggle maximize mode (fullscreen preview) |

> The `Ctrl+W` chord works like vim's window chord: press it, the status bar shows `-- WINDOW --`, then the next key picks a direction or cycles panes. Directional `h`/`l` does **not** wrap — pressing `Ctrl+W h` while already on the leftmost pane stays put.

### Reading actions

| Key | Action |
|-----|--------|
| `Enter` | Feeds pane: open the highlighted feed (loads items, focuses Items). Items pane: focus the Preview pane. |
| `r` | Toggle **read** / unread |
| `s` | Toggle **star** |
| `L` | Toggle **queued** (read-later) |
| `S` | Toggle **saved** |
| `x` | Toggle **archived** |
| `o` | Open the item link in the system browser (also marks read) |
| `R` | Refresh all feeds |
| `d` / `Delete` | Delete the highlighted feed (asks for `y` / `n` confirmation) |

### Filter views

These re-filter the items pane (and the Latest tab) without leaving your selection:

| Key | View |
|-----|------|
| `a` | All items (default) |
| `u` | Unread only |
| `f` | Starred |
| `Q` | Queued / read-later |
| `v` | Saved |
| `X` | Archived |

### General

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Ctrl+C` | Quit |

### Item markers

| Marker | Meaning |
|--------|---------|
| `NEW` | Inserted by the latest refresh batch (Latest tab only) |
| `.` | Unread |
| `*` | Starred |
| `Q` | Queued |
| `S` | Saved |
| `x` | Archived |
| _(blank)_ | Read |

## Command-line reference

```bash
# Feeds
rivulet add <URL>                # Add a feed
rivulet remove <URL>             # Remove a feed
rivulet import <FILE.opml>       # Import OPML
rivulet list                     # List feeds
rivulet list --items             # List items across all feeds
rivulet list --unread            # Filter to unread
rivulet list --queued            # Filter to read-later

# Sync & search
rivulet update                   # Refresh all feeds
rivulet search <QUERY>           # FTS over titles, summaries, scraped content
rivulet search rust --unread     # Combine search with a filter

# Content scraping
rivulet scrape --limit 10                                     # Scrape un-scraped items
rivulet scrape --feed "https://beej.us/blog/rss.xml"          # Scope to one feed
rivulet auth add my-site --site https://example.com/login     # Open Chrome to log in
rivulet auth check my-site --url https://example.com/account  # Verify the session
rivulet scrape --auth-profile my-site --limit 10              # Use a saved profile

# Daemon (background refresh)
rivulet daemon start
rivulet daemon stop
rivulet daemon status

# TUI
rivulet tui
```

Add `-w <N>` / `--workers <N>` to any sync command to tune fetch parallelism (default `10`).

## Config & data locations

| Platform | Config | Database |
|----------|--------|----------|
| macOS | `~/.config/rivulet/config.toml` | `~/Library/Application Support/rivulet/rivulet.db` |
| Linux | `~/.config/rivulet/config.toml` | `~/.local/share/rivulet/rivulet.db` |
| Windows | `%APPDATA%\rivulet\config.toml` | `%APPDATA%\rivulet\rivulet.db` |

The full default config — including every keybinding, color, and scraper option — lives in [`config.sample.toml`](config.sample.toml).

## Development

### Project layout

```
src/
├── cli/          # Subcommands behind `rivulet <cmd>` (clap)
├── config/       # TOML config — colors, keybindings, scraper, ui
├── daemon.rs     # Background refresh process
├── domain/       # Core types: Feed, Item, ItemState
├── fetcher/      # HTTP / RSS-Atom fetching, parallel orchestrator
├── normalizer/   # feed-rs → domain-model conversion + dedup hashing
├── scraper/      # Headless-Chrome article extraction
├── store/        # SQLite layer (rusqlite + rusqlite_migration)
└── tui/
    ├── app.rs    # TuiApp state machine — panes, selections, item state cache
    ├── event.rs  # Key event channel + Action enum
    ├── layout.rs # ratatui rendering
    └── mod.rs    # Event-loop wiring (fetch → store → render)
migrations/       # Versioned SQLite schemas (rusqlite_migration)
```

### Common tasks

```bash
# Run with verbose logging
RUST_LOG=rivulet=debug cargo run -- tui

# Unit tests (89 currently)
cargo test

# Lint
cargo clippy --all-targets

# Format
cargo fmt
```

### Adding a keybinding

1. Add a field to `KeybindingConfig` and a default in `src/config/keybindings.rs`
2. Add a variant to `Action` in `src/tui/event.rs`
3. Map the key → action in `KeybindingConfig::get_action`
4. Handle the action in the event loop in `src/tui/mod.rs`
5. Document it in this README and in `SHORTCUTS.md`
6. Add a test in the `test_keybinding_config_get_action` table

### Adding a database migration

Drop a new directory under `migrations/NNN-name/` containing `up.sql` (and optionally `down.sql`). The store applies pending migrations on startup via `rusqlite_migration`.

## Contributing

Pull requests are welcome. Before opening one:

- `cargo fmt && cargo clippy --all-targets -- -D warnings`
- `cargo test`
- Update `SHORTCUTS.md` and the keybindings table in this README if you touch the TUI

For larger changes, please open an issue first to discuss the design.

## License

MIT
