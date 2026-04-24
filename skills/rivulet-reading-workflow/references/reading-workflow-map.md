# Rivulet Reading Workflow Map

## Current Files

- `migrations/002-reading-workflow/up.sql`: adds `is_queued`, `is_saved`, `is_archived`, and timestamp columns to `item_state`.
- `src/domain/state.rs`: defines `ItemState`.
- `src/store/mod.rs`: defines `ItemListFilter` and state mutation/query methods.
- `src/store/sqlite.rs`: owns state persistence, list filtering, and regression tests.
- `src/cli/mod.rs`: exposes `list` and `search` filter flags.
- `src/cli/commands.rs`: maps CLI flags to `ItemListFilter` and prints state markers.
- `src/config/keybindings.rs`: maps default TUI state/view actions.
- `src/tui/app.rs`: keeps in-memory item state for rendering and toggles.
- `src/tui/mod.rs`: handles TUI events and store updates.
- `src/tui/layout.rs`: renders queued/saved/archived markers.
- `docs/USER_GUIDE.md`: documents CLI filters and keybindings.

## Acceptance Checklist

- State changes persist after app restart.
- `All`, `Unread`, `Starred`, `Queued`, and `Saved` exclude archived items.
- `Archived` shows archived items.
- CLI and TUI use the same filter semantics.
- Toggle operations do not unexpectedly reset the user's current selection.
- Docs and generated default config stay in sync with keybindings.
