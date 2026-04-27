# Output Template

Copy this template for each incident response. Fill in every section.
Use "Not provided" for sections with no evidence. Never leave a section blank.

---

## Incident Summary

[1-2 sentences. State: what application/job, what symptom, what was observed at
the top level. Example: "Spark batch job app_123 failed at Stage 7 with
FetchFailed after running for 42 minutes. One executor was lost during the
shuffle read phase."]

---

## Observations

### Jobs Tab
- Job ID: [value]
- Status: [running / succeeded / failed]
- Duration: [value] (baseline: [value if known])
- Stage breakdown: [N active / N pending / N completed / N failed]
- Input bytes: [value]
- Output bytes: [value]
- Shuffle read: [value]
- Shuffle write: [value]
- Anomalies: [list any values that stand out]

### Stages Tab
- Slow/failed stage ID: [value]
- Stage status: [value]
- Failure reason: [value if failed]
- Task count: [N total / N running / N failed]
- Task duration — min: [v] / median: [v] / p75: [v] / max: [v]
- GC time: [value] ([N]% of task time)
- Scheduler delay: [value]
- Shuffle Read Fetch Wait Time: [value]
- Shuffle Spill (memory): [value]
- Shuffle Spill (disk): [value]
- Peak Execution Memory: [value]
- Input size per task — min: [v] / median: [v] / max: [v]
- Anomalies: [list]

### Executors Tab
- Active executor count: [value]
- Storage memory used / total: [value] / [value]
- GC time total: [value]
- Disk used: [value]
- Task distribution: [even / uneven — describe]
- Dead/removed executors: [N, note reason if shown]
- Anomalies: [list]

### SQL Tab
- Not provided / Query ID: [value]
- Query duration: [value]
- Dominant operator: [name, duration or data size]
- Spill: [operator name, spill size]
- Peak memory: [operator name, value]
- BroadcastExchange data size: [value]
- ShuffleExchange records: [value]
- Anomalies: [list]

### Environment Tab
- Not provided / Key configs explicitly set:
  - spark.executor.memory: [value or "not set"]
  - spark.executor.cores: [value or "not set"]
  - spark.sql.shuffle.partitions: [value or "not set"]
  - spark.sql.adaptive.enabled: [value or "not set"]
  - spark.dynamicAllocation.enabled: [value or "not set"]
  - [other relevant configs]

### Logs
- Not provided / Key log signals:
  - [timestamp] [executor/driver] [message]

---

## Hypotheses (ranked by confidence)

### 1. [Hypothesis title] — Confidence: HIGH / MEDIUM / LOW

- **Supporting evidence**: [tab name → metric → value]
- **Alternative explanation**: [what else could produce the same signal]
- **What would confirm**: [specific evidence needed to upgrade to HIGH]

### 2. [Hypothesis title] — Confidence: HIGH / MEDIUM / LOW

- **Supporting evidence**: [tab → metric → value]
- **Alternative explanation**: [...]
- **What would confirm**: [...]

### 3. [Hypothesis title] — Confidence: HIGH / MEDIUM / LOW

- **Supporting evidence**: [...]
- **Alternative explanation**: [...]
- **What would confirm**: [...]

---

## Missing Evidence

- [ ] [What is missing and why it matters for hypothesis N]
- [ ] [...]

---

## Next 3 Checks

1. **[Tab / log / command]** — [what to look for and why]
2. **[Tab / log / command]** — [what to look for and why]
3. **[Tab / log / command]** — [what to look for and why]

---

## Immediate Mitigation

[Safe action available now, or "None until root cause confirmed."]

Examples:
- Kill and resubmit with increased `spark.executor.memoryOverhead`
- Reduce `spark.sql.autoBroadcastJoinThreshold` to prevent broadcast OOM
- Add salting to the skewed join key

---

## Longer-Term Remediation

[Fix after root cause confirmed. Include config change or code change.]

Examples:
- Enable AQE skew join: `spark.sql.adaptive.skewJoin.enabled=true`
- Increase executor memory: `spark.executor.memory=8g`
- Replace SortMergeJoin with salted join or rangePartition
- Upgrade Spark version to get improved AQE
