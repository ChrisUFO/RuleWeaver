# Migration Notes

This document records breaking path or capability changes between milestones.
Entries are ordered newest-first.

---

## Milestone: feat/skills-status-execution-depth

### Skills Distribution (Native SKILL.md files)

RuleWeaver now writes native `SKILL.md` files into each AI tool's skill directory
on every reconcile cycle. No manual migration is needed; new skill files will
appear automatically after the first reconcile on upgrade.

Supported adapter skill directories:

| Adapter     | Global skill path                       |
| ----------- | --------------------------------------- |
| Claude Code | `~/.claude/skills/<slug>/SKILL.md`      |
| Gemini      | `~/.gemini/skills/<slug>/SKILL.md`      |
| Windsurf    | `~/.windsurf/skills/<slug>/SKILL.md`    |
| Cline       | `~/.cline/skills/<slug>/SKILL.md`       |
| Roo         | `~/.roo/skills/<slug>/SKILL.md`         |
| OpenCode    | `~/.opencode/skills/<slug>/SKILL.md`    |
| Antigravity | `~/.antigravity/skills/<slug>/SKILL.md` |
| Codex       | `~/.codex/skills/<slug>/SKILL.md`       |

Adapters that do **not** support skills (`supports_skills: false`) — Cursor and
Kilo Code — will never have skill files written to them, even if they appear in
`targetAdapters`. Unsupported adapters are silently skipped by the reconciler.

### DB Migration v13 — `target_adapters` / `target_paths` on Skills

Two columns were added to the `skills` table:

| Column            | Type | Default | Meaning                                                             |
| ----------------- | ---- | ------- | ------------------------------------------------------------------- |
| `target_adapters` | TEXT | `"[]"`  | JSON array of adapter IDs to distribute to; empty = all supported   |
| `target_paths`    | TEXT | `"[]"`  | JSON array of explicit file paths (advanced / local-scope override) |

The migration runs automatically on first launch via the embedded migration
engine (`PRAGMA user_version = 13`). Existing skills default to empty arrays,
which means **all supported adapters** — identical to pre-migration behavior.
No data loss occurs and no manual SQL is required.

### Windsurf Skills Path

Windsurf users will see new skill files written to `~/.windsurf/skills/` after
upgrading. This directory may need to be created on first use; RuleWeaver creates
it automatically if absent.

Previously, Windsurf skills were not distributed (the registry entry was
incomplete). The capability flag is now `supports_skills: true` with a
configured path.

---

## Milestone: feat/skills-status-execution-depth (Adapter Capability Notes)

### Kilo Code — Skills and Command Stubs Not Yet Distributed

Kilo Code's registry entry has capability flags `supports_skills: true` and
`supports_command_stubs: true`, but the directory paths are not yet configured
(`None`). As a result, **no files are distributed to Kilo Code** for skills or
command stubs in this release. This will be resolved once Kilo Code publishes
their directory spec.

See `docs/PARITY.md` for the current known-divergence log.

### Cursor — No Command Stubs, No Skills

Cursor's registry entry has `supports_command_stubs: false` and
`supports_skills: false`. Cursor users receive rule files (`.cursorrules`) and
slash commands (`.cursor/commands/*.md`) only. Custom command stubs and skill
files are never written for Cursor.

---

## Regenerating `docs/SUPPORT_MATRIX.md`

`docs/SUPPORT_MATRIX.md` is generated from the canonical `REGISTRY` constant in
`src-tauri/src/models/registry.rs`. If you modify adapter capability flags or
paths, regenerate the file before committing:

```bash
npm run gen:docs
```

Or directly:

```bash
cargo run --manifest-path src-tauri/Cargo.toml --bin gen_docs
```

The CI `docs-check` job will fail on any PR where `SUPPORT_MATRIX.md` is stale,
printing:

```
docs/SUPPORT_MATRIX.md is stale — run `npm run gen:docs` and commit the result
```

---

## Earlier Milestones

See `docs/slash-command-migration-guide.md` for migration notes from the Native
Slash Commands milestone.
