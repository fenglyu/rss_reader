# Worked Example: SQL Join Blowup (BroadcastExchange OOM + Plan Regression)

## Incident context

- Spark version: 3.5.1
- Cluster manager: YARN
- Workload type: batch SQL-DataFrame
- Application ID: application_1700000000000_0201
- Symptom: FAILED — driver OOM after 4 minutes
- Recent changes: added a new dimension table join; table stats not yet collected
- Baseline: similar jobs run in 8-10 minutes without failure

---

## Evidence provided

### Jobs tab
- 1 job, status: FAILED at Stage 2
- Stage 0 and Stage 1 completed (scan stages)
- Stage 2 failed immediately: 0 tasks completed

### Stages tab — Stage 2
- Stage 2 failure reason: "Job aborted due to stage failure: Task serialization failed"
- No task metrics (stage never produced running tasks)

### Executors tab
- Driver: FAILED
- 8 executor containers: still running (not involved in failure)

### SQL tab
- Physical plan:
  - BroadcastHashJoin (fact_orders JOIN dim_product ON product_id)
    - BroadcastExchange — data size: 18.4GB  ← ANOMALY
      - FileSourceScan: dim_product (Parquet, partitioned table, 3500 partitions)
        - metadata time: 4m 12s  ← ANOMALY
        - number of files: 14,200
    - ShuffleExchange
      - FileSourceScan: fact_orders

### Environment tab
- spark.driver.memory = 4g
- spark.sql.autoBroadcastJoinThreshold = 10MB
- spark.sql.adaptive.enabled = true
- spark.sql.adaptive.localShuffleReader.enabled = true
- spark.sql.statistics.fallBackToHdfs = false  ← NOTE
- Table statistics for dim_product: NOT PRESENT in plan (no ANALYZE TABLE run)

### Driver logs
```
2024-11-20 09:04:33 ERROR SparkContext: Error initializing SparkContext
java.lang.OutOfMemoryError: Java heap space
  at org.apache.spark.sql.execution.joins.BroadcastHashJoinExec$$anonfun$...
  at org.apache.spark.broadcast.TorrentBroadcast.blockifyObject(...)
```

---

## Incident Summary

Spark SQL job failed with driver OOM at Stage 2 when attempting to broadcast
`dim_product` as the build side of a BroadcastHashJoin. The table had no
collected statistics, and with `spark.sql.statistics.fallBackToHdfs=false`,
the planner could not estimate the table size. AQE chose to broadcast based on
incomplete information, resulting in an 18.4GB BroadcastExchange that exceeded
the 4GB driver heap.

---

## Observations

### Jobs Tab
- Job failed before any tasks ran in Stage 2
- Stages 0 and 1 (scans) completed normally

### Stages Tab
- Stage 2 produced zero tasks — failure occurred during broadcast materialization
  on the driver, before task dispatch

### Executors Tab
- Driver killed with OOM
- Executors unaffected (never received tasks for Stage 2)

### SQL Tab
- BroadcastExchange data size: 18.4GB — 1840x the 10MB threshold
- dim_product scan: metadata time 4m 12s across 14,200 files in 3,500 partitions
- No table statistics in plan (ANALYZE TABLE not run on dim_product)

### Environment Tab
- spark.sql.autoBroadcastJoinThreshold = 10MB (Spark default)
- spark.sql.statistics.fallBackToHdfs = false — planner cannot fall back to HDFS
  size to estimate table size when catalog stats are absent
- spark.driver.memory = 4g — insufficient to broadcast 18.4GB

### Logs
- Driver OOM in BroadcastHashJoin serialization — confirms broadcast reached driver

---

## Hypotheses (ranked by confidence)

### 1. Missing table statistics caused the planner to underestimate dim_product size, triggering an unsafe broadcast — Confidence: HIGH
- Supporting evidence:
  - SQL tab: No statistics present in plan for dim_product
  - SQL tab: BroadcastExchange data size 18.4GB (1840x threshold)
  - Environment tab: spark.sql.statistics.fallBackToHdfs = false
  - Without stats and without HDFS fallback, the planner has no size signal and
    may default to a broadcast decision based on heuristics
- Alternative explanation: Planner bug in Spark 3.5.1 (less likely; stats absence is sufficient cause)
- What would confirm: Run `ANALYZE TABLE dim_product COMPUTE STATISTICS` and re-check the plan; AQE should switch to SortMergeJoin

### 2. dim_product partition explosion (3,500 partitions, 14,200 files) inflated the broadcast size — Confidence: HIGH
- Supporting evidence:
  - SQL tab: 14,200 files in 3,500 partitions — high file count per partition
  - SQL tab: metadata time 4m 12s — extreme metadata overhead
  - High file count often indicates small files accumulation, which inflates per-file metadata but also per-file data
- Alternative explanation: dim_product is legitimately large (new data added)
- What would confirm: `SELECT COUNT(*) FROM dim_product` and `SHOW PARTITIONS dim_product` to assess true cardinality

### 3. autoBroadcastJoinThreshold not updated after table growth — Confidence: MEDIUM
- Supporting evidence:
  - 10MB threshold is the Spark default — not intentionally tuned for this workload
  - If stats had been present, AQE would have used them to avoid the broadcast
  - Threshold was designed as a safety cap, but without stats it provides no protection
- Alternative explanation: Threshold is irrelevant here — the issue is the absent stats, not the threshold value
- What would confirm: Collect stats and confirm the plan changes without touching threshold

---

## Missing Evidence

- [ ] True row count and byte size of dim_product (to understand whether it has genuinely grown or stats are simply missing)
- [ ] When dim_product was last ANALYZE'd (or if ever)
- [ ] Whether dim_product is a Hive-managed or external table (affects stats collection approach)

---

## Next 3 Checks

1. **Hive metastore / catalog** — run `DESCRIBE EXTENDED dim_product` or `SHOW TABLE EXTENDED LIKE 'dim_product'` to check last stats update timestamp and total size
2. **SQL tab → FileSourceScan → dim_product** — check `number of output rows` estimate in the plan (a very low or 0 estimate confirms missing stats drove the bad broadcast decision)
3. **HDFS / object store** — check actual size of dim_product data directory to estimate what stats would show

---

## Immediate Mitigation

Option A (safest): Disable broadcast for this query:
```sql
SELECT /*+ MERGEJOIN(dim_product) */ * FROM fact_orders JOIN dim_product ...
```
Or set for the session:
```
spark.sql.autoBroadcastJoinThreshold=-1
```

Option B: Collect statistics before re-running:
```sql
ANALYZE TABLE dim_product COMPUTE STATISTICS;
ANALYZE TABLE dim_product COMPUTE STATISTICS FOR ALL COLUMNS;
```
Then re-run — AQE should now choose SortMergeJoin or a safe BroadcastHashJoin
based on actual size.

---

## Longer-Term Remediation

1. Add `ANALYZE TABLE` to the dim_product refresh pipeline (run after each load).
2. Enable HDFS stats fallback as a safety net:
   `spark.sql.statistics.fallBackToHdfs=true`
   This lets the planner use file system size when catalog stats are absent.
3. Compact dim_product partitions to reduce the 14,200-file footprint —
   small file accumulation increases metadata time and broadcast serialization overhead.
4. Set a conservative broadcast threshold for the production cluster:
   `spark.sql.autoBroadcastJoinThreshold=50MB`
   This reduces the blast radius of missing-stats broadcast decisions.
5. Add a pre-flight check in the job that aborts if stats are absent on dimension tables.
