# Latest Refresh Page and Reader Layout Plan

## Context Findings

- Today's TUI is a *vertical* three-pane stack: feeds row, items row, preview, plus a 1-line status bar. See `src/tui/layout.rs:27-35` for the constraints, and `render_feeds_pane` / `render_items_pane` / `render_preview_pane` / `render_status_bar` defined further down in the same file. The user request implies a horizontal/column rework, not just adding a tab.
- A separate maximized mode (toggled by `m`) already collapses everything to preview + status bar. See `src/tui/layout.rs:13-24`. Latest/Reader tabs must coexist with this mode without breaking it.
- Default refresh keybinding is `R` (uppercase, no explicit modifier) — `keybindings.rs:60`. The matcher tolerates an extra `SHIFT`: `self.modifiers == (key.modifiers & !KeyModifiers::SHIFT)` at `src/config/keybindings.rs:141-145`. So `Shift+R` and `R` both fire `Action::Refresh`.
- Refresh flow in `src/tui/mod.rs`:
  - `Action::Refresh` arm starts at `mod.rs:190`, sets `is_refreshing`, spawns the progress relay task (`mod.rs:198-204`) and the fetch task that calls `parallel_fetcher.fetch_all(...)` at `mod.rs:217`, then sends `AppEvent::RefreshComplete` at `mod.rs:220`.
  - `AppEvent::RefreshComplete` handler starts at `mod.rs:242`, aggregates `total_new`, queues post-refresh scraping, calls `load_feeds` / `load_current_items` at `mod.rs:272-273`, and posts the status message at `mod.rs:276` (`Refreshed: {} new items`).
  - Progress events (`AppEvent::RefreshProgress`) update `tui_app.refresh_progress` at `mod.rs:239-241`; the gauge is rendered inside `render_status_bar` (`layout.rs:255-278`).
- `Action` enum lives in `src/tui/event.rs:57-82`. `AppEvent` is in the same file (`event.rs:7-13`) and currently carries `RefreshComplete(Vec<(i64, Result<usize>)>)` — i.e. only per-feed counts, no inserted IDs.
- SQLite stores both `published_at` (nullable) and `fetched_at` (NOT NULL DEFAULT `datetime('now')`) on `items`. See `migrations/001-initial/up.sql`. Item queries already `ORDER BY published_at DESC, fetched_at DESC` — `src/store/sqlite.rs:99` and `src/store/sqlite.rs:104`.
- Existing migrations are `001-initial`, `002-reading-workflow`, `003-search-index`, `004-auth-profiles`. Each contains `up.sql` only — there is no `down.sql` convention in this repo. The next slot is `005`.
- `ParallelFetcher::fetch_all` returns `Vec<(i64, Result<usize>)>` — feed id + count. See `src/fetcher/parallel.rs:30-78`. The actual insert happens in `fetch_single_feed` at `parallel.rs:122` (`store.add_items(&items)?`), which discards which item ids landed.
- `SqliteStore::add_items` at `src/store/sqlite.rs:500-539` already collects `inserted_ids` for search-index refresh, then drops the vec and returns `usize`. Plumbing the IDs out is a small, contained change.
- Direct `add_items` callers besides `fetch_single_feed`:
  - `src/cli/commands.rs:72` (single-flow path that adds items outside the parallel fetcher).
- `fetch_all` callers:
  - `src/tui/mod.rs:217` (TUI refresh, with progress channel)
  - `src/cli/commands.rs:113` (CLI `update` command, no progress)
  - `src/cli/commands.rs:324` (CLI `import` command, no progress)
  - `src/daemon.rs:270` (daemon background update, no progress)
- Recent commit `739dcb4 add progress bar for refresh operations` already wires the gauge — reuse it as-is; no need to rebuild progress.

## Requirements Summary

User request (consolidated, including the Chinese-language original):

1. **New top-level tabs** — switchable via `Alt+1` (Latest) and `Alt+2` (Reader). The two views are siblings, not nested.
2. **Latest tab** — a "what's new" view backed by SQLite. It shows feed items from the last *N* days, sorted by recency. After a `Shift+R` refresh, the items inserted by *that specific* refresh batch float to the top of the list and are rendered with a distinct color/marker so they read as a diff. Selecting an item renders its content in the main area.
3. **Reader tab** — the existing feeds → items → content workflow, but reorganized so the feed list becomes a **collapsible left-edge rail** instead of a permanent full-width pane. By default the rail is collapsed and the screen is dominated by item list + content. A keybinding expands the rail, the user picks a feed, items reload for that feed, and the rail can collapse again.
4. **Persistence** — because storage is SQLite, the Latest tab must survive app restart: the most recent refresh batch is still identifiable as "the latest run" until a new refresh happens.
5. The `Shift+R` / `R` refresh behavior itself, including the progress bar, must keep working unchanged for users who don't care about the new tab.

