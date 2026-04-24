---
name: rivulet-search-index
description: Use this skill when changing Rivulet local search, SQLite FTS5 indexing, search filters, ranking, scraped-content indexing, or search CLI/TUI behavior.
---

# Rivulet Search Index

## Overview

Use this workflow for full-text search work over feed items, including schema migrations, index refresh paths, query behavior, ranking, CLI output, and future TUI search mode.

## Workflow

1. Inspect the index contract in `migrations/003-search-index/up.sql` and the sync methods in `src/store/sqlite.rs`.
2. Keep indexing explicit and testable. Insert/update/delete paths should refresh `item_search` in the same SQLite transaction or clearly documented call path.
3. Search the same item universe as list filters. Use `ItemListFilter` so archived items stay excluded from normal search unless `--archived` is passed.
4. Preserve scraped-content recall. `update_item_content` must refresh the FTS row so `rivulet search <query>` can find article body text, not only RSS summaries.
5. Treat query input as user data. Bind search terms through rusqlite parameters; do not interpolate raw user queries into SQL.
6. Update CLI/help/docs together when adding flags, ranking behavior, snippets, rebuild commands, or TUI search.

## Design Notes

- Current FTS table: `item_search(item_id, title, author, summary, content, feed_title, link)`.
- Current ranking: `ORDER BY bm25(item_search), i.published_at DESC, i.fetched_at DESC`.
- Future rebuild support should reuse store-level indexing helpers rather than duplicating SQL in the CLI.

## Verification

Run these before reporting completion:

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

Add focused tests in `src/store/sqlite.rs` for any changed insert, scrape update, delete, filter, ranking, or query parsing behavior.

## Reference

Load `references/search-index-map.md` for the current implementation map and acceptance checklist.
