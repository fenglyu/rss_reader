# Worked Example: Task Skew

## Incident context

- Spark version: 3.4.1
- Cluster manager: YARN
- Workload type: batch SQL-DataFrame
- Application ID: application_1700000000000_0099
- Symptom: SLOW — job takes 3h, baseline is 25 minutes
- Recent changes: input data volume 10x'd due to new customer segment
- Baseline: 25 minutes

---

## Evidence provided

### Jobs tab
- 1 job, status: running (2h 47m elapsed)
- Stage breakdown: 12 completed, 1 active (Stage 14)
- Stage 14: 200 tasks, 1 active, 199 succeeded

### Stages tab — Stage 14
- Task duration summary:
  - min: 4s
  - 25th: 18s
  - median: 22s
  - 75th: 31s
  - max: 2h 44m  ← ANOMALY
- GC time: 3s median, 8s max (normal)
- Scheduler delay: 0.3s (normal)
- Shuffle Read Fetch Wait Time: 1.2s median, 1.8s max (normal)
- Shuffle Spill (memory): 0
- Shuffle Spill (disk): 0
- Input size per task: min 1MB, median 220MB, max 48GB  ← ANOMALY

### Executors tab
- 20 executors, all active
- Memory: 6.1GB / 8GB per executor (normal)
- GC time: 12s total across all executors (normal)
- Task distribution: 19 executors have 0 active tasks; 1 executor has 1 task

### SQL tab
- Query: large customer event aggregation with GROUP BY customer_id
- Physical plan highlights:
  - HashAggregate (final) — peak memory: 420MB, no spill
  - ShuffleExchange (HashPartitioning by customer_id, 200 partitions)
    - shuffle records written: 4.2B
  - HashAggregate (partial)
  - FileSourceScan — scan time: 8m 12s, records: 4.2B

### Environment tab
- spark.sql.adaptive.enabled = false  ← NOTE
- spark.sql.shuffle.partitions = 200
- spark.executor.memory = 8g
- spark.executor.cores = 4

### Driver logs
- No errors

### Executor logs
- No errors on the stuck executor

---

## Incident Summary

Spark SQL aggregation job (Stage 14) has been running for 2h 47m against a
25-minute baseline. One task in Stage 14 has been running for 2h 44m while
199 other tasks completed in a median of 22 seconds. The stuck task has 48GB
of input data vs a median of 220MB, indicating extreme data skew on the
shuffle partition for one customer_id value.

---

## Observations

### Jobs Tab
- 1 active job, 12 stages completed, 1 stage active (Stage 14)
- Stage 14 has been active for the vast majority of job runtime

### Stages Tab
- Task max duration (2h 44m) is 447x the median (22s) — severe skew confirmed
- Max input size per task (48GB) is 218x the median (220MB) — data skew confirmed
- One task is processing the vast majority of the input data for this stage
- GC time and shuffle spill are normal — memory pressure is not the issue

### Executors Tab
- 19 of 20 executors idle, waiting on 1 task
- No memory pressure signals

### SQL Tab
- Shuffle partitioned by `customer_id` with 200 partitions
- One partition has 48GB / ~4B records (disproportionate)
- AQE is disabled — skew join optimization not available

### Environment Tab
- `spark.sql.adaptive.enabled = false` — AQE skew handling is off
- `spark.sql.shuffle.partitions = 200` — fixed, not adaptive

---

## Hypotheses (ranked by confidence)

### 1. Data skew on customer_id key — Confidence: HIGH
- Supporting evidence:
  - Stages tab: max task input 48GB vs median 220MB (218x ratio)
  - Stages tab: max task duration 2h 44m vs median 22s (447x ratio)
  - SQL tab: shuffle partitioned by customer_id
  - One customer_id value has an extreme number of records (e.g., test account, internal account, null key)
- Alternative explanation: Remote read slowdown on one specific executor partition
  (ruled out: Shuffle Read Fetch Wait Time is normal at 1.8s max)
- What would confirm: Query `SELECT customer_id, COUNT(*) FROM source GROUP BY customer_id ORDER BY 2 DESC LIMIT 10` to find the hot key

### 2. AQE disabled, preventing automatic skew handling — Confidence: HIGH
- Supporting evidence:
  - Environment tab: spark.sql.adaptive.enabled = false
  - With AQE enabled, Spark 3.x would detect and split the skewed partition automatically
- Alternative explanation: AQE may not help if the skewed partition is already at minimum split size
- What would confirm: Enable AQE and re-run to observe whether Stage 14 splits the partition

### 3. 10x data growth exposed a pre-existing hot key — Confidence: MEDIUM
- Supporting evidence:
  - Recent changes: input data 10x'd
  - If one customer had proportionally more data before, they now dominate
- Alternative explanation: New customer with bulk historical backfill
- What would confirm: Compare customer_id distribution before and after data volume increase

---

## Missing Evidence

- [ ] Distribution of customer_id values in source data (needed to identify the hot key)
- [ ] Whether this customer_id is a known internal/test account with anomalous volume

---

## Next 3 Checks

1. **Run key distribution query** — `SELECT customer_id, COUNT(*) FROM events GROUP BY customer_id ORDER BY 2 DESC LIMIT 20` — identify the hot key and its record count
2. **SQL tab → ShuffleExchange** — check records per partition if visible, confirm which partition is hot
3. **Environment tab** — verify no other AQE-related configs are set that might interfere with enabling it

---

## Immediate Mitigation

Kill the stuck job and resubmit with:
```
spark.sql.adaptive.enabled=true
spark.sql.adaptive.skewJoin.enabled=true
spark.sql.adaptive.skewJoin.skewedPartitionThresholdInBytes=256MB
```

If the hot key is a known null or test value, add a filter or pre-aggregate step.

---

## Longer-Term Remediation

1. Enable AQE globally in cluster defaults (`spark.sql.adaptive.enabled=true`).
2. Investigate whether the hot key (null, internal account, specific customer) can
   be pre-filtered or handled with a two-phase aggregation.
3. Add a data quality check that alerts when a single key exceeds X% of total records.
4. Consider salting the join key if the hot key is a legitimate production customer
   that will continue to grow.
