# Platform Adapter Parity

This document is the source of truth for capability divergence between AI tool adapters supported by RuleWeaver. It is updated per release and reflects the registry definitions in `src-tauri/src/models/registry.rs`.

Last updated: 2026-02-26 (release: feat/skills-status-execution-depth)

---

## Adapter Capability Matrix

| Adapter     | Rules | Command Stubs | Slash Commands | Skills | Global Scope | Local Scope |
| ----------- | :---: | :-----------: | :------------: | :----: | :----------: | :---------: |
| Antigravity |  ✅   |      ✅       |       ✅       |   ✅   |      ✅      |     ✅      |
| Claude Code |  ✅   |      ✅       |       ✅       |   ✅   |      ✅      |     ✅      |
| Cline       |  ✅   |      ✅       |       ✅       |   ✅   |      ✅      |     ✅      |
| Codex       |  ✅   |      ✅       |       ✅       |   ✅   |      ✅      |     ✅      |
| Cursor      |  ✅   |      ❌       |       ✅       |   ❌   |      ✅      |     ✅      |
| Gemini      |  ✅   |      ✅       |       ✅       |   ✅   |      ✅      |     ✅      |
| Kilo Code   |  ✅   |      ✅       |       ✅       |  ✅\*  |      ✅      |     ✅      |
| OpenCode    |  ✅   |      ✅       |       ✅       |   ✅   |      ✅      |     ✅      |
| Roo Code    |  ✅   |      ✅       |       ✅       |   ✅   |      ✅      |     ✅      |
| Windsurf    |  ✅   |      ✅       |      ✅\*      |   ✅   |      ✅      |     ✅      |

---

## Documented Divergences

### Cursor — No Command Stubs, No Skills

**Capability flags:** `supports_command_stubs: false`, `supports_skills: false`

Cursor does not expose a markdown command stub format or a skills directory convention. Commands are not distributed as stubs to Cursor. Skills are not written to any Cursor directory.

- Slash commands are supported via `.md` extension in `.cursor/commands/`.
- Rules are written to `~/.cursorrules` (global) or `.cursorrules` (local).
- No command stub file (`COMMANDS.md`) is written.
- MCP fallback applies for commands that have Cursor in their adapter list (command is exposed via MCP only).

### Kilo Code — Skills Capability Flag Set but Paths Not Configured

**Capability flags:** `supports_skills: true`
**Path config:** `global_skills_dir: None`, `local_skills_dir: None`

Kilo Code has `supports_skills: true` in its capability flags, but both `global_skills_dir` and `local_skills_dir` are `None` in the path configuration. This means:

- The capability flag is set in anticipation of future Kilo Code skills support.
- The reconciliation engine will skip Kilo Code when writing skill files (no path to write to).
- No `SKILL.md` files are distributed to Kilo Code directories until path configuration is added.
- Similarly, `global_commands_dir` and `local_commands_dir` are `None`, so no command stubs are written for Kilo Code despite `supports_command_stubs: true`.

**Action required when Kilo Code publishes their skills directory spec:** update `PathTemplates` in `registry.rs` to set `global_skills_dir` and `local_skills_dir`.

### Windsurf — Slash Commands Capability Flag Set but Paths Not Configured

**Capability flags:** `supports_slash_commands: true`
**Path config:** `slash_command_extension: None`, `global_commands_dir: None`, `local_commands_dir: None`

Windsurf has `supports_slash_commands: true` but no slash command extension, global commands directory, or local commands directory is configured. This means:

- No slash command files (e.g., `command.md`) are written to any Windsurf directory.
- The slash command generation UI will not offer Windsurf as a target adapter.
- Skills are supported and written to `~/.windsurf/skills/` (global) and `.windsurf/skills/` (local).

**Action required when Windsurf publishes their slash command spec:** update `PathTemplates` to set `slash_command_extension`, `global_commands_dir`, and `local_commands_dir`.

---

## Fully Supported Adapters

The following adapters have full capability support for all artifact types with all paths configured:

- **Antigravity** — global: `~/.gemini/antigravity/skills`, local: `.agents/skills`
- **Claude Code** — global: `~/.claude/skills`, local: `.claude/skills`
- **Cline** — global: `Documents/Cline/Skills`, local: `.clinerules/skills`
- **Codex** — global: `~/.codex/skills`, local: `.codex/skills`
- **Gemini** — global: `~/.gemini/skills`, local: `.gemini/skills`
- **OpenCode** — global: `~/.config/opencode/skills`, local: `.opencode/skills`
- **Roo Code** — global: `~/.roo/skills`, local: `.roo/skills`

---

## Capability Detection vs. Platform Detection

Per the monorepo shared code rule: **prefer capability checks over platform checks**.

In the reconciliation engine (`src-tauri/src/reconciliation/mod.rs`), adapter targeting uses `registry.validate_support(adapter, scope, ArtifactType::Skill)` rather than branching on adapter identity. UI components (e.g., the Skills editor adapter checkbox list) are populated by `get_skill_supported_adapters`, which filters the registry by `supports_skills: true` and the presence of a configured skills directory path.

---

## Machine-Readable Reference: `docs/SUPPORT_MATRIX.md`

`docs/SUPPORT_MATRIX.md` is **generated directly from the `REGISTRY` constant** in
`src-tauri/src/models/registry.rs` by the `gen_docs` binary. It is the authoritative,
machine-readable capability reference and will never drift from the code.

- Regenerate after any registry change: `npm run gen:docs`
- The CI `docs-check` job fails on any PR where the file is stale
- A Rust test (`test_support_matrix_is_current` in `registry.rs`) also asserts freshness

Use `PARITY.md` (this file) for human-authored narrative about known divergences.
Use `SUPPORT_MATRIX.md` for the exhaustive per-adapter × per-capability matrix.

---

## Updating This Document

When adding or modifying an adapter in `src-tauri/src/models/registry.rs`:

1. Update the capability matrix table above.
2. Add a "Documented Divergences" section for any capability flag / path config discrepancy.
3. Remove divergence entries when the discrepancy is resolved (paths configured to match the capability flag).
4. Update the "Last updated" line with the current date and release branch.
5. Run `npm run gen:docs` to regenerate `docs/SUPPORT_MATRIX.md`.
