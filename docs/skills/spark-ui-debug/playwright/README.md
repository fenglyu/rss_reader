# Spark UI Playwright Automation

Automated evidence collection from Spark Web UI using Playwright.

## Prerequisites

```bash
npm install playwright
npx playwright install chromium
```

## Quick Start

```bash
# Full evidence collection (all tabs)
node playwright/spark-ui-scraper.mjs https://<spark-ui-url>/proxy/<app-id> --action all

# SQL tab only (list running + completed queries)
node playwright/spark-ui-scraper.mjs https://<spark-ui-url>/proxy/<app-id> --action sql

# Drill into a specific SQL query's physical plan
node playwright/spark-ui-scraper.mjs https://<spark-ui-url>/proxy/<app-id> --action sql-detail --query-id 3

# Auto-find longest running query and extract its plan
node playwright/spark-ui-scraper.mjs https://<spark-ui-url>/proxy/<app-id> --action sql-debug

# Individual tabs
node playwright/spark-ui-scraper.mjs <url> --action jobs
node playwright/spark-ui-scraper.mjs <url> --action stages
node playwright/spark-ui-scraper.mjs <url> --action executors
node playwright/spark-ui-scraper.mjs <url> --action environment
```

## Actions

| Action | Description |
|---|---|
| `all` | Scrape Jobs, Stages, Executors, SQL, Environment tabs (default) |
| `jobs` | Jobs tab — status, duration, stage breakdown |
| `stages` | Stages tab — active, completed, failed with task metrics |
| `executors` | Executors tab — memory, GC, task distribution |
| `sql` | SQL tab — running and completed queries with links |
| `sql-detail` | SQL query detail — physical plan, metrics (requires `--query-id`) |
| `sql-debug` | Auto-flow: find longest query → extract physical plan |
| `environment` | Environment tab — Spark Properties |

## Integration with Claude Code

When using the `spark-ui-debug` skill, you can ask Claude Code to:

```
Use spark-ui-debug with Playwright to investigate
https://badge-megarm.tiktok-row.net/proxy/application_1773129570997_1179136
```

Claude Code will:
1. Run the scraper to collect evidence from all tabs
2. Parse the output following the inspection protocol
3. Produce a structured diagnosis report
