# Project Strategy: Multi-Rule & Multi-File-Model Fix

## 1. High-Level Strategy

### Problem Statement

RuleWeaver has two interlocked bugs that mean users' rules are silently lost or written to wrong paths:

**Bug 1 — Single-file overwrite (all adapters).**
`compute_desired_state_rules()` iterates over all rules and calls `HashMap::insert()` for each one. Since every global rule for the same adapter resolves to the same path (e.g., `~/.claude/CLAUDE.md`), the last rule in the DB query order wins and all earlier rules are discarded. A user with three global Claude Code rules only ever has one rule on disk.

**Bug 2 — Wrong file model for seven adapters.**
The registry's `global_path` / `local_path_template` fields point to single files for adapters that natively expect **per-rule files in a directory**. The reconciler then reads, writes, and imports from the wrong paths entirely.

| Adapter | Current Registry Path | Correct Behavior | Gap |
|---|---|---|---|
| Claude Code | `~/.claude/CLAUDE.md` | Single aggregated file | ✅ Correct model; only Bug 1 |
| Gemini | `~/.gemini/GEMINI.md` | Single aggregated file | ✅ Correct model; only Bug 1 |
| Cursor | `~/.cursorrules` | Single aggregated file | ✅ Correct model; only Bug 1 |
| Windsurf | `~/.windsurf/rules/rules.md` | Per-rule dir (global) `~/.windsurf/rules/*.md`; single (local) `.windsurfrules` | ❌ Wrong paths, wrong model |
| Roo Code | `~/.roo/rules/rules.md` | Per-rule dir `~/.roo/rules/*.md` | ❌ Wrong model |
| Kilo Code | `~/.kilocode/rules/AGENTS.md` | Per-rule dir `~/.kilocode/rules/*.md` | ❌ Wrong model |
| OpenCode | `~/.config/opencode/AGENTS.md` | Per-rule dir `~/.config/opencode/rules/*.md` | ❌ Wrong path and model |
| Cline | `~/.clinerules` | Per-rule dir `~/Documents/Cline/Rules/*.md` (global); dir `.clinerules/*.md` (local) | ❌ Wrong global path and model |
| Antigravity | `~/.gemini/GEMINI.md` | Per-rule dir `~/.gemini/antigravity/rules/*.md` (global); `.agents/rules/*.md` (local) | ❌ WRONG PATH (using Gemini's path!) |
| Codex | `~/.codex/AGENTS.md` | Per-rule dir `~/.codex/rules/*.md` | ❌ Wrong model |

**Bug 3 — Import discovers wrong paths.**
The `rule_import` engine has a separate hardcoded list of tool paths (`global_tool_paths()`, `local_tool_paths()`) that mirrors the same wrong assumptions. For per-rule adapters, it looks for single files instead of scanning directories, so it misses all but the first rule the user ever created.

### Architecture Decision

We introduce a **`RuleFileModel`** concept in the registry. Each adapter declares whether its rules live in:
- `SingleFile` — all enabled rules for that adapter are concatenated (with section headers) into one file
- `PerRuleDir` — each rule is written as an individual `<slug>.md` file inside a directory

The reconciliation engine, import engine, path resolver, formatter, and status projection all branch on this model. Existing tests are preserved and extended.

---

## 2. Implementation Plan

### Phase 1: Branch & Registry Architecture
> Add `RuleFileModel` to the registry; update all adapter entries; add `global_rules_dir` / `local_rules_dir_template` fields for per-rule adapters; fix Antigravity's wrong paths. No behavior change yet — just the data model.

**Key tasks:**
- Create feature branch `feat/multi-rule-file-model`
- Add `RuleFileModel` enum (`SingleFile`, `PerRuleDir`) to `registry.rs`
- Add `global_rules_dir: Option<&'static str>` and `local_rules_dir_template: Option<&'static str>` to `PathTemplates`
- Populate new fields for all ten adapters:
  - `claude_code`, `gemini`, `cursor` → `RuleFileModel::SingleFile`, no dir fields
  - `opencode`, `cline`, `roo_code`, `antigravity`, `windsurf`, `kilo`, `codex` → `RuleFileModel::PerRuleDir`, fill in correct directory paths
  - Fix **Antigravity** `global_path` (currently `~/.gemini/GEMINI.md` — wrong!) to `~/.gemini/antigravity/rules` dir; local `.agents/rules`
- Add `rule_file_model(&self) -> RuleFileModel` helper to `ToolEntry`
- Add registry tests asserting each adapter's correct model
- Regenerate `docs/SUPPORT_MATRIX.md` (add rules-dir column)
- Update `docs/ai-tools-commands-reference.md` to match

### Phase 2: Path Resolver
> Extend the path resolver to produce per-rule file paths for `PerRuleDir` adapters.

**Key tasks:**
- Add `rule_file_path(adapter, rule_name, scope, repo_root?) -> Result<ResolvedPath>` to the path resolver
  - Sanitizes rule name to a safe filename slug (lowercase, hyphens, alphanumeric)
  - Returns `<rules_dir>/<slug>.md` for per-rule adapters
  - For single-file adapters, delegates to existing `global_path()` / `local_path()`
- Add `rules_dir(adapter, scope, repo_root?) -> Result<ResolvedPath>` for directory scanning
- Unit-test all ten adapters × both scopes

### Phase 3: Reconciliation Engine — Write Side (Bug 1 & 2)
> Fix both the overwrite bug (single-file aggregation) and add per-rule directory writes.

**Key tasks:**

*3a. Fix single-file aggregation (Bug 1):*
- Refactor `compute_desired_state_rules()`:
  - Pre-collect rules grouped by `(adapter, path_str)` → `Vec<&Rule>`
  - After grouping, format all rules for that path into one combined document
  - Insert one `ExpectedArtifact` per unique `(adapter, path)` pair
  - `ExpectedArtifact.id` = synthetic key `"rules-{adapter}-{scope}"`, `name` = adapter display name + " Rules"

*3b. Add per-rule directory writes (Bug 2):*
- In `compute_desired_state_rules()`, branch on `rule_file_model()`:
  - `SingleFile` → existing (fixed) aggregation path
  - `PerRuleDir` → per-rule: one `ExpectedArtifact` per (rule, adapter) pair; `id` = `rule.id`, `name` = `rule.name`
- Update `formatter.rs`:
  - `format_rule_content_standalone(name, content)` → for per-rule files (clean standalone doc)
  - `format_rule_content_aggregate(rules: &[(&str, &str)])` → for single-file adapters (each rule as a `## Rule Name` section)

### Phase 4: Reconciliation Engine — Scan Side
> Fix actual-state scanning to read per-rule directories correctly.

**Key tasks:**
- Refactor `scan_actual_state_rules()`:
  - For `SingleFile` adapters: scan one file (current behavior)
  - For `PerRuleDir` adapters: scan the directory, read each `*.md` file inside it
- Add `scan_rule_directory()` helper (analogous to `scan_skill_directory()` / `scan_command_directory()`)
  - Only picks up files that contain `RULEWEAVER_MARKER` (preserves safety guarantee)
- Update `reconcile_after_mutation()` to correctly remove stale per-rule files when a rule is deleted, disabled, or retargeted

### Phase 5: Import Engine — Read Side (Bug 3)
> Fix the import engine to use per-rule directories for the adapters that need them.

**Key tasks:**
- Refactor `global_tool_paths()` to use the REGISTRY (not a hardcoded list):
  - `SingleFile` adapters → single-file `ToolPath` (current behavior)
  - `PerRuleDir` adapters → produce a `ToolPath` where `.path` is the rules directory
- Add a new import path type: `ToolPathKind::RulesDir` vs `ToolPathKind::SingleFile`
- In the import scanning logic: when `ToolPathKind::RulesDir`, list `*.md` files in the directory and yield one import candidate per file (each becomes one rule)
- For `SingleFile` adapters with multiple `## Rule Name` sections (RuleWeaver-managed files), keep existing multi-section split logic
- Fix `local_tool_paths()` similarly using the registry's `local_rules_dir_template`
- Fix `detect_artifact_type_from_path()` to recognize per-rule directory patterns
- Fix Antigravity's wrong hardcoded paths in import

### Phase 6: Frontend
> Update the rule editor preview and path display for per-rule adapters.

**Key tasks:**
- Add `ruleFileModel: "single_file" | "per_rule_dir"` to the `ToolEntry` type in `src/types/rule.ts`
- Update the registry API response to expose the new model field and rules dir paths
- Update `getAdapterPath()` in `useRuleEditorState.ts` to return `<rules_dir>/<slug>.md` for `PerRuleDir` adapters
- Update `generatePreview()` in `useRuleEditorState.ts` to format content as a standalone document for `PerRuleDir` adapters (no composite headers)
- Update `RulePreviewCard` "Target" footer to show the per-rule file path

### Phase 7: Tests, Polish & Hardening
> Ensure full coverage across both Rust and frontend. Harden edge cases.

**Key tasks:**
- Rust: slug collision handling (two rules with same name → `rule-name-2.md`)
- Rust: empty slug fallback to `rule-{id}.md`
- Rust: slug truncated to safe filesystem length
- Rust: Windsurf global = per-rule, Windsurf local = single file
- Rust: idempotency test still passes with new model
- Rust: dry-run previews correctly for per-rule adapters
- Parity test: every adapter with `global_rules_dir: Some(...)` has `PerRuleDir` model

### Phase 8: Documentation & Verification
> Update all docs. Run full test suite. Open PR.

**Key tasks:**
- Update `architecture.md` — document the two rule file models
- Verify `docs/ai-tools-commands-reference.md` paths match updated registry exactly
- Regenerate `docs/SUPPORT_MATRIX.md` via `cargo run --bin gen_docs`
- Run full test suite; TypeScript check
- Open PR referencing this plan

### Phase 9: Follow-up Issues (#88, #89, #90)
> Fold post-implementation hardening into this same delivery.

**Issue #88 — Pre-push diagnostics**
- Add explicit step start/end markers in `.husky/pre-push`
- Print failing command and exit code in final failure summary
- Write full hook output to deterministic log file (`.git/hooks-logs/pre-push.log` by default)
- Document troubleshooting and custom log path override (`HOOK_LOG_PATH=...`)

**Issue #89 — RuleFileModel integration matrix**
- Add integration tests for `single_file` adapters across global + local scopes
- Add integration tests for `per_rule_dir` adapters across global + local scopes
- Add integration tests for mixed model adapters (Windsurf global per-rule, local single-file)
- Add import → reconcile integration test asserting no legacy OpenCode `AGENTS.md` path write

**Issue #90 — Frontend adapter path/preview tests**
- Add hook tests for `useRuleEditorState.getAdapterPath()` for per-rule and single-file adapters
- Add slug edge-case test (empty slug fallback to rule-id-based filename)
- Add preview-generation tests for both models and ensure legacy preview wrapper does not regress

---

## 3. Execution Checklist

### Branch Setup
- [ ] Create branch `feat/multi-rule-file-model`

### Phase 1: Registry Architecture
- [ ] Add `RuleFileModel` enum (`SingleFile`, `PerRuleDir`) to `registry.rs`
- [ ] Add `global_rules_dir: Option<&'static str>` to `PathTemplates`
- [ ] Add `local_rules_dir_template: Option<&'static str>` to `PathTemplates`
- [ ] Add `rule_file_model` field (or derive from `global_rules_dir` presence) to `PathTemplates`
- [ ] Update Claude Code → `SingleFile`, no dir fields
- [ ] Update Gemini → `SingleFile`, no dir fields
- [ ] Update Cursor → `SingleFile`, no dir fields
- [ ] Update OpenCode → `PerRuleDir`, `global_rules_dir: "~/.config/opencode/rules"`, local: `.opencode/rules`
- [ ] Update Cline → `PerRuleDir`, `global_rules_dir: "~/Documents/Cline/Rules"`, local: `.clinerules`
- [ ] Update Roo Code → `PerRuleDir`, `global_rules_dir: "~/.roo/rules"`, local: `.roo/rules`
- [ ] **Fix Antigravity** → `PerRuleDir`, `global_rules_dir: "~/.gemini/antigravity/rules"`, local: `.agents/rules` (fix the wrong `~/.gemini/GEMINI.md` path)
- [ ] Update Windsurf → `PerRuleDir` global `~/.windsurf/rules`, `SingleFile` local `.windsurfrules`
- [ ] Update Kilo Code → `PerRuleDir`, `global_rules_dir: "~/.kilocode/rules"`, local: `.kilocode/rules`
- [ ] Update Codex → `PerRuleDir`, `global_rules_dir: "~/.codex/rules"`, local: `.codex/rules`
- [ ] Add `rule_file_model()` helper method to `ToolEntry`
- [ ] Registry test: each adapter has correct `RuleFileModel`
- [ ] Registry test: Antigravity `global_rules_dir` is NOT `~/.gemini/GEMINI.md` (regression guard)
- [ ] Registry test: all `SingleFile` adapters have `global_rules_dir: None`
- [ ] Registry test: all `PerRuleDir` adapters have `global_rules_dir: Some(...)`
- [ ] Update `generate_support_matrix()` to include rules-dir column
- [ ] Run `cargo run --bin gen_docs` → commit updated `docs/SUPPORT_MATRIX.md`
- [ ] Update `docs/ai-tools-commands-reference.md` paths to match registry

### Phase 2: Path Resolver
- [ ] Add `slug_rule_name(name: &str) -> String` helper (safe filename from display name)
- [ ] Add `rule_file_path(adapter, rule_name, scope, repo_root) -> Result<ResolvedPath>` to path resolver
- [ ] Add `rules_dir(adapter, scope, repo_root) -> Result<ResolvedPath>` to path resolver
- [ ] Unit test `rule_file_path` for all 10 adapters × global + local scope
- [ ] Unit test `rules_dir` returns correct directory for per-rule adapters
- [ ] Test `slug_rule_name` handles spaces, special chars, unicode, empty string, very long names

### Phase 3: Reconciliation Write
- [ ] Refactor `compute_desired_state_rules()` — group rules by `(adapter, target_path)` before inserting
- [ ] Single-file branch: collect all rules for same path, concatenate into one aggregated document, insert one `ExpectedArtifact` per unique path
- [ ] Per-rule branch: insert one `ExpectedArtifact` per `(rule, adapter, scope)` triple using `rule_file_path()`
- [ ] Add `format_rule_content_standalone(name, content) -> String` to `formatter.rs`
- [ ] Add `format_rule_content_aggregate(rules: &[(&str, &str)]) -> String` to `formatter.rs`
- [ ] Test: 3 rules → Claude Code (single-file) → 1 combined file, all 3 rule names appear in content
- [ ] Test: 3 rules → OpenCode (per-rule) → 3 separate `.md` files, each has one rule's content
- [ ] Test: disabled rule excluded from desired state for both models
- [ ] Test: local-scope rules produce per-repo paths for both models
- [ ] Test: zero rules → zero entries in desired state for that adapter
- [ ] Test: single rule → single-file adapter produces the same content as the standalone format wrapped in aggregate

### Phase 4: Reconciliation Scan
- [ ] Refactor `scan_actual_state_rules()` to branch on `rule_file_model()`
- [ ] `SingleFile` branch: scan one file per adapter per scope (existing behavior)
- [ ] `PerRuleDir` branch: call `scan_rule_directory()` per adapter per scope
- [ ] Add `scan_rule_directory(dir, adapter, scope, actual) -> Result<()>` helper
- [ ] `scan_rule_directory` only picks up `*.md` files containing `RULEWEAVER_MARKER`
- [ ] Test: per-rule dir with 3 managed files → 3 entries in `actual.found_paths`
- [ ] Test: per-rule dir containing a non-RuleWeaver user file → user file is NOT touched
- [ ] Test: deleted rule causes its per-rule file to be removed at next reconcile
- [ ] Test: `reconcile_after_mutation()` removes orphaned per-rule file when rule is deleted
- [ ] Test: `reconcile_after_mutation()` removes per-rule file when rule is retargeted away from adapter

### Phase 5: Import Engine
- [ ] Introduce `ToolPathKind` enum (`SingleFile`, `RulesDir`) in `rule_import/mod.rs`
- [ ] Refactor `global_tool_paths()` to derive adapter paths from `REGISTRY`
- [ ] Refactor `local_tool_paths()` to derive adapter paths from `REGISTRY`
- [ ] In import scan: for `RulesDir` kind, enumerate `*.md` files in the directory as individual candidates
- [ ] For `SingleFile` imports with RuleWeaver `##` sections: retain existing multi-section split
- [ ] Fix Antigravity import paths — remove wrong `.antigravity/GEMINI.md` entries; add correct `~/.gemini/antigravity/rules/` dir
- [ ] Update `detect_artifact_type_from_path()` for per-rule dir patterns
- [ ] Test: import from OpenCode rules dir discovers all `.md` files as separate rule candidates
- [ ] Test: import from Claude Code `CLAUDE.md` with 3 `## Rule` sections → 3 rule candidates
- [ ] Test: import from Antigravity reads from `~/.gemini/antigravity/rules/` not Gemini's path
- [ ] Test: non-RuleWeaver `.md` file in per-rule dir can be imported as a new rule (correct behavior)
- [ ] Test: `detect_artifact_type_from_path` correctly identifies members of per-rule dirs

### Phase 6: Frontend
- [ ] Add `ruleFileModel: "single_file" | "per_rule_dir"` to `ToolEntry` type in `src/types/rule.ts`
- [ ] Add rules dir path fields to `ToolEntry` type (`globalRulesDir`, `localRulesDirTemplate`)
- [ ] Update `get_tools` Tauri command (or add new query) to expose the new fields
- [ ] Update `getAdapterPath()` in `useRuleEditorState.ts` to return per-rule path for `PerRuleDir` adapters
- [ ] Update `generatePreview()` in `useRuleEditorState.ts`: use standalone format for `PerRuleDir`, aggregate format for `SingleFile`
- [ ] Update `RulePreviewCard` "Target" footer to show per-rule file path
- [ ] Frontend test: `getAdapterPath` returns per-rule path for OpenCode
- [ ] Frontend test: `getAdapterPath` returns single-file path for Claude Code
- [ ] Frontend test: `generatePreview` for per-rule adapter has no `<!-- Rule: ... -->` aggregate wrapper
- [ ] Frontend test: `generatePreview` for single-file adapter includes rule name section header

### Phase 7: Hardening & Edge Cases
- [ ] Rust test: two rules with same slug → second gets `rule-name-2.md` (collision resolution)
- [ ] Rust test: rule name that slugifies to empty string → fallback to `rule-{id}.md`
- [ ] Rust test: very long rule name → slug is truncated to safe filesystem limit (255 bytes)
- [ ] Rust test: Windsurf global scope uses per-rule dir; Windsurf local scope uses single file `.windsurfrules`
- [ ] Rust test: `test_reconcile_all_artifact_types` still passes with new model
- [ ] Rust test: `test_reconcile_is_idempotent` still passes — reconciling twice = same result
- [ ] Rust test: dry-run mode previews per-rule creates/updates correctly
- [ ] Parity test: every adapter with `global_rules_dir: Some(...)` has `PerRuleDir` model
- [ ] Parity test: every `SingleFile` adapter has `global_rules_dir: None`
- [ ] Parity test: Antigravity global rules dir does not overlap with Gemini global rules dir

### Phase 8: Documentation & Final Verification
- [ ] Update `architecture.md` — document the `RuleFileModel` concept and two write strategies
- [ ] Verify all paths in `docs/ai-tools-commands-reference.md` match the updated registry
- [ ] Run `cargo run --bin gen_docs` → commit updated `docs/SUPPORT_MATRIX.md`
- [ ] Run `cargo test` in `src-tauri/` — all tests pass
- [ ] Run `npx vitest run` — all tests pass
- [ ] Run `npx tsc --noEmit` — no TypeScript errors
- [ ] Push branch `feat/multi-rule-file-model`; open PR referencing this plan

### Phase 9: Follow-up Issues (#88, #89, #90)
- [x] Issue #88: Add pre-push step markers, failure command/exit summary, and deterministic hook logs
- [x] Issue #88: Document pre-push troubleshooting + `HOOK_LOG_PATH` override in docs
- [x] Issue #89: Add integration tests for RuleFileModel global/local matrix (single-file + per-rule + mixed)
- [x] Issue #89: Add import→reconcile test guarding against legacy OpenCode `AGENTS.md` write path
- [x] Issue #90: Add frontend tests for `useRuleEditorState` adapter target-path behavior
- [x] Issue #90: Add frontend tests for preview generation and slug fallback edge case
