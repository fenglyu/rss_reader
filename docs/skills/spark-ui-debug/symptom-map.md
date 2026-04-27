# Symptom Map

Tab inspection routes per symptom. Follow the order listed. Stop when you have
enough evidence to form a HIGH-confidence hypothesis or when you run out of
available evidence.

---

## FAILED — Job or stage error

```
Jobs tab
  └─ Find failed job → note error message and failed stage IDs
Stages tab
  └─ Open failed stage → read failure reason
  └─ Check task list → find tasks with FAILED status → read error detail
Driver logs
  └─ Search for exception stack trace around the job submission time
Executor logs
  └─ Search for OOM / executor lost / exception near the failed task time
Environment tab
  └─ Verify any config that appears in the error message was actually set
```

**Common failure causes by error type**

| Error string             | Likely cause                                                     |
| ------------------------ | ---------------------------------------------------------------- |
| `OutOfMemoryError`       | Executor heap too small; check executor memory and spill metrics |
| `FetchFailed`            | Shuffle data lost (executor died during shuffle read)            |
| `ExecutorLostFailure`    | Executor evicted (YARN preemption, K8s OOM kill, node failure)   |
| `TaskKilled`             | Speculative execution killed a straggler, or job cancelled       |
| `ClassNotFoundException` | Classpath conflict; check Environment → Classpath                |
| `FileNotFoundException`  | Input path moved/deleted; check Input column in Jobs tab         |
| `AnalysisException`      | Schema mismatch or missing table; SQL plan issue                 |

---

## SLOW — Much slower than baseline

```
Jobs tab
  └─ Find the slow job → note total duration vs expectation
  └─ Check event timeline for gaps between stages (scheduling overhead)
Stages tab
  └─ Sort stages by Duration → identify the slowest stage
  └─ For the slowest stage:
       - Check task duration distribution (min / median / max / p75)
       - Check GC time (high GC = memory pressure)
       - Check Shuffle Read Fetch Wait Time (high = network or lost executor)
       - Check Scheduler Delay (high = driver bottleneck or resource contention)
       - Check Shuffle Spill (memory) and Shuffle Spill (disk)
SQL tab (if DataFrame / SQL workload)
  └─ Find the query → open physical plan
  └─ Identify the operator with highest duration or data size
  └─ Check for expensive BroadcastExchange or ShuffleExchange
  └─ Check for Sort + spill size
  └─ Check for HashAggregate + spill size or low avg hash probe bucket iters
  └─ Check scan time and metadata time for FileSourceScan
Executors tab
  └─ Check GC time per executor
  └─ Check task counts per executor (uneven = scheduling issue)
Environment tab
  └─ Validate AQE settings: spark.sql.adaptive.enabled
  └─ Validate executor memory: spark.executor.memory
  └─ Validate parallelism: spark.sql.shuffle.partitions / spark.default.parallelism
```

---

## SKEWED — One or few tasks dominate

```
Stages tab
  └─ Open the slow stage
  └─ Task list: sort by Duration → compare max task vs median task
       - If max > 5x median: skew confirmed
  └─ Check Input Size / Records per task (uneven = data skew)
  └─ Check Shuffle Read Size per task (uneven = shuffle skew)
SQL tab (if DataFrame / SQL workload)
  └─ Find the Exchange operator upstream of the slow stage
  └─ Check number of output rows per partition (if visible)
  └─ Check for Join type: SortMergeJoin with skewed key is classic cause
  └─ Look for missing or stale table statistics (can prevent AQE skew join)
Environment tab
  └─ Validate: spark.sql.adaptive.enabled = true
  └─ Validate: spark.sql.adaptive.skewJoin.enabled = true
  └─ Check: spark.sql.adaptive.skewJoin.skewedPartitionThresholdInBytes
```

**Negative rule**: Do not diagnose skew from slow job runtime alone. Confirm
with task-level duration or data size distribution in the Stages tab.

---

## OOM — Executor or driver out of memory

