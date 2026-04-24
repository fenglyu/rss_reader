# Rivulet Growth And Paid-Site Integration Plan

## Requirements Summary

Rivulet is currently a terminal-first, offline-first RSS/Atom reader with SQLite persistence, a Ratatui TUI, a CLI, background feed updates, and Chrome-based full-text scraping. Evidence:

- Feed/item/state persistence exists in SQLite, but the schema only models feeds, items, and read/starred state; there are no folders, tags, search index, auth profiles, sessions, scrape attempts, or source-specific rules. See `migrations/001-initial/up.sql:1`.
- The TUI supports navigation, read/star toggles, open-in-browser, refresh, feed delete, and feed selection. There is no filter/search/queue workflow in the event loop. See `src/tui/mod.rs:49`.
- The CLI supports init/add/remove/import/update/list/tui/daemon/scrape, but no auth, cookies, export, search, tagging, or digest commands. See `src/cli/mod.rs:19`.
- The Chrome scraper launches a new Chromium session from config and extracts content via page evaluation. It does not persist browser profile data or load source-specific cookies. See `src/scraper/chrome.rs:22`.
- Scraper config already has the right extension point for richer behavior: headless/visible mode, selectors, resource blocking, wait time, and user agent. See `src/scraper/config.rs:7`.

Primary goal: make Rivulet useful enough to open daily by improving triage, recall, and "read later" habits, then add authenticated paid-site scraping through explicit user-driven Chrome login flows with persistent cookies/profile data.

## Guiding Principles

- Local-first by default: store content, state, sessions metadata, and search indexes locally.
- No password storage: paid-site integration should use interactive browser login and persistent browser profile/cookies, not captured credentials.
- Respect boundaries: do not bypass paywalls; use the user's authenticated session only for sites they can access.
- Prefer small schema migrations and clear commands over new services or dependencies.
- Build habit loops before platform integrations: search, filters, queue, and digests will increase daily use more reliably than auth alone.

## Prioritized Feature Recommendations

### P0: Reading Workflow Upgrade

Add inbox-style filters and queues:

- `Unread`, `Starred`, `Saved`, `Today`, `Recently scraped`, and `Failed scrape` virtual feeds.
- TUI quick filters: unread-only, starred-only, search mode, source filter, and "next unread".
- Bulk actions: mark feed/all-visible as read, star/unstar visible, delete old read items.
- Reading queue: "send to queue" and "archive from queue" state separate from read/starred.

Why this helps usage: the app becomes a daily triage surface instead of a passive feed dump.

Implementation shape:

- Add state fields/table for `saved`, `queued`, `archived`, and timestamps.
- Add store queries for virtual feeds rather than hardcoding filters in the TUI.
- Extend `Action` and keybindings in `src/tui/event.rs` and `src/config/keybindings.rs`.
- Add CLI filters to `rivulet list --items`, for example `--unread`, `--starred`, `--since`, `--feed`.

Acceptance criteria:

- User can open TUI and jump directly through unread items across all feeds.
- User can toggle unread/starred/queued states without losing current cursor position.
- CLI can list unread/starred/queued items and mark visible/read items.
- Store tests cover all new state transitions.

### P1: Full-Text Search And Recall

Add local full-text search over title, author, summary, scraped content, feed title, and link.

Implementation shape:

- Add SQLite FTS5 virtual table for items, populated when items are inserted or scraped.
- Add `rivulet search <query>` with flags for `--feed`, `--unread`, `--starred`, `--limit`.
- Add TUI `/` search mode and search results as a temporary item list.
- Update `update_item_content` path to refresh the FTS index after scraping.

Acceptance criteria:

- Search finds terms from scraped article content, not only RSS summaries.
- Search result navigation reuses the normal preview pane.
- Updates and imports keep the index consistent.
- Tests cover insert, content update, delete cascade, and query ranking basics.

### P2: Authenticated Chrome Profiles For Paid Sites

Add first-class auth profiles that let the user log in interactively once, then reuse cookies for scraping paid sites.

User workflow:

- `rivulet auth add <name> --site https://example.com --visible`
- Rivulet launches a visible Chrome window with a dedicated persistent user-data directory.
- User logs in manually.
- `rivulet auth check <name>` verifies that configured test URLs are accessible.
- Feed/source config maps domains or URL patterns to an auth profile.
- Background scraper uses the matching profile when scraping item links.

Implementation shape:

- Extend scraper config with `profiles_dir`, `default_profile`, and per-site auth rules.
- Add `auth_profiles` table with `name`, `site_url`, `domain_pattern`, `profile_dir`, `created_at`, `last_checked_at`, and `status`.
- Add `feed_auth_rules` or generic `source_rules` table mapping feed/domain/url pattern to `auth_profile_id`.
- Extend `ChromeScraper::new` to accept a profile/user-data-dir and launch Chromium with persistent browser state.
- Add a `ChromeSessionManager` wrapper so scraping can choose the right profile per URL and avoid cross-site cookie mixing.
- Keep cookies in the browser profile directory. Do not copy cookie values into the main SQLite DB unless a later encryption design is explicitly approved.

Acceptance criteria:

- User can create a profile, log in visibly, quit, and later scrape a matching article headlessly using that session.
- Auth profiles are isolated by directory and never share cookies by default.
- `auth check` reports actionable failures: expired login, blocked by CAPTCHA, selector extraction failure, or navigation failure.
- Background scraping skips or defers auth-required sources when the matching profile is missing or unhealthy.

