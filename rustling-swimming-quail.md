# Rivulet - Terminal RSS Client Implementation Plan

## Overview
Build a terminal-first, offline-first RSS/Atom client in Rust following a modular architecture:
**Fetcher → Normalizer → Store → Index → UI**

**Stack:** Rust + ratatui + crossterm + feed-rs + rusqlite + tokio + reqwest + tracing

---

## Phase 1: Project Setup & Core Infrastructure

### 1.1 Initialize Cargo Project
```bash
cargo new rivulet
```

### 1.2 Cargo.toml Dependencies
```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["gzip", "brotli"] }
feed-rs = "2.3"
rusqlite = { version = "0.32", features = ["bundled"] }
rusqlite_migration = "1.3"
ratatui = "0.29"
crossterm = "0.28"
anyhow = "1.0"
thiserror = "2.0"
sha2 = "0.10"
hex = "0.4"
chrono = { version = "0.4", features = ["serde"] }
url = "2.5"
async-trait = "0.1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
serde = { version = "1.0", features = ["derive"] }
html_escape = "0.2"
open = "5"
dirs = "5"

[dev-dependencies]
tempfile = "3.14"
tokio-test = "0.4"
```

### 1.3 Directory Structure
```
src/
  main.rs
  lib.rs
  app/mod.rs, context.rs, error.rs
  domain/mod.rs, feed.rs, item.rs, state.rs
  fetcher/mod.rs, http_fetcher.rs
  normalizer/mod.rs
  store/mod.rs, sqlite.rs
  tui/mod.rs, app.rs, layout.rs, event.rs
  cli/mod.rs, commands.rs
migrations/001-initial/up.sql
```

---

## Phase 2: Domain Models & Store (Checkpoint: `rivulet add <url>`)

### Files to Create (in order):
1. `src/app/error.rs` - RivuletError enum with thiserror
2. `src/app/mod.rs` - Module exports
3. `src/domain/feed.rs` - Feed struct (id, url, title, etag, last_modified)
4. `src/domain/item.rs` - Item struct + SHA256 ID generation
5. `src/domain/state.rs` - ItemState struct (read, starred)
6. `src/domain/mod.rs` - Re-exports
7. `migrations/001-initial/up.sql` - SQLite schema with foreign keys
8. `src/store/mod.rs` - Store trait definition
9. `src/store/sqlite.rs` - Full SQLite implementation

### Key Implementation Details:
- Item ID: `SHA256(feed_url + entry_id_or_link)` for determinism
- Foreign keys enforced in SQLite
- Separate `item_state` table for clean read/starred updates
- In-memory SQLite for tests via `SqliteStore::in_memory()`

---

## Phase 3: Fetcher & Normalizer

### Files to Create:
1. `src/fetcher/mod.rs` - Fetcher trait + FetchResult enum
2. `src/fetcher/http_fetcher.rs` - reqwest with ETag/If-Modified-Since
3. `src/normalizer/mod.rs` - Convert feed-rs output to domain Items

### Key Implementation Details:
- 10s timeout per feed
- Respect HTTP 304 Not Modified
- Support both RSS and Atom via feed-rs

---

## Phase 4: CLI (Checkpoint: Full CLI working)

### Files to Create:
1. `src/app/context.rs` - AppContext wiring Store + Fetcher + Normalizer
2. `src/cli/mod.rs` - clap derive CLI structure
3. `src/cli/commands.rs` - Command handlers
4. `src/main.rs` - Entry point

### CLI Commands:
```
rivulet add <url>     # Add and fetch a feed
rivulet remove <url>  # Remove a feed
rivulet update        # Update all feeds
rivulet list          # List feeds
rivulet list --items  # List items
rivulet tui           # Launch TUI
```

---

## Phase 5: TUI (Checkpoint: Interactive browser)

### Files to Create:
1. `src/tui/app.rs` - TuiApp state (ActivePane, selection indices)
2. `src/tui/event.rs` - Keyboard event handling
3. `src/tui/layout.rs` - Three-pane vertical layout
4. `src/tui/mod.rs` - Main render loop

### Three-Pane Layout:
```
┌──────────────┐
│ Feeds        │  (compact, ~6 lines)
├──────────────┤
│ Item List    │  (40% height)
├──────────────┤
│ Preview Pane │  (remaining)
└──────────────┘
```

### Keybindings:
- `j/k` or arrows: Navigate
- `Tab`: Cycle panes
- `Enter`: Select/expand
- `r`: Mark read
- `o`: Open in browser
- `R`: Refresh feeds
- `q`: Quit

---

## Phase 6: Parallel Fetch with ETag Support

### Files to Create:
1. `src/fetcher/parallel.rs` - ParallelFetcher with tokio Semaphore

### Key Implementation:
- `MAX_CONCURRENT_FETCHES = 5`
- Uses `tokio::sync::Semaphore` for bounded concurrency
- Returns `Vec<(feed_id, new_item_count)>` for progress reporting

---

## Testing Strategy

### Unit Tests (in-module):
- `src/domain/item.rs` - ID generation determinism
- `src/store/sqlite.rs` - CRUD operations with in-memory DB
- `src/normalizer/mod.rs` - RSS and Atom parsing

### Integration Test:
```bash
cargo run -- add https://blog.rust-lang.org/feed.xml
cargo run -- list
cargo run -- list --items
cargo run -- update
cargo run -- tui
```

---

## Verification Checklist

- [ ] `cargo build` succeeds with no warnings
- [ ] `cargo test` passes all unit tests
- [ ] `rivulet add <url>` fetches and stores items
- [ ] `rivulet list --items` shows items sorted by date desc
- [ ] `rivulet update` respects ETag (304 responses)
- [ ] `rivulet tui` displays three-pane layout
- [ ] Navigation with j/k/Tab works
- [ ] `o` opens item in browser
- [ ] `r` marks item as read
- [ ] `q` cleanly exits TUI
- [ ] No `unwrap()` in production code

---

## Critical Files Summary

| File | Purpose |
|------|---------|
| `src/store/sqlite.rs` | Core persistence, Store trait impl |
| `src/domain/item.rs` | Item struct + SHA256 ID generation |
| `src/fetcher/http_fetcher.rs` | HTTP client with conditional headers |
| `src/tui/mod.rs` | Main TUI render loop |
| `src/app/context.rs` | Application context wiring |
| `migrations/001-initial/up.sql` | Database schema |
