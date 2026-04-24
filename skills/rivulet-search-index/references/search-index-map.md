# Rivulet Search Index Map

## Current Files

- `migrations/003-search-index/up.sql`: creates and backfills the `item_search` FTS5 virtual table.
- `src/store/mod.rs`: declares `search_items`.
- `src/store/sqlite.rs`: refreshes FTS rows and implements filtered search.
- `src/cli/mod.rs`: defines `rivulet search <query>` and filter flags.
- `src/cli/commands.rs`: prints search results and state markers.
- `src/scraper/background.rs`: updates scraped item content, which should refresh FTS through the store.
- `docs/USER_GUIDE.md`: documents search behavior.

## Acceptance Checklist

- Newly inserted items are searchable by title, author, summary, feed title, and link.
- Scraped content becomes searchable after `update_item_content`.
- Archived items are excluded unless `--archived` is used.
- `--unread`, `--starred`, `--queued`, and `--saved` filters match list behavior.
- User query strings are bound parameters, not interpolated SQL.
- Tests cover index refresh for insert and scraped-content update paths.
