# Worked Example: Executor OOM

## Incident context

- Spark version: 3.3.2
- Cluster manager: Kubernetes
- Workload type: batch SQL-DataFrame
- Application ID: application_spark-etl-prod-20240315-001
- Symptom: FAILED — job failed after ~18 minutes
- Recent changes: upgraded from Spark 3.1 to 3.3.2, same config
- Baseline: 12 minutes runtime, no failures

---

## Evidence provided

### Jobs tab
- 1 job, status: FAILED
- 3 stages completed, Stage 4 failed
- Stage 4: 50 tasks, 47 succeeded, 3 failed

### Stages tab — Stage 4
- Task duration: min 2m / median 9m / max 18m (3 tasks failed)
- GC time: min 8s / median 45s / max 4m 12s  ← HIGH GC
- Peak Execution Memory: min 2.1GB / median 4.8GB / max 6.9GB
- Shuffle Spill (memory): 12GB total across failed tasks  ← SPILL
- Shuffle Spill (disk): 4.3GB total across failed tasks
- Failure reason: ExecutorLostFailure (Executor 3 exited caused by ... OOMKilled)

### Executors tab
- Executor 3: DEAD — removed
- Last GC time before removal: 4m 12s (62% of task time)  ← ANOMALY
- Storage memory: 5.8GB / 6GB at time of failure
- Disk: 4.3GB (shuffle spill)
- Other executors: 4-7 active, GC 8-22% of task time (elevated but alive)

### SQL tab
- Physical plan:
  - SortMergeJoin (customer_events INNER JOIN customer_profile ON customer_id)
    - Sort → spill size: 3.1GB (on executor 3)
    - Sort → peak memory: 5.4GB (on executor 3)
  - ShuffleExchange (HashPartitioning, 200 partitions)
    - data size: 28GB total
    - remote bytes read: 18GB by executor 3
  - FileSourceScan: customer_events (Parquet, 22GB)
  - FileSourceScan: customer_profile (Parquet, 180MB)

### Environment tab
- spark.executor.memory = 6g
- spark.executor.memoryOverhead = 512m  ← NOTE: default, not explicitly tuned
- spark.memory.fraction = 0.6
- spark.executor.cores = 4
- spark.sql.shuffle.partitions = 200
- spark.sql.adaptive.enabled = true
- spark.sql.autoBroadcastJoinThreshold = 10MB

### Executor logs (executor 3)
```
2024-03-15 18:42:11 WARN TaskMemoryManager: Failed to allocate 524288000 bytes for Sort
2024-03-15 18:42:11 INFO MemoryStore: Not enough space to cache rdd_12_3 in memory
2024-03-15 18:43:04 WARN GarbageCollectionMetrics: GC pause 12345ms
2024-03-15 18:44:18 ERROR SparkUncaughtExceptionHandler: Uncaught exception in thread
  java.lang.OutOfMemoryError: GC overhead limit exceeded
    at org.apache.spark.sql.execution.UnsafeExternalRowSorter.insertRow(...)
```

---

## Incident Summary

Spark SQL job failed at Stage 4 when Executor 3 was OOM-killed by Kubernetes
after spending 62% of its time in GC. The executor was running Sort operations
for a SortMergeJoin with 3.1GB of sort spill and 5.4GB peak memory, exceeding
the 6GB executor heap. The upgrade from Spark 3.1 to 3.3.2 likely changed
memory accounting or shuffle behavior.

---

## Observations

### Jobs Tab
- Job failed at Stage 4
- 3 of 50 Stage 4 tasks failed

### Stages Tab
- GC time max 4m 12s = 62% of task time — severe memory pressure before OOM
- Shuffle spill (memory) 12GB and disk 4.3GB — executor could not hold sort buffers
- Peak Execution Memory 6.9GB exceeded executor heap (6GB) on at least one task

### Executors Tab
- Executor 3 killed by K8s OOMKilled
- GC time 62% before death — JVM was thrashing before container kill
- Storage memory was nearly full (5.8GB / 6GB)

