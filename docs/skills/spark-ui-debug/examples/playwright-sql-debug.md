# Worked Example: Playwright SQL Debug Flow

Demonstrates the automated workflow for investigating a slow SQL query using
Playwright CLI against a live Spark UI.

---

## Scenario

User provides:
```
Investigate https://badge-megarm.tiktok-row.net/proxy/application_1773129570997_1179136/SQL/
— there are long-running queries.
```

---

## Step 1 — Navigate to SQL tab, list queries

```bash
node playwright/spark-ui-scraper.mjs \
  https://badge-megarm.tiktok-row.net/proxy/application_1773129570997_1179136 \
  --action sql
```

Output:
```
=== SQL Tab ===

--- Running Queries ---
  ID | Description                                      | Duration | Running Jobs
  3  | INSERT INTO warehouse.fact_events SELECT ...      | 2.3 h    | 1
  7  | SELECT count(*) FROM staging.raw_clicks WHERE ... | 45 min   | 1

Query links found:
  - 3: .../SQL/execution/?id=3
    Description: INSERT INTO warehouse.fact_events SELECT ...
  - 7: .../SQL/execution/?id=7
    Description: SELECT count(*) FROM staging.raw_clicks WHERE ...

--- Completed Queries ---
  ID | Description                         | Duration | Jobs
  1  | CREATE TABLE IF NOT EXISTS ...       | 12 s     | 1
  2  | ANALYZE TABLE staging.raw_clicks ... | 3 min    | 2
```

**Observation**: Query ID 3 has been running for 2.3 hours — this is the primary suspect.

---

## Step 2 — Drill into the slow query

```bash
node playwright/spark-ui-scraper.mjs \
  https://badge-megarm.tiktok-row.net/proxy/application_1773129570997_1179136 \
  --action sql-detail --query-id 3
```

The scraper:
1. Navigates to `/SQL/execution/?id=3`
2. Clicks the "Details" button to expand the execution plan
3. Extracts the `== Physical Plan ==`

Output:
```
=== SQL Query Detail (id=3) ===
  (Details section expanded)

--- Physical Plan ---
== Physical Plan ==
InsertIntoHadoopFsRelation (12)
+- *(6) Sort [event_date ASC, user_id ASC], false, 0
   +- Exchange rangepartitioning(event_date ASC, user_id ASC, 2000) (11)
      +- *(5) SortMergeJoin [user_id], [user_id], Inner (10)
         :- *(3) Sort [user_id ASC], false, 0
         :  +- Exchange hashpartitioning(user_id, 2000) (9)
         :     +- *(2) Filter isnotnull(user_id)
         :        +- *(2) FileSourceScan parquet staging.raw_clicks (1)
         :           - number of output rows: 4,820,000,000
         :           - metadata time: 2 min
         :           - data size: 892 GB
         +- *(4) Sort [user_id ASC], false, 0
            +- Exchange hashpartitioning(user_id, 2000) (8)
               +- *(1) Filter isnotnull(user_id)
                  +- *(1) FileSourceScan parquet warehouse.dim_users (2)
                     - number of output rows: 180,000,000
                     - data size: 24 GB

--- Query Metrics ---
  Operator         | Spill (Memory) | Spill (Disk) | Peak Memory | Output Rows
  SortMergeJoin    | 340 GB         | 128 GB       | 4.2 GB      | 12,800,000,000
  Exchange (hash)  | —              | —            | —           | 4,820,000,000
  Sort (pre-join)  | 180 GB         | 67 GB        | 2.1 GB      | 4,820,000,000
```

---

## Step 3 — Classify and diagnose

From the Playwright output, apply the inspection protocol:

**Symptom**: SLOW — query running 2.3 hours

**Key observations from SQL tab**:
- SortMergeJoin producing 12.8B output rows from 4.8B × 180M inputs = fan-out
- Massive spill: 340 GB memory spill, 128 GB disk spill on the join
- Exchange uses 2000 partitions for 892 GB of data = ~446 MB/partition (reasonable)
- The join fan-out (12.8B output from 4.8B input) suggests many-to-many on user_id

**Hypothesis**: user_id is not unique in dim_users — this is a many-to-many join
causing output row explosion.

---

## Step 4 — Collect supporting evidence

```bash
# Get executor health to check memory pressure
node playwright/spark-ui-scraper.mjs \
  https://badge-megarm.tiktok-row.net/proxy/application_1773129570997_1179136 \
  --action executors

# Get environment to check AQE and memory settings
node playwright/spark-ui-scraper.mjs \
  https://badge-megarm.tiktok-row.net/proxy/application_1773129570997_1179136 \
  --action environment
```

---

## Alternative: One-shot auto-debug

```bash
node playwright/spark-ui-scraper.mjs \
  https://badge-megarm.tiktok-row.net/proxy/application_1773129570997_1179136 \
  --action sql-debug
```

This automatically finds query ID 3 (longest running) and extracts its plan.
