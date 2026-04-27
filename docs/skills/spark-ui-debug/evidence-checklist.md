# Evidence Checklist

Use this checklist before starting a diagnosis. Missing high-priority evidence
should be requested from the user before proposing root causes.

---

## Tier 1 — Always required

- [ ] **Symptom** — described in one sentence (slow / failed / OOM / skewed / stuck / streaming lag / underutil)
- [ ] **Workload type** — batch RDD / batch SQL-DataFrame / Structured Streaming / DStream / JDBC-ODBC
- [ ] **Spark version** — e.g., 3.3.2, 3.5.1
- [ ] **Cluster manager** — YARN / K8s / Standalone
- [ ] **Application ID** — e.g., `application_1700000000000_0042`

## Tier 2 — High value, collect before diagnosing

- [ ] **Jobs tab** — job list with status, duration, stage counts
- [ ] **Stages tab** — stage list; for the problematic stage: task duration summary
      (min / 25th / median / 75th / max), input/output/shuffle sizes, GC time,
      scheduler delay, shuffle spill metrics
- [ ] **Executors tab** — memory usage, GC time, task counts, disk usage per executor
- [ ] **Driver logs** — exception stack traces, warnings around the incident time
- [ ] **Executor logs** — OOM errors, GC overhead messages, executor lost messages

## Tier 3 — Required for SQL/DataFrame workloads

- [ ] **SQL tab** — query duration, physical plan with operator metrics
      (especially spill size, peak memory, data size, number of output rows per operator)
- [ ] **SQL query ID or query text**

## Tier 4 — Required for config-related hypotheses

- [ ] **Environment tab → Spark Properties** — explicitly set properties
- [ ] **`spark-submit` parameters** or programmatic `SparkConf`

## Tier 5 — Required for streaming

- [ ] **Structured Streaming tab** — input rate, process rate, batch duration trend,
      operation duration breakdown (addBatch, walCommit, queryPlanning),
      state row counts, watermark gap
- [ ] **Checkpoint location type** (local / HDFS / S3 / GCS / ADLS)
- [ ] **Trigger interval setting**

## Tier 6 — Context (always helpful)

- [ ] **Recent changes** — code, config, data volume, cluster size
- [ ] **Baseline** — normal runtime or resource usage for comparison
- [ ] **Event log path** (if parseable)
- [ ] **Storage tab** — cached RDDs/DataFrames and their sizes

---

## What to do when evidence is missing

| Missing evidence        | Action                                                                      |
| ----------------------- | --------------------------------------------------------------------------- |
| No Stages tab data      | Ask for stage screenshots or copied text; cannot diagnose task-level issues |
| No executor logs        | Cannot confirm OOM or executor loss; mark hypothesis as MEDIUM max          |
| No Environment tab      | Cannot confirm config hypothesis; note as assumption                        |
| No SQL tab              | Cannot diagnose SQL plan issues; limit to Stages tab signals                |
| No symptom description  | Ask before proceeding                                                       |
| No cluster manager info | Ask; affects executor sizing interpretation                                 |

---

## How to export Spark UI data

**Playwright automation (recommended when URL is available)**:
```bash
# Collect all tabs automatically
node playwright/spark-ui-scraper.mjs <base-url> --action all

# SQL-focused: find slow query and extract physical plan
node playwright/spark-ui-scraper.mjs <base-url> --action sql-debug
```
See `playwright-workflows.md` for detailed workflows.

**Copy tab content**: In most browsers, the Spark UI tables can be selected and
copied as text. For long tables, use the pagination controls to show all rows.

**Event log**: Set `spark.eventLog.enabled=true` and `spark.eventLog.dir` before
running. Parse with `spark-history-server` or tools like `sparklens`, `Dr. Elephant`.

**Executor logs on YARN**:

```bash
yarn logs -applicationId <app_id> -containerId <container_id>
# Or aggregate all:
yarn logs -applicationId <app_id> > app_logs.txt
```

**Executor logs on K8s**:

```bash
kubectl logs <executor-pod-name> -n <namespace>
# For completed pods:
kubectl logs <executor-pod-name> -n <namespace> --previous
```

**Thread dump** (for stuck executors):

- Executors tab → click "Thread Dump" link next to the executor