### SQL Tab
- SortMergeJoin chosen — customer_profile (180MB) is above 10MB broadcast threshold
- Sort on executor 3: 3.1GB spill, 5.4GB peak memory — cannot fit in 6GB heap
- ShuffleExchange: executor 3 read 18GB remotely (hot partition assignment)

### Environment Tab
- spark.executor.memory = 6g (unchanged from Spark 3.1 config)
- spark.executor.memoryOverhead = 512m (default; K8s container limit = 6.5GB)
- spark.sql.autoBroadcastJoinThreshold = 10MB (customer_profile at 180MB exceeds this)
- AQE enabled but did not convert to broadcast (too large)

### Logs
- Explicit OOM: `OutOfMemoryError: GC overhead limit exceeded` in Sort
- Sort triggered at the SortMergeJoin operator

---

## Hypotheses (ranked by confidence)

### 1. Executor heap too small for Sort in SortMergeJoin — Confidence: HIGH
- Supporting evidence:
  - Executor logs: OutOfMemoryError in UnsafeExternalRowSorter
  - Stages tab: Sort peak memory 5.4GB in a 6GB heap executor
  - Stages tab: GC time 62% — JVM was in memory crisis before kill
  - SQL tab: SortMergeJoin Sort had 3.1GB spill
- Alternative explanation: Memory leak in Spark 3.3.2 (possible but less likely than sizing)
- What would confirm: Increase executor memory or memoryOverhead and observe whether OOM is resolved

### 2. Spark 3.1→3.3.2 upgrade changed memory accounting — Confidence: MEDIUM
- Supporting evidence:
  - Recent changes: Spark version upgrade with no config change
  - Spark 3.2+ introduced changes to memory accounting for Sort and AQE
  - Same config that worked on 3.1 fails on 3.3.2
- Alternative explanation: Coincidental data growth unrelated to upgrade
- What would confirm: Run the same job on Spark 3.1 with identical data to compare peak memory

### 3. autoBroadcastJoinThreshold too low, forcing expensive SortMergeJoin — Confidence: MEDIUM
- Supporting evidence:
  - SQL tab: customer_profile is 180MB — 18x the 10MB threshold
  - Broadcasting 180MB would eliminate the Sort and shuffle on the build side
  - SortMergeJoin requires sorting both sides, consuming more memory
- Alternative explanation: Broadcast could cause driver OOM if driver memory is also limited
- What would confirm: Check spark.driver.memory; if > 1GB, increasing threshold to 256MB is safe

---

## Missing Evidence

- [ ] spark.driver.memory value (to assess broadcast safety)
- [ ] K8s pod resource limits (to confirm container memory ceiling)
- [ ] Whether data volume changed between Spark versions (rule out coincidence)

---

## Next 3 Checks

1. **Environment tab** — find `spark.driver.memory` and K8s executor resource limits to determine safe broadcast size ceiling
2. **SQL tab → SortMergeJoin node** — check exact data sizes of both join sides across all partitions to estimate broadcast feasibility
3. **Executor 3 logs (full)** — search for earlier GC warnings to establish when memory pressure started (before or after the sort step)

---

## Immediate Mitigation

Option A (fastest): Increase executor memory overhead for K8s:
```
spark.executor.memoryOverhead=2g
```
This raises the K8s container ceiling from 6.5GB to 8GB.

Option B (lower memory): Raise broadcast threshold to eliminate SortMergeJoin:
```
spark.sql.autoBroadcastJoinThreshold=256MB
```
Only safe if driver memory >= 2GB.

---

## Longer-Term Remediation

1. Set `spark.executor.memoryOverhead` explicitly in cluster defaults for K8s
   (do not rely on the 384m/10% default — it is insufficient for sort-heavy workloads).
2. Establish a post-upgrade validation that compares peak memory metrics on
   the same representative job before and after a Spark version bump.
3. Review `spark.sql.autoBroadcastJoinThreshold` policy — for tables < 512MB,
   broadcasting is usually safe and avoids SortMergeJoin memory pressure.
4. Add a monitoring alert for executor GC time > 30% to catch memory pressure
   before OOM kills occur.
