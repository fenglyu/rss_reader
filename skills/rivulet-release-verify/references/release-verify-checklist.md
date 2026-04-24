# Rivulet Release Verify Checklist

## Standard Verification

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

## Skill Verification

For each local skill folder:

```bash
python3 /Users/bytedance/.codex/skills/.system/skill-creator/scripts/quick_validate.py skills/<skill-name>
```

## Commit Template

```text
<intent line: why this change exists>

<context, constraints, and approach rationale>

Confidence: high
Scope-risk: narrow
Tested: cargo fmt -- --check; cargo clippy -- -D warnings; cargo test
Not-tested: <known gap, or omit if none>
```

## Exclude From Commits

- `.omx/` runtime logs, plans, and state unless explicitly requested.
- `target/` build output.
- Local SQLite databases.
- Browser profile directories and cookie/session data.
- Editor or OS metadata.
