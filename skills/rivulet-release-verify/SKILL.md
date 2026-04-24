---
name: rivulet-release-verify
description: Use this skill before finalizing Rivulet changes, preparing commits, pushing branches, or reporting verification evidence for this repository.
---

# Rivulet Release Verify

## Overview

Use this workflow to close out Rivulet changes with the right checks, clean git scope, Lore-format commits, and concise completion evidence.

## Workflow

1. Inspect scope with `git status --short` and `git diff --stat`. Do not include unrelated user changes or `.omx/` runtime files.
2. Verify code before committing:

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

3. If docs or skills changed, validate those artifacts too. For local skills, run the skill validator from `skill-creator`.
4. Review staged changes with `git diff --cached --stat` before committing.
5. Use the repository Lore commit protocol. The first line must explain why the change exists, and trailers should record confidence, scope risk, verification, and known gaps.
6. Push only when the user asked for push or the active task explicitly requires it.

## Git Hygiene

- Never use destructive cleanup commands such as `git reset --hard` or `git checkout --` unless explicitly requested.
- Do not commit generated build output, local databases, browser profile directories, or OMX runtime state.
- Prefer a narrow commit that matches the user-visible feature or maintenance task.
- If verification fails, fix the issue and rerun the relevant check before reporting completion.

## Reference

Load `references/release-verify-checklist.md` for the exact closeout checklist and commit template.
