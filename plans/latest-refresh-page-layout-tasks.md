# Latest Refresh Page Task List

Source plan: `.omx/plans/latest-refresh-page-layout-plan.md`

## Tasks

- [x] T0 Baseline regression tests: lock current `R` / `Shift+R`, `add_items` de-duplication, and item ordering behavior.
- [x] T1 Database migration: add `migrations/005-refresh-runs/up.sql` with `refresh_runs`, `refresh_run_items`, and supporting indexes.
- [x] T2 Store API: add `RefreshSource`, `AddItemsResult`, `FeedRefreshResult`, `RecentItem`, and refresh-run methods.
- [x] T3 Insert reporting: make `add_items` delegate to `add_items_with_report` while preserving existing callers.
- [x] T4 Recent queries: implement latest-run lookup and recent-item query with batch pinning.
- [x] T5 Orphan run cleanup: sweep stale incomplete refresh runs on store open.
- [x] T6 Fetcher result upgrade: return per-feed inserted item IDs from `ParallelFetcher::fetch_all`.
- [x] T7 Refresh-run recording: wire TUI refresh, CLI update, CLI import, direct add, and daemon update into refresh runs.
- [x] T8 TUI state model: add `AppTab`, `FeedPanelState`, latest list state, and recent settings.
- [x] T9 Actions and keybindings: add `Alt+1`, `Alt+2`, and feed-panel toggle.
- [x] T10 Event handling: switch tabs, toggle rail, reload Latest after refresh, and route item actions by active tab.
- [x] T11 Layout refactor: render tab strip, Latest two-column view, Reader left feed rail + item list + content, and narrow fallback.
- [x] T12 Shared item behavior: filters and read/star/queue/save/archive/open work in both tabs with shared state.
- [x] T13 Docs/config: update shortcuts, README, changelog, and sample config.
- [x] T14 Verification: run formatting, tests, clippy where available, and capture remaining manual TUI checks.

## Execution Notes

- Keep `add_items(&[Item]) -> Result<usize>` as a compatibility wrapper.
- Use `fetched_at` for recent-window membership and explicit `refresh_run_items` for latest-batch highlighting.
- `limit` applies to non-batch tail rows only; all latest-batch rows remain visible.
- Feed rail is left-edge by confirmed convention.
