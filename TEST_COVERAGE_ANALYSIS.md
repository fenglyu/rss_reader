# Test Coverage Analysis for Rivulet

## Current State

**29 tests** across **10 files** (out of 31 source files). All 29 tests pass.

### Tested Modules

| Module | File | Tests | What's Covered |
|--------|------|-------|----------------|
| config/colors | `colors.rs` | 4 | Named, hex, short hex, and invalid color parsing |
| config/keybindings | `keybindings.rs` | 9 | Key parsing, modifiers, function keys, matching, action resolution |
| config/mod | `mod.rs` | 3 | Default config deserializes, partial config, empty config |
| domain/item | `item.rs` | 3 | ID generation determinism, uniqueness, format validation |
| normalizer | `mod.rs` | 3 | RSS parsing, Atom parsing, item ID determinism |
| scraper/extractor | `extractor.rs` | 1 | JavaScript extraction script generation |
| store/sqlite | `sqlite.rs` | 4 | Feed CRUD, item CRUD, read state toggling, unread count |
| daemon | `daemon.rs` | 2 | Interval parsing, interval formatting |

### Untested Modules (0 tests)

| Module | File | LOC | Risk Level |
|--------|------|-----|------------|
| cli/commands | `commands.rs` | 431 | **High** — core user-facing operations |
| cli/mod | `mod.rs` | 101 | Medium — CLI argument definitions |
| fetcher/http_fetcher | `http_fetcher.rs` | 84 | **High** — HTTP fetching with conditional headers |
| fetcher/parallel | `parallel.rs` | 115 | **High** — concurrent feed fetching orchestration |
| scraper/chrome | `chrome.rs` | 234 | Medium — requires browser, hard to unit test |
| scraper/background | `background.rs` | 185 | Medium — background scraping orchestration |
| scraper/config | `config.rs` | 125 | Low — mostly configuration defaults |
| scraper/mod | `mod.rs` | 94 | Medium — `needs_scraping` logic |
| tui/app | `app.rs` | 211 | Medium — TUI application state |
| tui/event | `event.rs` | 47 | Low — event loop plumbing |
| tui/layout | `layout.rs` | 289 | Medium — layout rendering |
| tui/mod | `mod.rs` | 268 | Medium — TUI entry point |
| app/context | `context.rs` | 91 | Low — struct construction |
| app/error | `error.rs` | 36 | Low — error type definitions |
| domain/feed | `feed.rs` | 42 | Low — `display_title()` fallback logic |
| domain/state | `state.rs` | 23 | Low — trivial constructor |

---

## Recommended Improvements (Priority Order)

### 1. Store: Expand Coverage of Untested Operations (High Priority)

**Current gap:** Only 4 of ~15 `Store` trait methods are tested. Missing coverage for critical data operations.

**Tests to add in `store/sqlite.rs`:**

- `test_get_feed_by_url` — verifying URL-based feed lookup
- `test_get_all_feeds_ordering` — verifying feeds are returned sorted by title then URL
- `test_update_feed` — partial and full updates, verifying each field can be updated independently
- `test_delete_feed` — verifying cascade behavior (items belonging to the deleted feed)
- `test_add_items_batch` — batch insert counting, deduplication via `INSERT OR IGNORE`
- `test_add_duplicate_item` — verifying that inserting the same item twice doesn't create duplicates
- `test_get_items_by_feed_ordering` — verifying chronological ordering by `published_at DESC`
- `test_item_exists` — true/false for existing/missing items
- `test_set_starred` — toggle starred state, verify `starred_at` timestamp presence
- `test_update_item_content` — scrape content update roundtrip
- `test_get_all_items` — returns items from multiple feeds

These are all straightforward to write using the existing `SqliteStore::in_memory()` pattern.

### 2. OPML Parsing (High Priority)

**Current gap:** `parse_opml()` and `extract_attr()` in `cli/commands.rs` are pure functions with zero tests. They contain hand-rolled XML attribute parsing that is easy to break.

**Tests to add (new `#[cfg(test)]` module in `cli/commands.rs`):**

- `test_parse_opml_basic` — standard OPML with `xmlUrl` and `title` attributes
- `test_parse_opml_text_fallback` — uses `text` attribute when `title` is missing
- `test_parse_opml_html_entities` — `&amp;` and other entities in titles/URLs
- `test_parse_opml_empty` — empty/no-outline OPML returns empty vec
- `test_extract_attr_basic` — simple attribute extraction
- `test_extract_attr_missing` — returns None for missing attributes
- `test_parse_opml_nested_outlines` — OPML with category groups (nested `<outline>` tags)

### 3. Normalizer: Edge Cases (High Priority)

**Current gap:** Only happy-path RSS and Atom parsing tested. No coverage for malformed feeds, feeds with missing fields, HTML entity handling, or empty entry IDs.

**Tests to add in `normalizer/mod.rs`:**

