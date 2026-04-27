# Playwright CLI Workflows for Spark UI

Step-by-step Playwright CLI commands for automated Spark UI evidence collection.
Use these when the user provides a Spark UI URL instead of screenshots or copied text.

---

## Workflow 1 — Full Evidence Collection

When the user provides an application URL, collect all tabs in one pass:

```bash
node playwright/spark-ui-scraper.mjs <base-url> --action all
```

This follows the default inspection order: Jobs → Stages → Executors → SQL → Environment.

---

## Workflow 2 — SQL Query Debugging (most common)

When the user wants to investigate a slow or long-running SQL query:

### Step 1: List SQL queries

```bash
node playwright/spark-ui-scraper.mjs <base-url> --action sql
```

Output includes:

- Running Queries table with duration and description
- Completed Queries table
- Direct links to each query's detail page

### Step 2: Identify the problematic query

From the running queries output, find the query with the longest duration or the
one matching the user's description.

### Step 3: Get the physical plan

```bash
node playwright/spark-ui-scraper.mjs <base-url> --action sql-detail --query-id <ID>
```

This navigates to `/SQL/execution/?id=<ID>`, clicks the "Details" button to unfold
the execution plan, and extracts:

- `== Physical Plan ==` text
- Query metrics tables (spill, peak memory, data sizes)
- Operator-level statistics

### Step 4: (Optional) Auto-detect slowest query

```bash
node playwright/spark-ui-scraper.mjs <base-url> --action sql-debug
```

Automatically finds the longest-running query and drills into its detail page.

---

## Workflow 3 — Stage-Level Investigation

When the user reports task skew or a failed stage:

```bash
# Get stage overview
node playwright/spark-ui-scraper.mjs <base-url> --action stages
```

For stage detail pages (task metrics, summary percentiles), navigate directly:

```bash
# Use Playwright CLI interactively
npx playwright open "<base-url>/stages/stage/?id=<STAGE_ID>&attempt=0"
```

---

## Workflow 4 — Executor Health Check

When the user reports OOM, executor loss, or GC pressure:

```bash
node playwright/spark-ui-scraper.mjs <base-url> --action executors
```

Look for:

- Dead/removed executors
- High GC time (> 10% of task time)
- Memory usage near limits
- Uneven task distribution

---

## Workflow 5 — Configuration Audit

When the user suspects misconfiguration:

```bash
node playwright/spark-ui-scraper.mjs <base-url> --action environment
```

Key properties to verify:

- `spark.executor.memory` / `spark.driver.memory`
- `spark.sql.shuffle.partitions`
- `spark.sql.adaptive.enabled`
- `spark.sql.adaptive.skewJoin.enabled`
- `spark.dynamicAllocation.enabled`

---

## Using Playwright CLI Directly (Interactive)

For cases where the scraper doesn't capture what you need, use Playwright CLI
interactively:

```bash
# Open browser to a specific page
npx playwright open "<base-url>/SQL/"

# Take a screenshot for manual inspection
npx playwright screenshot --url "<base-url>/SQL/execution/?id=3" screenshot.png

# Generate code for a custom navigation flow
npx playwright codegen "<base-url>/SQL/"
```

---

## Error Handling

| Error                         | Cause                              | Fix                                            |
| ----------------------------- | ---------------------------------- | ---------------------------------------------- |
| `net::ERR_CONNECTION_REFUSED` | Spark UI not accessible            | Check VPN / proxy / URL                        |
| `Timeout exceeded`            | Page took too long to load         | Increase `TIMEOUT` in scraper or check network |
| `Physical Plan not found`     | Plan hidden behind JS toggle       | Use `npx playwright open` to inspect manually  |
| `SSL error`                   | Self-signed cert on internal proxy | Already handled: `ignoreHTTPSErrors: true`     |

---

## Combining with Diagnosis

After collecting evidence via Playwright, feed the output directly into the
spark-ui-debug inspection protocol:

1. Run the appropriate scraper action(s)
2. Parse stdout as the evidence for each tab section
3. Follow the symptom map (`symptom-map.md`) using the collected data
4. Produce the output in the standard template (`output-template.md`)
