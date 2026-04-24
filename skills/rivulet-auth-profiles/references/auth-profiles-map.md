# Rivulet Auth Profiles Map

## Current Files

- `migrations/004-auth-profiles/up.sql`: stores auth profile metadata.
- `src/domain/auth.rs`: defines `AuthProfile`.
- `src/store/mod.rs`: declares auth profile CRUD and status operations.
- `src/store/sqlite.rs`: persists profiles and statuses.
- `src/cli/mod.rs`: exposes `rivulet auth add/check/list` and `scrape --auth-profile`.
- `src/cli/commands.rs`: wires profile metadata to scraper config and opens visible login.
- `src/scraper/config.rs`: carries optional `user_data_dir`.
- `src/scraper/chrome.rs`: passes `user_data_dir` to Chromium.
- `src/config/mod.rs`: documents optional scraper profile directory config.
- `docs/USER_GUIDE.md`: documents authenticated scraping behavior.

## Acceptance Checklist

- `auth add` stores metadata and opens visible Chrome for manual login.
- `auth check` reuses the stored profile directory.
- `scrape --auth-profile <name>` reuses the profile through `ScraperConfig.user_data_dir`.
- Rivulet does not store passwords or raw cookie values.
- Profiles are isolated by directory.
- Missing profile and scrape/auth failures produce actionable errors.