- `test_parse_invalid_feed` — garbage bytes return `RivuletError::FeedParse`
- `test_parse_rss_missing_fields` — items with no title, no link, no guid
- `test_parse_rss_html_entities` — `&amp;`, `&lt;` in titles/descriptions decode correctly
- `test_parse_entry_empty_id_uses_link` — verifies the empty-ID fallback to link for hashing
- `test_parse_rss_with_content` — items that have `<content:encoded>` element
- `test_parse_atom_with_content` — entries with `<content>` element vs `<summary>`
- `test_normalize_preserves_author` — author field extraction

### 4. Domain Model: `display_*` Methods (Low Effort, Medium Value)

**Current gap:** `Item::display_title()`, `Item::display_content()`, and `Feed::display_title()` have fallback logic that is untested.

**Tests to add:**

In `domain/item.rs`:
- `test_display_title_with_title` — returns the title
- `test_display_title_without_title` — returns "(Untitled)"
- `test_display_content_prefers_content` — content takes priority over summary
- `test_display_content_falls_back_to_summary` — uses summary when content is None
- `test_display_content_empty_when_neither` — returns "" when both are None

In `domain/feed.rs`:
- `test_display_title_with_title` — returns the title
- `test_display_title_falls_back_to_url` — returns URL when title is None

### 5. Fetcher: Mock-Based Testing (Medium Priority)

**Current gap:** The `Fetcher` trait exists and is well-designed for testing, but no mock implementation is used anywhere. `ParallelFetcher` and the fetch orchestration in `parallel.rs` are untested.

**Tests to add in `fetcher/parallel.rs`:**

- Create a `MockFetcher` implementing the `Fetcher` trait
- `test_fetch_all_single_feed` — one feed, content returned, store updated
- `test_fetch_all_not_modified` — 304 response, no items added
- `test_fetch_all_multiple_feeds` — concurrent fetch of 3 feeds
- `test_fetch_all_with_error` — one feed errors, others succeed
- `test_fetch_single_feed_updates_metadata` — etag/last_modified stored correctly

This requires an `async` test with `#[tokio::test]` and a mock `Fetcher`+`Store` or using the in-memory `SqliteStore`.

### 6. Scraper Config and Extractor (Low Priority)

**Current gap:** `ScraperConfig` has helper methods (`timeout()`, `wait_after_load()`, `fast()`, `thorough()`) and preset constructors that are untested. The extractor has only 1 test.

**Tests to add:**

In `scraper/config.rs`:
- `test_default_config_values` — verify default field values
- `test_fast_config` — verify overrides in `fast()` preset
- `test_thorough_config` — verify overrides in `thorough()` preset
- `test_timeout_duration` — `timeout()` returns correct `Duration`
- `test_wait_after_load_duration` — `wait_after_load()` returns correct `Duration`

In `scraper/extractor.rs`:
- `test_resource_blocking_script_both` — when both images and stylesheets blocked
- `test_resource_blocking_script_none` — returns `None` when nothing blocked
- `test_resource_blocking_script_images_only` — only images blocked
- `test_extraction_script_custom_selectors` — custom content/remove selectors appear in script

### 7. Config Deserialization Robustness (Low Priority)

**Current gap:** Config tests cover happy paths but not error cases.

**Tests to add in `config/mod.rs`:**

- `test_invalid_toml_returns_error` — malformed TOML
- `test_invalid_color_in_config` — unrecognized color string
- `test_unknown_keys_ignored` — extra keys don't break deserialization (verify serde behavior)

---

## Structural Recommendations

### A. Introduce a `MockFetcher` Test Utility

The `Fetcher` trait is already defined as an `async_trait` with a clean interface. A reusable `MockFetcher` would unlock testing for `ParallelFetcher`, CLI commands (`add_feed`, `update_feeds`), and the scraper orchestration — all of which currently depend on real HTTP.

```rust
// In a tests/common/mod.rs or fetcher/mod.rs #[cfg(test)] block
struct MockFetcher {
    responses: HashMap<String, Result<FetchResult>>,
}
```

### B. Extract Pure Functions from CLI Commands

`cli/commands.rs` mixes pure logic (OPML parsing, output formatting) with I/O (HTTP fetching, stdout printing). Extracting `parse_opml()` and `extract_attr()` to a separate module (or making them `pub(crate)`) would make them directly testable without mocking.

### C. Consider Integration Tests

The project has no `tests/` directory for integration tests. A small set of integration tests that:
1. Create an in-memory store
2. Use a `MockFetcher` to simulate feed responses
3. Run the full add → fetch → normalize → store pipeline

...would cover the most critical user-facing path end-to-end.

---

## Summary

| Priority | Area | Tests to Add | Effort |
|----------|------|-------------|--------|
| **High** | Store operations | ~11 tests | Low — existing pattern |
| **High** | OPML parsing | ~7 tests | Low — pure functions |
| **High** | Normalizer edge cases | ~7 tests | Low — existing pattern |
| **Medium** | Domain display methods | ~7 tests | Trivial |
| **Medium** | Fetcher (mock-based) | ~5 tests | Medium — needs mock |
| **Low** | Scraper config/extractor | ~9 tests | Low |
| **Low** | Config error handling | ~3 tests | Low |

Total: **~49 new tests** recommended, which would bring the suite from 29 to ~78 tests, covering all modules with non-trivial logic.
