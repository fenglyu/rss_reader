---
name: rivulet-reading-workflow
description: Use this skill when changing Rivulet item states, reading queues, saved/archive behavior, list filters, TUI item views, or daily reading habit-loop features.
---

# Rivulet Reading Workflow

## Overview

Use this workflow for changes that affect how users triage, revisit, or hide articles: unread/read, starred, queued, saved, archived, item-list filters, TUI state toggles, and related documentation.

## Workflow

1. Map the state path before editing. Start with `src/domain/state.rs`, `src/store/mod.rs`, `src/store/sqlite.rs`, `src/cli/mod.rs`, `src/cli/commands.rs`, `src/tui/app.rs`, `src/tui/mod.rs`, `src/tui/layout.rs`, and `src/config/keybindings.rs`.
2. Keep behavior local-first and database-backed. Any new durable state needs a migration under `migrations/`, store trait methods, SQLite implementation, and unit coverage.
3. Preserve list semantics. `All`, `Unread`, `Starred`, `Queued`, and `Saved` exclude archived items by default; `Archived` is the explicit archived view.
4. Preserve cursor stability. TUI toggle handlers should update in-memory state without resetting the current feed/item selection unless the visible filtered list must change.
5. Update every surface together. If a state appears in the store, make sure CLI flags, TUI markers/keybindings, docs, and sample config remain consistent.
6. Add or update regression tests in `src/store/sqlite.rs` for persistence and filter semantics before relying on manual TUI behavior.

## Guardrails

- Do not add a new abstraction if extending `ItemListFilter`, `ItemState`, or existing store methods is sufficient.
- Do not make archived items visible in active reading views unless the command or view explicitly asks for archived content.
- Avoid special-casing state only in the TUI; the store/query layer should own durable filtering rules.
- Keep keybindings discoverable in `docs/USER_GUIDE.md` and default config generation in `src/config/mod.rs`.

## Verification

Run these before reporting completion:

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

For TUI-only behavior, also state what was covered by unit tests and what remains manual.

## Reference

Load `references/reading-workflow-map.md` when you need the current implementation map or acceptance checklist.
