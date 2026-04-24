# Installing Rivulet Skills

Rivulet includes project-specific agent skills in `skills/`. Keep that folder as the source of truth, then install or symlink the skills into your agent with the Vercel Labs `skills` CLI.

## Recommended Project Install

Install the local skills for Codex from the repository root:

```bash
npx skills add ./skills -a codex -y
```

This installs the skills for the current project. For Codex, the Vercel Labs tool uses `.agents/skills/` for project installs.

Restart Codex after installing so the new skills are loaded.

## Preview Before Installing

List the skills detected in this repository:

```bash
npx skills add ./skills -a codex --list
```

List installed Codex skills:

```bash
npx skills list -a codex
```

## Global Install

Use a global install only if you want these Rivulet skills available outside this repository:

```bash
npx skills add ./skills -a codex -g -y
```

For Codex, global installs go under `~/.codex/skills/`.

## Symlink vs Copy

The interactive installer can install by symlink or copy. Prefer symlink mode when available, because `skills/` remains the single source of truth and updates are immediately reflected in the installed agent path.

Use copy mode when symlinks are not supported or when you intentionally want an independent snapshot:

```bash
npx skills add ./skills -a codex --copy -y
```

## Installing Skills From GitHub

The same CLI can install skills from public GitHub repositories.

List skills in a repository:

```bash
npx skills add vercel-labs/agent-skills --list
```

Install a specific skill:

```bash
npx skills add vercel-labs/agent-skills -a codex --skill frontend-design -y
```

Install from a direct skill path:

```bash
npx skills add https://github.com/vercel-labs/agent-skills/tree/main/skills/web-design-guidelines -a codex -y
```

## Rivulet Skills Included

- `rivulet-reading-workflow`: item states, queues, saved/archive behavior, filters, and TUI reading workflow.
- `rivulet-search-index`: SQLite FTS search, indexing, ranking, and search CLI/TUI work.
- `rivulet-auth-profiles`: Chrome auth profiles, paid/private-site scraping, cookie-safe session reuse.
- `rivulet-release-verify`: verification, commit hygiene, Lore commit protocol, and release closeout.

## Troubleshooting

If no skills are found, verify each skill directory contains a valid `SKILL.md`:

```bash
find skills -name SKILL.md -maxdepth 2 -print
```

Validate a skill with the Codex skill creator validator:

```bash
python3 /Users/bytedance/.codex/skills/.system/skill-creator/scripts/quick_validate.py skills/rivulet-reading-workflow
```

If a newly installed skill is not picked up, restart Codex and run:

```bash
npx skills list -a codex
```
