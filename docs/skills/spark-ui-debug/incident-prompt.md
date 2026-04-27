# Incident Prompt Template

Copy this template when starting a new Spark debugging session.
Fill in the context block at the bottom, then paste into Claude Code or Codex.

---

```
Use the spark-ui-debug skill.

## Goal
Determine the most likely cause of this Spark production incident and
propose the next 3 checks.

## Rules
1. Base every conclusion only on evidence I provide. Do not invent metrics.
2. Follow the inspection order: Jobs → Stages → Executors → SQL → Environment.
   Skip tabs for which no evidence is provided; note them as missing.
3. Separate observations from hypotheses. Never mix them.
4. Rank hypotheses by confidence: HIGH / MEDIUM / LOW.
   Do not claim HIGH confidence without at least 2 independent supporting signals.
5. If evidence is insufficient to diagnose, state exactly what is missing
   and what it would tell us.
6. Do not assume any config was set unless it appears in the Environment tab
   or in the provided SparkConf / spark-submit params below.
7. Do not recommend repartition, broadcast, or other transformations without
   citing which specific metric makes them appropriate.

## Output format (required)
- Incident summary (1-2 sentences)
- Observations (by tab, facts only)
- Hypotheses ranked by confidence (each with: supporting evidence, alternative, what would confirm)
- Missing evidence
- Next 3 checks
- Immediate mitigation
- Longer-term remediation

## Incident context

**Spark version**: [e.g. 3.5.1]
**Cluster manager**: [YARN / K8s / Standalone]
**Workload type**: [batch-RDD / batch-SQL-DataFrame / Structured Streaming / DStream]
**Application ID**: [e.g. application_1700000000000_0042]
**Symptom**: [slow / failed / OOM / skewed / stuck / streaming-lag / underutil]
**Symptom description**: [1-2 sentences describing what is wrong]
**Recent changes**: [code / config / data volume / cluster size — or "none"]
**Baseline**: [normal runtime or resource usage — or "unknown"]

## Evidence

### Jobs tab
[Paste copied text or describe what you see. Include: job status, duration,
stage counts, input/output/shuffle bytes.]

### Stages tab
[Paste stage list and the detail page for the slow/failed stage.
Include: task duration summary (min/median/max), GC time, scheduler delay,
shuffle read fetch wait, shuffle spill memory, shuffle spill disk,
peak execution memory, input size per task distribution.]

### Executors tab
[Paste executor table. Include: memory used/total, GC time, task counts,
disk used, any dead executors.]

### SQL tab
[Paste or describe the physical plan with metrics. Include: dominant operator,
spill sizes, peak memory, exchange data sizes.]
[Or: "Not available for this workload type"]

### Environment tab — Spark Properties
[Paste explicitly set properties, or "Not available"]

### Driver logs
[Paste relevant excerpts, especially exceptions and stack traces]
[Or: "Not available"]

### Executor logs
[Paste relevant excerpts, especially OOM errors, GC messages, executor lost]
[Or: "Not available"]
```

---

## Notes on filling this in

**For Stages tab**: The most valuable data is the task duration distribution.
In the Spark UI, open the stage detail page and look for the "Summary Metrics"
table which shows percentiles. Copy that entire table.

**For SQL tab**: Open the query detail page and expand the physical plan.
The key numbers to copy are spill size, peak memory, and data size for each
Exchange operator.

**For Executor logs on YARN**:
```bash
yarn logs -applicationId <app_id> 2>&1 | grep -E "ERROR|WARN|OutOfMemory|GC overhead" | head -100
```

**For Executor logs on K8s**:
```bash
kubectl logs <executor-pod> -n <namespace> | grep -E "ERROR|WARN|OutOfMemory|GC overhead" | head -100
```

**For Driver logs on YARN**:
```bash
yarn logs -applicationId <app_id> -log_files stdout 2>&1 | tail -200
```

---

## Minimal version (for quick triage)

If you only have a symptom and partial data, use this shorter version:

```
Use the spark-ui-debug skill to triage this incident.

Symptom: [describe]
Spark version: [version]
Cluster manager: [YARN / K8s / Standalone]
Workload: [batch / SQL / streaming]

Evidence:
[paste whatever you have]

Based on available evidence:
1. What are the top 2 hypotheses?
2. What are the 3 most important pieces of evidence I should collect next?
3. Is there anything that should be mitigated immediately?
```