```
Executor logs (highest priority)
  └─ Search: OutOfMemoryError, GC overhead limit exceeded, executor lost
  └─ Note which executor ID and at what stage/task
Executors tab
  └─ Check Storage Memory column: is memory nearly full?
  └─ Check GC time: high GC = memory pressure before OOM
  └─ Check Disk usage: high spill = already under memory pressure
Stages tab (for the stage running when OOM occurred)
  └─ Check Peak Execution Memory per task
  └─ Check Shuffle Spill (memory) and Shuffle Spill (disk)
  └─ Check task input size: very large tasks may not fit in executor memory
SQL tab (if SQL workload)
  └─ Check Sort → spill size and peak memory
  └─ Check HashAggregate → spill size and peak memory
  └─ Check BroadcastExchange → data size (too large = driver OOM risk)
Storage tab
  └─ Check if RDDs/DataFrames are cached and consuming memory needed for execution
Environment tab
  └─ Validate spark.executor.memory
  └─ Validate spark.memory.fraction and spark.memory.storageFraction
  └─ Validate spark.executor.memoryOverhead (K8s / YARN off-heap)
  └─ Validate spark.sql.autoBroadcastJoinThreshold (if broadcast OOM)
```

**Negative rule**: Do not diagnose executor memory pressure without executor
log evidence or Executors tab GC/spill signals.

---

## STUCK — No task progress

```
Jobs tab
  └─ Check active jobs: is there an active job? Or is the driver idle?
  └─ Check event timeline: when did last activity occur?
Stages tab
  └─ Find active stages → open → check active task count
  └─ If active tasks = 0 but stage not complete: likely waiting for resources
  └─ If active tasks > 0 but no progress: check individual task durations
  └─ Click thread dump on a slow executor to see what it is doing
Executors tab
  └─ Count active executors vs expected
  └─ If executor count is 0 or dropping: resource acquisition issue
  └─ Check for stuck tasks (very high duration, no completion)
Driver logs
  └─ Search: "waiting for", "no resources available", deadlock indicators
  └─ Check if driver is stuck in an action (collect, toPandas on huge data)
```

---

## STREAMING_LAG — Micro-batch duration > trigger interval

```
Structured Streaming tab
  └─ Check Input Rate vs Process Rate:
       - If Input Rate > Process Rate: falling behind
  └─ Check Batch Duration trend: is it growing over time?
  └─ Check addBatch time (read + process + write) — usually the bottleneck
  └─ Check walCommit time (high = metadata log write latency)
  └─ Check queryPlanning time (high = complex plan, recompiled each batch)
  └─ Check Global Watermark Gap: large gap = state cleanup not keeping up
  └─ Check Aggregated State Memory: growing unbounded = state blowup
  └─ Check State Rows Dropped By Watermark: 0 when expected > 0 = watermark stuck
Stages tab (for the micro-batch job)
  └─ Check task skew within the batch
  └─ Check shuffle spill (state store operations can shuffle heavily)
Executors tab
  └─ Check if executors are being added/removed (dynamic allocation instability)
Environment tab
  └─ Validate spark.sql.streaming.statefulOperator.checkCorrectness.enabled
  └─ Check trigger interval setting
  └─ Check checkpoint location (slow object store = high walCommit)
```

---

## UNDERUTIL — Executors idle despite pending work

```
Jobs tab
  └─ Check scheduling mode (FIFO vs Fair)
  └─ Check if jobs are queued or running
Stages tab
  └─ Check active task count vs total executor cores
  └─ High Scheduler Delay = tasks queued, not enough slots, or driver GC
Executors tab
  └─ Count active executors and total cores
  └─ Compare active tasks vs cores: if tasks << cores, parallelism is too low
Environment tab
  └─ Validate spark.default.parallelism
  └─ Validate spark.sql.shuffle.partitions
  └─ Check if dynamic allocation is enabled: spark.dynamicAllocation.enabled
  └─ Check spark.executor.cores
```
