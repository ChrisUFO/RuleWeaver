<!-- AUTO-GENERATED: do not edit manually. Run `cargo run --bin gen_docs` to regenerate. -->
# RuleWeaver Tool Support Matrix

Generated from `src-tauri/src/models/registry.rs`. Any change to adapter capabilities or paths must be followed by running `cargo run --bin gen_docs` and committing the updated file.

---

## Capability Flags

| Tool | Rules | Command Stubs | Slash Commands | Skills | Global Scope | Local Scope |
| ---- | :---: | :-----------: | :------------: | :----: | :----------: | :---------: |
| Antigravity | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Claude Code | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Cline | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Codex | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Cursor | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ |
| Gemini | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Kilo Code | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ |
| OpenCode | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Roo Code | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Windsurf | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ |

---

## Path Configuration

Paths prefixed with `~/` expand to the user home directory at runtime.

| Tool | Rules (Global) | Rules (Local) | Commands Dir (Global) | Commands Dir (Local) | Skills Dir (Global) | Skills Dir (Local) |
| ---- | -------------- | ------------- | --------------------- | -------------------- | ------------------- | ------------------ |
| Antigravity | `~/.gemini/GEMINI.md` | `.gemini/GEMINI.md` | .gemini/antigravity/global_workflows | .agents/workflows | .gemini/antigravity/skills | .agents/skills |
| Claude Code | `~/.claude/CLAUDE.md` | `.claude/CLAUDE.md` | .claude/commands | .claude/commands | .claude/skills | .claude/skills |
| Cline | `~/.clinerules` | `.clinerules` | Documents/Cline/Workflows | .clinerules/workflows | Documents/Cline/Skills | .clinerules/skills |
| Codex | `~/.codex/AGENTS.md` | `.codex/AGENTS.md` | .agents/skills | .agents/skills | .codex/skills | .codex/skills |
| Cursor | `~/.cursorrules` | `.cursorrules` | .cursor/commands | .cursor/commands | — | — |
| Gemini | `~/.gemini/GEMINI.md` | `.gemini/GEMINI.md` | .gemini/commands | .gemini/commands | .gemini/skills | .gemini/skills |
| Kilo Code | `~/.kilocode/rules/AGENTS.md` | `.kilocode/rules/AGENTS.md` | — | — | — | — |
| OpenCode | `~/.config/opencode/AGENTS.md` | `.config/opencode/AGENTS.md` | .config/opencode/commands | .opencode/commands | .config/opencode/skills | .opencode/skills |
| Roo Code | `~/.roo/rules/rules.md` | `.roo/rules/rules.md` | .roo/commands | .roo/commands | .roo/skills | .roo/skills |
| Windsurf | `~/.windsurf/rules/rules.md` | `.windsurf/rules/rules.md` | — | — | .windsurf/skills | .windsurf/skills |

---

## Slash Command Extensions

| Tool | File Extension | Argument Pattern |
| ---- | -------------- | ---------------- |
| Antigravity | `md` | `—` |
| Claude Code | `md` | `$ARGUMENTS` |
| Cline | `md` | `—` |
| Codex | `md` | `—` |
| Cursor | `md` | `—` |
| Gemini | `toml` | `{{args}}` |
| Kilo Code | `—` | `—` |
| OpenCode | `md` | `$ARGUMENTS` |
| Roo Code | `md` | `—` |
| Windsurf | `—` | `—` |

---

*See `docs/PARITY.md` for documented divergences and known unsupported combinations.*
