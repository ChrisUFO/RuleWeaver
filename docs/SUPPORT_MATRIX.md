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

| Tool | Rules (Global) | Rules (Local) | Rule Model (Global) | Rule Model (Local) | Rules Dir (Global) | Rules Dir (Local) | Commands Dir (Global) | Commands Dir (Local) | Skills Dir (Global) | Skills Dir (Local) |
| ---- | -------------- | ------------- | ------------------- | ------------------ | ------------------ | ----------------- | --------------------- | -------------------- | ------------------- | ------------------ |
| Antigravity | `~/.gemini/antigravity/rules` | `.agents/rules` | `per_rule_dir` | `per_rule_dir` | ~/.gemini/antigravity/rules | .agents/rules | .gemini/antigravity/global_workflows | .agents/workflows | .gemini/antigravity/skills | .agents/skills |
| Claude Code | `~/.claude/CLAUDE.md` | `.claude/CLAUDE.md` | `single_file` | `single_file` | — | — | .claude/commands | .claude/commands | .claude/skills | .claude/skills |
| Cline | `~/Documents/Cline/Rules` | `.clinerules` | `per_rule_dir` | `per_rule_dir` | ~/Documents/Cline/Rules | .clinerules | Documents/Cline/Workflows | .clinerules/workflows | Documents/Cline/Skills | .clinerules/skills |
| Codex | `~/.codex/rules` | `.codex/rules` | `per_rule_dir` | `per_rule_dir` | ~/.codex/rules | .codex/rules | .agents/skills | .agents/skills | .codex/skills | .codex/skills |
| Cursor | `~/.cursorrules` | `.cursorrules` | `single_file` | `single_file` | — | — | .cursor/commands | .cursor/commands | — | — |
| Gemini | `~/.gemini/GEMINI.md` | `.gemini/GEMINI.md` | `single_file` | `single_file` | — | — | .gemini/commands | .gemini/commands | .gemini/skills | .gemini/skills |
| Kilo Code | `~/.kilocode/rules` | `.kilocode/rules` | `per_rule_dir` | `per_rule_dir` | ~/.kilocode/rules | .kilocode/rules | — | — | — | — |
| OpenCode | `~/.config/opencode/rules` | `.opencode/rules` | `per_rule_dir` | `per_rule_dir` | ~/.config/opencode/rules | .opencode/rules | .config/opencode/commands | .opencode/commands | .config/opencode/skills | .opencode/skills |
| Roo Code | `~/.roo/rules` | `.roo/rules` | `per_rule_dir` | `per_rule_dir` | ~/.roo/rules | .roo/rules | .roo/commands | .roo/commands | .roo/skills | .roo/skills |
| Windsurf | `~/.windsurf/rules` | `.windsurfrules` | `per_rule_dir` | `single_file` | ~/.windsurf/rules | — | — | — | .windsurf/skills | .windsurf/skills |

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