Security constraints:

- Never ask for or store passwords.
- Restrict profile directories under the Rivulet data directory unless explicitly overridden.
- Redact cookie/profile paths from normal logs when verbose logging is not enabled.
- Add docs warning that profile directories contain sensitive browser session material.

### P3: Site Rules And Extraction Recipes

Add source-specific rules so paid sites and difficult blogs are reliable.

Implementation shape:

- Add TOML site rules by domain: auth profile, content selectors, remove selectors, wait condition, block resources, custom user agent, and optional "reader mode" preference.
- Add `rivulet rules test <url>` to open/scrape one URL and show extraction diagnostics.
- Persist scrape attempts with status, error, content length, and timestamp.

Acceptance criteria:

- A user can tune one site without changing global scraper defaults.
- Failed scrapes appear in a virtual feed/status view.
- Rule tests make it obvious whether login, navigation, or extraction failed.

### P4: Daily Digest And Habit Loop

Add a daily briefing that gives the app a reason to be opened.

Implementation shape:

- `rivulet digest --today --unread --starred --limit 20`
- Optional Markdown export to a file for notes apps.
- TUI "digest" virtual feed sorted by source freshness and personal signals.
- Later optional integrations: email-to-self, local notification, or Lark/Slack export. Keep these optional and outside the MVP.

Acceptance criteria:

- Digest includes title, source, date, short excerpt, link, and local read/star status.
- Digest can be generated offline from existing local data.
- Exported Markdown is deterministic enough for snapshot tests.

### P5: Feed Discovery And Source Health

Improve source management:

- `rivulet discover <site-url>` finds RSS/Atom/JSON feed links from HTML.
- Source health view: last fetched, last success, error count, new items/week, scrape success rate.
- Stale-feed cleanup suggestions.

Acceptance criteria:

- Adding a normal website can discover likely feed URLs.
- Broken feeds are visible and actionable.
- Health metrics use existing update/scrape records rather than live-only output.

## Recommended Implementation Sequence

1. Baseline tests and store seams:
   - Expand store tests for state transitions, item updates, and delete cascades.
   - Add CLI command unit tests where practical.
   - Add a mock fetcher test utility before touching feed/update flows.

2. Reading workflow MVP:
   - Add state schema migration.
   - Add store query methods for unread/starred/queued/archived views.
   - Add CLI list filters and TUI keybindings.
   - Verify with unit tests and a short manual TUI smoke test.

3. Search MVP:
   - Add FTS migration and indexing methods.
   - Wire index updates into insert, update content, and delete paths.
   - Add `search` CLI and TUI search mode.

4. Auth profile MVP:
   - Add auth profile config/schema/CLI commands.
   - Add visible login flow using a persistent Chrome profile.
   - Add URL-to-profile rule resolution.
   - Wire matching profiles into manual `scrape` first, then background scraping.

5. Site rules and diagnostics:
   - Add per-site scraper overrides.
   - Add scrape attempts table.
   - Add `rules test` and failed-scrape visibility.

6. Daily digest:
   - Add query builder for digest candidates.
   - Add Markdown output and TUI digest view.
   - Add optional scheduler/daemon integration after the command is reliable.

## Data Model Changes

Proposed migrations:

- `002-reading-workflow.sql`: extend item state or add `item_annotations` for queued/saved/archived.
- `003-search-index.sql`: FTS5 table and triggers or explicit sync methods.
- `004-scrape-attempts.sql`: scrape diagnostics and status history.
- `005-auth-profiles.sql`: auth profile metadata and source rule mappings.

Keep sensitive browser session data outside SQLite in Chrome profile directories. SQLite stores only metadata and mappings.

## Risks And Mitigations

- Auth sessions are sensitive. Mitigation: isolate per-profile user data dirs, never store passwords, document risk, redact paths/logs.
- Paid sites differ heavily. Mitigation: site rules and diagnostics before trying to make a universal scraper.
- Background scraping can trigger anti-bot friction. Mitigation: low concurrency per auth profile, backoff on failures, visible login/check commands.
- TUI complexity can grow quickly. Mitigation: push filtering/search into store/query layer and keep TUI as a state renderer.
- FTS consistency can drift. Mitigation: tests for insert/update/delete and a `rivulet search rebuild-index` recovery command.

## Verification Plan

- Run `cargo test` after each phase.
- Add migration tests using `SqliteStore::in_memory()` or temporary DB files.
- Add store-level tests for all new query methods.
- Add CLI parse tests for new commands.
- Add scraper unit tests around profile/rule resolution without launching Chrome.
- Add an ignored/manual integration test for visible login and authenticated scrape.
- Manual smoke tests:
  - Add/import feeds, update, TUI read/star/filter/search.
  - Create auth profile in visible mode, log in, run `auth check`, run authenticated scrape.
  - Restart app and verify profile still works.

## Execution Notes

Best first implementation target: P0 reading workflow plus P1 search. These give immediate daily-use value and create the query/state infrastructure needed for auth diagnostics.

Best second target: P2 auth profiles with manual visible login and persistent Chrome profile directories. Do not start by exporting/importing raw cookies; persistent browser profiles are safer and simpler for Chromium-based auth interaction.

Do not add third-party sync, summaries, AI ranking, or external notification dependencies until the local workflow is better. The current architecture is strongest as an offline-first terminal reader; grow that core first.