Side convention: feed rail on the left, items in the middle, content/preview on the right. This matches the final implementation and keeps navigation in the conventional sidebar position.

## Proposed UX

Top-of-screen tab strip, always visible (1 line, above the body): `[ Latest ]  [ Reader ]`, with the active tab highlighted (active border color + bold). Hotkeys `Alt+1` / `Alt+2` switch tabs. The body region below the strip and above the status bar is what each tab renders into. Status bar and the existing maximized (`m`) mode keep working in both tabs.

### Latest tab (Horizontal layout)

- Two columns (Side-by-side):
  - Left (e.g. 40% width): recent item list. Items from the last *N* days (default `N=7`, configurable), capped at a configurable `limit` (default 200), ordered with the latest-refresh-batch items first, then everything else by `published_at DESC, fetched_at DESC`.
  - Right (remaining width, majority area): selected item's content, rendered with the same widget code as the Reader preview pane.
- Latest-batch items (from the most recent `Shift+R` / `R` run) render with a distinct foreground accent + a `NEW` marker prefix (color + text marker, so the cue still reads on color-blind / 8-color terminals) to act as a "diff" for new messages.
- Empty / cold-start states:
  - No refresh ever recorded: show the full N-day window using `fetched_at` / `published_at` only, and an inline hint line "No refresh batch recorded yet — press R to fetch.".
  - No items in the N-day window at all: show "No recent items in the last N days." centered.
- Per-item actions (`r` toggle read, `s` star, `L` queue, `S` save, `x` archive, `o` open) work the same as in the Reader tab. Opening an item marks it read just like the existing flow at `tui/mod.rs:180-186`.
- View-filter actions (`a/u/f/l/v/X`) apply to the Latest list as well, intersected with the N-day window.

### Reader tab (Horizontal layout with collapsible left rail)

- Body splits horizontally into three potential columns:
  - Left-edge feed rail: collapsible. Collapsed shows a 3-column-wide marker rail. Expanded shows the existing feed list (titles + unread counts) at about 30 columns wide.
  - Middle column: items list (the current items pane, repurposed).
  - Right/main area: preview / content pane for displaying text.
