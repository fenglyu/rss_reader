---
name: rivulet-auth-profiles
description: Use this skill when changing Rivulet Chrome auth profiles, paid/private-site scraping, persistent browser sessions, cookie handling, or scraper auth flows.
---

# Rivulet Auth Profiles

## Overview

Use this workflow for authenticated scraping through user-driven Chrome login sessions. Rivulet should reuse browser profile directories, not passwords or exported cookie blobs.

## Workflow

1. Map the auth path before editing: `src/domain/auth.rs`, `migrations/004-auth-profiles/up.sql`, `src/store/mod.rs`, `src/store/sqlite.rs`, `src/cli/mod.rs`, `src/cli/commands.rs`, `src/scraper/config.rs`, and `src/scraper/chrome.rs`.
2. Keep login explicit. `rivulet auth add <name> --site <url>` opens visible Chrome and waits for the user to finish login.
3. Keep session material in Chrome. Store profile metadata in SQLite, but never store passwords or raw cookie values in Rivulet tables, config, tests, docs, or logs.
4. Keep profiles isolated. Each auth profile should have its own `user_data_dir`; do not share cookies across sites by default.
5. Reuse scraper config. Authenticated scrape paths should flow through `ScraperConfig.user_data_dir` and `ChromeScraper`, not a separate cookie importer.
6. Make failures actionable. `auth check` and scrape errors should distinguish missing profile, expired login, navigation failure, extraction failure, and likely CAPTCHA/blocking cases where feasible.

## Security Guardrails

- Do not implement paywall bypass logic. Rivulet may use the user's authenticated session only for sites they can access.
- Do not print full sensitive profile paths unnecessarily; prefer profile names in user-facing output unless the path is intentionally requested.
- Do not add background scraping against auth profiles without considering concurrency, backoff, and account-friction risk.
- Do not add browser automation that submits credentials. The user performs login manually in Chrome.

## Verification

Run these before reporting completion:

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

For real paid/private sites, mark browser-login checks as manual or ignored integration tests. Unit-test profile metadata, config resolution, and error handling without launching Chrome where possible.

## Reference

Load `references/auth-profiles-map.md` for the current implementation map and acceptance checklist.
