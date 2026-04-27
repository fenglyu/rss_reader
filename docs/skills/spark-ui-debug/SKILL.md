---
name: spark-ui-debug
description: >
  Investigate production Spark failures and regressions by reasoning tab-by-tab
  over Spark Web UI evidence, executor/driver logs, and SparkConf.
  Trigger when the user provides UI screenshots, copied metrics, event logs,
  stage/task data, or describes a Spark job symptom (slow, failed, OOM, skewed,
  hanging, streaming lag).
---

# Spark UI Debug Skill

## Purpose

Systematically diagnose Spark production incidents using a fixed inspection
order over Spark Web UI evidence. Do not free-associate from generic Spark
knowledge. Base every conclusion on cited evidence.

Supports two evidence modes:
1. **Manual** — user provides screenshots, copied text, or log excerpts
2. **Automated (Playwright)** — user provides a Spark UI URL; use the Playwright
   scraper to collect evidence directly from the live UI

## When to trigger

- Failed Spark application (job or stage failure)
- Performance regression vs baseline
- Task skew or straggler tasks
- Executor loss / OOM kill
- Driver or executor OOM
- SQL / DataFrame bottleneck
- Structured Streaming micro-batch lag or stateful operator blowup
- "Job is stuck" / progress stalled
- Cluster underutilization (cores idle, executors sitting empty)
- **User provides a Spark UI URL** and asks to investigate

## Required inputs

Collect as many of these as possible before starting. If key inputs are missing,
ask for them before diagnosing.

| Input | Priority |
|---|---|
| Spark UI screenshots or copied tab text | High |
| Application ID | High |
| Symptom description (slow / failed / OOM / skewed / stuck / lag) | High |
| Workload type (batch RDD / batch SQL-DataFrame / Structured Streaming / DStream) | High |
| Spark version | High |
| Cluster manager (YARN / K8s / Standalone) | High |
| Driver logs (stderr / stdout) | High for failures |
| Executor logs (stderr) | High for OOM / executor loss |
| SparkConf / `spark-submit` parameters | High for config suspects |
| Event log (if available) | Medium |
| Stage ID / job ID | Medium |
| SQL query ID | Medium for SQL workloads |
| Recent changes (code / config / data volume) | Medium |
| Baseline timing for comparison | Low |

## Playwright automated collection (when URL is provided)

When the user provides a Spark UI URL instead of screenshots:

### Quick collection (all tabs)
```bash
node playwright/spark-ui-scraper.mjs <base-url> --action all
```

### SQL-focused debug flow
```bash
# Step 1: List queries — find the slow/running one
node playwright/spark-ui-scraper.mjs <base-url> --action sql

# Step 2: Drill into specific query for physical plan
node playwright/spark-ui-scraper.mjs <base-url> --action sql-detail --query-id <ID>

# Or auto-detect: find longest query and extract its plan
node playwright/spark-ui-scraper.mjs <base-url> --action sql-debug
```

### Individual tabs
```bash
node playwright/spark-ui-scraper.mjs <base-url> --action jobs
node playwright/spark-ui-scraper.mjs <base-url> --action stages
node playwright/spark-ui-scraper.mjs <base-url> --action executors
node playwright/spark-ui-scraper.mjs <base-url> --action environment
```

After collecting evidence, proceed with the inspection protocol below using
the scraper output as the evidence source.

See `playwright-workflows.md` for detailed step-by-step workflows.

---

## Inspection protocol (always follow this order)

### Step 1 — Classify the symptom

Pick exactly one primary symptom from:

- **FAILED**: job or stage error / exception
- **SLOW**: much slower than baseline or expectation
- **SKEWED**: one or few tasks dominate runtime or data size
- **OOM**: executor or driver killed with memory error
- **STUCK**: no task progress for > N minutes
- **STREAMING_LAG**: micro-batch duration > trigger interval, or growing input queue
- **UNDERUTIL**: executors idle, cores unused despite pending work

### Step 2 — Follow the symptom route

See `symptom-map.md` for the exact tab order per symptom.

Default inspection order when symptom is unclear:
```
Jobs → Stages → Executors → SQL → Environment
```

### Step 3 — Record observations only

For each tab inspected, write down facts:
- metric name, value, unit
- which stage/task/executor
- comparison to median or expected value
- anomaly flag (if a single value is 10x+ the median, flag it)

Do NOT interpret yet. Just collect.

### Step 4 — Infer hypotheses from observations

After collecting observations, propose hypotheses. Each hypothesis must:
- cite at least one specific observation (tab + metric + value)
- have a confidence level: HIGH / MEDIUM / LOW
- have an alternative explanation listed

### Step 5 — Identify missing evidence

Before claiming root cause, list what is not yet available but would
confirm or rule out each hypothesis.

### Step 6 — Recommend next checks and mitigations

Output exactly:
- Next 3 checks (tab, log, command, or query to run)
- Immediate mitigation (safe to do now)
- Longer-term remediation

## Guardrails

- Do not invent metrics not present in the provided evidence.
- Do not assume a config was set unless it appears in the Environment tab
  or in the provided `spark-submit` / `SparkConf` text.
- Do not claim HIGH confidence root cause unless at least 2 independent
  indicators align (e.g., skew visible in Stages + confirmed by SQL plan).
- Do not recommend `repartition` without knowing the workload's shuffle cost.
- Do not diagnose executor OOM without executor log evidence (GC overhead,
  `OutOfMemoryError`, or executor lost message).
- Do not diagnose config misconfiguration without the Environment tab or
  explicit config provided.
- Separate observations from hypotheses. Never mix them.

## Output format

Use this exact structure for every response:

```
## Incident Summary
[1-2 sentences: what failed / what is slow / what symptom]

## Observations
### Jobs Tab
- [fact: metric, value, anomaly flag if any]

### Stages Tab
- [fact]

### Executors Tab
- [fact]

### SQL Tab
- [fact, or "Not provided"]

### Environment Tab
- [fact, or "Not provided"]

### Logs
- [fact from driver/executor logs, or "Not provided"]

## Hypotheses (ranked by confidence)
1. [Hypothesis] — Confidence: HIGH/MEDIUM/LOW
   - Supporting evidence: [tab + metric + value]
   - Alternative explanation: [what else could explain this]

2. [Hypothesis] — Confidence: HIGH/MEDIUM/LOW
   ...

## Missing Evidence
- [What is needed to confirm or rule out hypothesis N]

## Next 3 Checks
1. [Specific tab, log, or command]
2. [Specific tab, log, or command]
3. [Specific tab, log, or command]

## Immediate Mitigation
[Safe action to take now, or "None until root cause confirmed"]

## Longer-Term Remediation
[Fix after root cause confirmed]
```

## Companion files

- `symptom-map.md` — tab inspection routes per symptom
- `evidence-checklist.md` — what to collect before diagnosing
- `output-template.md` — blank output template to copy-paste
- `incident-prompt.md` — copy-paste prompt for ad hoc use
- `playwright-workflows.md` — Playwright CLI workflows for automated evidence collection
- `playwright/spark-ui-scraper.mjs` — Playwright scraper script
- `playwright/README.md` — scraper setup and usage guide
- `examples/skew-case.md` — worked example: task skew
- `examples/oom-case.md` — worked example: executor OOM
- `examples/sql-join-blowup.md` — worked example: SQL join blowup