- Rail toggle action: new `Action::ToggleFeedPanel`, default key `\` (free, easy left-pinky reach). `Alt+f` is also free if the user prefers a mnemonic; left as a configurable secondary binding. `f` itself is taken by `ViewStarred` (`keybindings.rs:55`).
- Rail navigation: when expanded, normal `j`/`k` plus `Enter` selects a feed and reloads the items list for that feed (existing behavior, just reachable via the rail).
- When collapsed, the items list shows the currently selected feed's items (or all items if no feed is selected, matching today's behavior).
- `m` maximize keeps working: in the Reader tab, maximize behaves as today (preview-only). In the Latest tab, maximize hides the item list and shows only the content column.

### Cross-tab behavior

- Refresh (`R` / `Shift+R`) works identically on both tabs and uses the same progress gauge.
- After a refresh completes while the user is on the Latest tab, the list reloads automatically and the new batch appears at top (with the highlight) — no manual reload.
- Switching tabs preserves each tab's selection index, scroll, and feed-rail state.

## Data Model Decision

Do not rely solely on `fetched_at` for "this refresh" highlighting — `INSERT OR IGNORE` plus parallel feed updates make the boundary between two refreshes fuzzy at the row level. Persist explicit refresh-run metadata.

New migration: `migrations/005-refresh-runs/up.sql` (no `down.sql`, matching repo convention).

Tables:

- `refresh_runs(id INTEGER PRIMARY KEY, started_at TEXT NOT NULL, completed_at TEXT, total_feeds INTEGER NOT NULL, new_item_count INTEGER NOT NULL DEFAULT 0, error_count INTEGER NOT NULL DEFAULT 0, source TEXT NOT NULL)` — `source` is one of `tui`, `cli`, `daemon`, `import`.
- `refresh_run_items(refresh_run_id INTEGER NOT NULL, item_id TEXT NOT NULL, feed_id INTEGER NOT NULL, inserted_at TEXT NOT NULL DEFAULT (datetime('now')), PRIMARY KEY (refresh_run_id, item_id), FOREIGN KEY (refresh_run_id) REFERENCES refresh_runs(id) ON DELETE CASCADE, FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE)`.

Indexes:

- `idx_refresh_runs_completed_at` on `refresh_runs(completed_at DESC)` — to find the latest run.
- `idx_refresh_run_items_run` on `refresh_run_items(refresh_run_id)`.
- `idx_refresh_run_items_item` on `refresh_run_items(item_id)`.
- `idx_items_fetched_at` on `items(fetched_at DESC)` — accelerates the recent-N-days query.

The "latest run" is the row in `refresh_runs` with the largest non-null `completed_at`. We do not depend on a single global "current run" pointer; restart and concurrent CLI runs both work.

## Store/API Plan

Introduce structured result types. Keep existing `add_items(&[Item]) -> Result<usize>` as a thin wrapper so we don't rewrite every call site at once.

```rust
pub struct AddItemsResult { pub count: usize, pub inserted_ids: Vec<String> }
pub struct FeedRefreshResult { pub feed_id: i64, pub new_count: usize, pub inserted_item_ids: Vec<String> }
pub struct RecentItem { pub item: Item, pub feed_title: String, pub is_latest_refresh_item: bool, pub arrived_at: chrono::DateTime<chrono::Utc> }
```

`Store` trait additions:

- `add_items_with_report(&self, items: &[Item]) -> Result<AddItemsResult>` — new primary path; existing `add_items` becomes `Ok(self.add_items_with_report(items)?.count)`.
- `begin_refresh_run(source: &str, total_feeds: usize) -> Result<i64>`
- `complete_refresh_run(run_id: i64, new_item_count: usize, error_count: usize) -> Result<()>`
- `record_refresh_run_items(run_id: i64, feed_id: i64, item_ids: &[String]) -> Result<()>`
- `get_latest_refresh_run_id() -> Result<Option<i64>>`
- `get_recent_items(days: i64, limit: usize, latest_run_id: Option<i64>) -> Result<Vec<RecentItem>>` — single SQL with a `CASE WHEN ri.refresh_run_id = ?latest THEN 1 ELSE 0 END AS is_latest`, ordered by `is_latest DESC, COALESCE(published_at, fetched_at) DESC`, restricted to `fetched_at >= datetime('now', '-? days')` OR `is_latest = 1` (so the highlighted batch never gets clipped by the window).

`ParallelFetcher::fetch_all` returns `Vec<FeedRefreshResult>` instead of `Vec<(i64, Result<usize>)>`. Errors become `FeedRefreshResult { feed_id, new_count: 0, inserted_item_ids: vec![] }` with a sibling `Vec<(i64, RivuletError)>` returned alongside, *or* a `Result<FeedRefreshResult>` per feed (clearer). Adapt:

- `src/tui/mod.rs:217` (TUI) — wraps the call in `begin_refresh_run` / `record_refresh_run_items` / `complete_refresh_run`.
- `src/cli/commands.rs:113` (CLI `update`) — same wrap.
- `src/cli/commands.rs:324` (CLI `import`) — same wrap with `source = "import"`.
- `src/cli/commands.rs:72` (single-feed direct `add_items` call) — switch to `add_items_with_report`; if this is part of a user-driven flow, also wrap in a one-feed `refresh_run`.
- `src/daemon.rs:270` — wrap with `source = "daemon"`.

Recording refresh runs from CLI/daemon is **required**, not optional — without it, scheduled or background refreshes wouldn't surface in the Latest tab on the next TUI launch, breaking the persistence requirement.

## TUI State/Layout Plan

Extend `TuiApp` (`src/tui/app.rs`) with:

- `active_tab: AppTab` where `AppTab::{ Latest, Reader }`, default `Latest` on startup so users land on the new view.
- `feed_panel: FeedPanelState::{ Collapsed, Expanded }`, default `Collapsed`.
- Separate selection state per tab: `latest_items: Vec<RecentItem>`, `latest_index: usize`, `latest_list_state: ListState`; reuse the existing `items` / `item_index` / `item_list_state` for the Reader tab.
- `latest_run_id: Option<i64>` — refreshed after each `RefreshComplete` and on startup via `get_latest_refresh_run_id()`.
- `recent_days: i64`, `recent_limit: usize` — populated from config.

Extend `Action` (`src/tui/event.rs:57-82`) with:

- `Action::ViewLatest`
- `Action::ViewReader`
- `Action::ToggleFeedPanel`

Extend `KeybindingConfig` (`src/config/keybindings.rs:11-65`) with matching fields and defaults:

- `view_latest: vec!["Alt+1".to_string()]`
- `view_reader: vec!["Alt+2".to_string()]`
- `toggle_feed_panel: vec!["\\".to_string()]` (also accept `Alt+f` if the user adds it). Verify `Alt+1` / `Alt+2` parse cleanly through `parse_key_string` (they should — `Alt` is already a recognized modifier and `1`/`2` are single chars).

Layout refactor (`src/tui/layout.rs`):

- Wrap the existing render entry in a top-level vertical split: tab strip (1 line) / body / status bar.
- Body is dispatched on `app.active_tab`:
  - `render_latest_tab(frame, app, area, colors)` — horizontal split: items list (left) + content (right).
  - `render_reader_tab(frame, app, area, colors)` — horizontal split: collapsible feed rail (left edge) + items list + content preview; the rail's width is 3/30 columns depending on `Collapsed` / `Expanded`.
- Extract a shared `render_item_list(frame, area, items_iter, list_state, colors, highlight_predicate)` so both tabs use the same rendering and the Latest tab can pass a closure that returns a "this-refresh" style for matching IDs. The existing per-state styling (`unread_item`, `read_item`, marker chars in `layout.rs:121-148`) stays.
- Keep `render_status_bar` as-is; it already handles the refresh gauge.
- `m` (maximize) interpretation per tab:
  - Reader: today's behavior (preview only) — unchanged.
  - Latest: hides the item list, content fills the body.

Event handling additions in `src/tui/mod.rs`:

- Match `Action::ViewLatest` / `Action::ViewReader` to flip `active_tab` and re-render. When entering Latest, refresh `latest_run_id` and call `get_recent_items(...)` once if the cache is stale.
- Match `Action::ToggleFeedPanel` to toggle `feed_panel` (Reader tab only — no-op on Latest).
- After `AppEvent::RefreshComplete`, also refresh the Latest list and `latest_run_id`.
- Item-action handlers (`ToggleRead`, `ToggleStar`, etc.) need to operate on the active tab's selected item, not just `tui_app.selected_item()`. Add a helper `selected_item_for_active_tab()` and route through it.

Config additions (`config.sample.toml`):

```toml
[ui.latest]
days = 7
limit = 200
```

Default tab (`Latest` vs `Reader` on startup) can also be a config knob — default `Latest`.

## Implementation Steps

1. Land regression tests for current behavior before changing any APIs:
   - keybinding parse + `KeyBinding::matches` for `R` / `Shift+R` (the SHIFT-stripping rule).
   - `add_items` count + idempotency (duplicate item id ignored).
   - existing item ordering (`published_at DESC, fetched_at DESC`).
2. Add migration `005-refresh-runs/up.sql` and the new `Store` trait methods + `SqliteStore` impls.
3. Switch `SqliteStore::add_items` internals onto `add_items_with_report`; keep `add_items` as a wrapper.
4. Change `ParallelFetcher::fetch_all` return shape; adapt all four call sites (`tui/mod.rs:217`, `cli/commands.rs:113`, `cli/commands.rs:324`, `daemon.rs:270`) and the direct call at `cli/commands.rs:72`. Each call site wraps the work in `begin_refresh_run` → `record_refresh_run_items` → `complete_refresh_run`.
5. Add `AppTab`, `FeedPanelState`, the new state fields, the `Action` variants, and the default keybindings. Update `KeybindingConfig::get_action`.
6. Add `Action::ViewLatest`, `Action::ViewReader`, `Action::ToggleFeedPanel` handlers in `tui/mod.rs`. Reload Latest on completion of `RefreshComplete`.
7. Refactor `tui/layout.rs` into the tab strip + per-tab renderers + shared item-list helper.
8. Wire item-action routing to the active tab's selected item.
9. Update docs: `SHORTCUTS.md`, `README.md`, `config.sample.toml`, `CHANGELOG.md`.

## Acceptance Criteria

- `R` and `Shift+R` both still trigger refresh and show the existing progress gauge.
- `Alt+1` switches to Latest; `Alt+2` switches to Reader. The active tab is visibly indicated in the tab strip.
- After a refresh that inserted N new items, the Latest tab lists those N items at the top, marked with both an accent color and a `NEW` text marker. Items outside that batch but inside the recent-N-days window appear below, ordered by date.
- Restarting the app preserves the "latest batch" highlight: the most recent `refresh_runs` row is loaded and used until the next refresh.
- A refresh run started by `rivulet update`, by `rivulet import`, or by the daemon also populates `refresh_runs` / `refresh_run_items` and is visible from the TUI Latest tab on next launch.
- Reader tab has a horizontal layout with a collapsible left-edge feed rail (default key `\`). Collapsed is the default state. Expanding reveals feed titles + unread counts; selecting a feed reloads the items list. Collapsing returns the previously selected feed's items.
- Latest and Reader tabs share content rendering. View filters (`a/u/f/l/v/X`) and item state actions (`r/s/L/S/x/o`) work in both.
- `m` maximize works in both tabs (preview-only in Reader, content-only in Latest).

## Verification Plan

- Unit tests:
  - `keybindings.rs`: `Alt+1` → `Action::ViewLatest`, `Alt+2` → `Action::ViewReader`, `\` → `Action::ToggleFeedPanel`, regression tests for `R`/`Shift+R` → `Action::Refresh`.
  - `store::sqlite`: `add_items_with_report` returns the expected `inserted_ids` (and skips dupes), `begin_refresh_run` + `record_refresh_run_items` + `complete_refresh_run` round-trip, `get_latest_refresh_run_id` returns the most recent completed run, `get_recent_items` returns latest-batch items first then by date, never clipping the latest batch with the day window.
- Integration-style tests:
  - In-memory SQLite update flow: two feeds with overlapping items; second run inserts one new item; assert that exactly one item in `refresh_run_items` for the second run, dupes are not re-recorded, `get_recent_items` highlights only that one.
  - TUI app-state test (where the existing app harness allows): tab switch updates `active_tab`, feed panel toggle updates `feed_panel`, item-action routes to active tab.
- Manual TUI verification:
  - Launch, land on Latest, see existing recent items with no highlight (no run yet).
  - Press `R`, see progress gauge, see Latest auto-reload with new items pinned at top in accent color + `NEW` marker.
  - Press `Alt+2`, navigate items + content, press `\` to expand the feed rail, pick a different feed, press `\` to collapse, confirm items reflect the new feed.
  - Press `Alt+1`, confirm tab state is preserved on switch back.
  - Quit, relaunch, confirm Latest still highlights the same batch.
- Standard gates: `cargo fmt`, `cargo test`, `cargo clippy` if part of the repo workflow.

## Risks and Mitigations

- **Risk:** `fetch_all` return-type change ripples into CLI, daemon, import, TUI. **Mitigation:** keep `add_items` as a `usize`-returning wrapper, update each call site in one focused commit, cover with tests before refactor.
- **Risk:** `published_at` from feeds can be older than the actual fetch time, so latest-batch items might sort below older publish dates. **Mitigation:** `get_recent_items` orders by `is_latest_refresh_item DESC` first; within each group, by `COALESCE(published_at, fetched_at) DESC`. Display still shows the published date.
- **Risk:** TUI layout becomes cramped at small widths. **Mitigation:** rail collapsed by default, fall back to a single-column body when terminal width drops below a threshold (~80 cols).
- **Risk:** color-only "new" cue is inaccessible. **Mitigation:** combine accent foreground with a `NEW` text prefix (already in the spec).
- **Risk:** Daemon refreshes happening while the TUI is open could change `latest_run_id` mid-session. **Mitigation:** on `RefreshComplete` (TUI's own runs) refresh the cached `latest_run_id`; for daemon-driven changes, accept a small staleness window — Latest reloads on the next manual refresh or tab switch into Latest.
- **Risk:** `Alt+1` / `Alt+2` may collide with terminal emulator shortcuts (some terminals intercept `Alt+digit` for tab switching at the OS level). **Mitigation:** also expose configurable secondaries (e.g. `[`, `]`) in `keybindings.toml`; document the conflict in `SHORTCUTS.md`.
