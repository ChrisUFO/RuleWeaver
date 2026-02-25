# RuleWeaver Roadmap

This roadmap reflects current implementation status and the next strategic push: robust, end-to-end lifecycle management for Rules, Custom Commands/Workflows, and Skills across all supported AI coding tools.

## Current Status Snapshot

### Delivered Foundations

- [x] Desktop app foundation (Tauri + React + Rust + SQLite)
- [x] Rules CRUD, sync adapters, and import pipeline (AI tools/files/folders/URL/clipboard)
- [x] Commands CRUD, MCP exposure, test execution, and command stub sync
- [x] Native slash command generation for supported tools
- [x] Embedded + standalone MCP runtime (`ruleweaver-mcp`)
- [x] Skills CRUD, templates, and MCP skill execution path
- [x] System tray lifecycle and background keep-alive

### Remaining Gaps to Close

- [ ] Single source of truth for tool capabilities/paths (currently duplicated across docs/frontend/backend)
- [ ] Full lifecycle parity for commands and skills import/reconcile (rules are ahead)
- [ ] Consistent adapter-specific local path resolution for all artifacts
- [ ] Unified artifact status/drift visibility and repair UX
- [ ] Documentation alignment with actual shipped behavior

## Strategic Initiative: Cross-Tool Lifecycle Hardening

Primary objective: make RuleWeaver the authoritative control plane for Rules, Commands/Workflows, and Skills with deterministic sync, import, validation, and reconciliation across tools.

### Track 1: Registry and Path Unification

_Goal: remove path/capability drift by centralizing tool metadata and path templates._

- [ ] Build canonical tool capability registry in backend (rules, command stubs, slash/workflows, skills, scope support, import support).
- [ ] Refactor backend sync/import/cleanup to consume registry-driven resolution.
- [ ] Replace duplicated frontend adapter metadata with shared/generated model.
- [ ] Add startup/runtime consistency checks for unsupported combinations.

### Track 2: Lifecycle Completeness (Create/Update/Delete/Import/Reconcile)

_Goal: achieve full lifecycle coverage beyond rules._

- [ ] Add command/workflow import scan+execute pipeline.
- [ ] Add skills import scan+execute pipeline.
- [ ] Add reconciliation for rename/delete/deselection and stale artifact cleanup.
- [ ] Ensure config import/export triggers full post-import reconciliation.

### Track 3: Skills Distribution Across Tools

_Goal: support capability-aware skills delivery in addition to MCP execution._

- [ ] Add skills sync adapters for tools that support native skill files.
- [ ] Add per-skill adapter targeting and scope-aware path controls.
- [ ] Add strict schema/type serialization parity tests between frontend and backend.

### Track 4: Operator UX and Observability

_Goal: make drift and state actionable._

- [ ] Add unified Artifact Status view (synced, missing, out-of-date, conflicted, unsupported).
- [ ] Add dry-run previews for command/slash/skill syncs (rules parity).
- [ ] Add one-click repair actions (sync, cleanup, resolve).

### Track 5: Testing and Documentation Hardening

_Goal: ship safely and keep docs truthful._

- [ ] Add Rust integration tests for registry/path resolution/import/reconcile flows.
- [ ] Add frontend tests for lifecycle flows and status UI.
- [ ] Maintain >=80% coverage for new lifecycle modules and avoid overall regression.
- [ ] Update `architecture.md`, `README.md`, `USER_GUIDE.md`, and `docs/ai-tools-commands-reference.md` to match implementation.

## Milestone Mapping (GitHub)

- Milestone #6: Cross-Tool Lifecycle Hardening (single milestone for all 13 lifecycle issues)

## GitHub Execution Plan

### Milestone #6: Cross-Tool Lifecycle Hardening

Implementation order is tracked inside this single milestone:

1. Foundation & Registry

- [ ] #31 - [6.1][Lifecycle] Canonical tool capability registry
- [ ] #42 - [6.2][Lifecycle] Replace duplicated frontend adapter metadata
- [ ] #38 - [6.3][Lifecycle] Add capability and path consistency checks

2. Sync & Reconciliation

- [ ] #41 - [6.4][Lifecycle] Adapter-specific local rule path resolver
- [ ] #43 - [6.5][Lifecycle] Harden slash cleanup/remove path resolution
- [ ] #39 - [6.6][Lifecycle] Artifact reconciliation engine (rename/delete/deselect)

3. Import Expansion

- [ ] #37 - [6.7][Lifecycle] Command/workflow import pipeline
- [ ] #33 - [6.8][Lifecycle] Skills import pipeline
- [ ] #40 - [6.9][Lifecycle] Full post-import reconciliation across artifacts

4. Skills Distribution & UX

- [ ] #45 - [6.10][Lifecycle] Skills sync adapters with capability-aware targeting
- [ ] #47 - [6.11][Lifecycle] Unified artifact status and repair actions

5. Testing & Documentation

- [ ] #44 - [6.12][Lifecycle] Integration test matrix and coverage gates
- [ ] #46 - [6.13][Lifecycle] Align architecture and user docs with shipped behavior

## Success Criteria

- One canonical capability/path definition drives frontend, backend, and docs.
- Rules/Commands/Skills all support deterministic lifecycle operations across tools.
- Drift is visible, diagnosable, and repairable from the UI.
- Regression risk is constrained by integration coverage and updated operator docs.
