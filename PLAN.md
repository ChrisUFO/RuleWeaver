# Project Strategy: Dashboard Artifact Health and Sync Progress Clarity (#79, #80)

## 1. High-Level Strategy

The objective is to make Dashboard a trustworthy operational surface for all first-class artifacts (rules, commands, and skills) while making sync activity transparent during execution.

Based on issue analysis and codebase verification:

- Issue `#79` requires Dashboard counts and health to cover commands and skills, not only rules.
- Issue `#80` requires meaningful in-flight sync context (current item + index), with graceful fallback when details are missing.
- Current implementation in `src/components/pages/Dashboard.tsx` is rules-centric (`useRulesStore`) and initializes sync progress fields but does not stream/update per-step details.
- Existing architecture supports this direction: unified status and reconciliation logic already exists (`src/components/pages/Status.tsx`, `src-tauri/src/status/mod.rs`) and should be reused as source data.
- Architecture validation result: core architecture still matches code layout (`src` frontend + `src-tauri/src` backend engines), but `architecture.md` contains a stale PLAN reference to issues `#74-#78`; this should be updated as part of documentation polish.

Constraint check against `architecture.md`: no conflict. The requested work aligns with the stated system design (unified artifact lifecycle + reconciliation-driven health), provided we avoid creating duplicate truth sources and instead build Dashboard summaries from existing APIs.

Implementation strategy:

1. Introduce a dashboard-focused aggregation layer that combines rules/commands/skills counts with status health signals.
2. Add event-driven sync progress updates from backend sync flow to frontend UI state.
3. Preserve current sync result behavior and dialogs while improving contextual visibility.
4. Harden with high-value tests (happy path + failure/degraded path), targeting >=80% coverage on modified modules without adding low-value tests.

## 2. Implementation Plan

### Phase 1: Branch Setup and Baseline Contract

- Create a dedicated feature branch for issues `#79` and `#80`.
- Capture current Dashboard and sync-progress behavior as baseline.
- Define a clear dashboard data contract for counts and health semantics (entity counts vs status health).
- Document architecture constraints and stale-doc items to close during final polish.

### Phase 2: Multi-Artifact Dashboard Metrics Foundation (#79)

- Add a dashboard metrics aggregator (hook or utility) that fetches data in parallel from:
  - `api.rules.getAll()`
  - `api.commands.getAll()`
  - `api.skills.getAll()`
  - `api.status.getSummary()` and/or filtered status entries where needed
- Build normalized metric outputs for:
  - Rules count (+ enabled)
  - Commands count
  - Skills count (+ enabled)
  - Health indicators (synced/attention states) from status data
- Keep terminology consistent with "artifact" language across Dashboard copy.

### Phase 3: Dashboard UI Integration and Readability Polish (#79)

- Update Dashboard stat cards to represent all artifact types while keeping scanability.
- Keep existing rules metrics accurate and visible.
- Integrate health indicators that reflect full workspace state, not rules-only drift.
- Preserve responsive readability on desktop and mobile breakpoints.
- Reuse existing visual patterns and avoid introducing a conflicting dashboard style.

### Phase 4: Sync Progress Event Pipeline in Backend (#80)

- Introduce a typed sync progress payload in Rust (current item, current index, total, optional step/context).
- Emit progress lifecycle events from sync flow (start, per-item update, complete/fail) during `sync_rules` execution.
- Ensure payloads remain valid when item details are unavailable.
- Keep existing sync success/failure result objects and logs unchanged.
- Limit event noise by emitting at meaningful boundaries (not excessive granular spam).

### Phase 5: Frontend Sync Progress Mapping and Rendering (#80)

- Add frontend listener wiring for sync progress events in Dashboard scope.
- Map event payloads into existing `SyncProgress` state model with resilient defaults.
- Render current item and item index context when available; gracefully fallback when missing.
- Prevent flicker/noisy redraws through guarded state updates.
- Keep completion dialogs and sync outcome UX functionally unchanged.

### Phase 6: Test Coverage, Hardening, and Edge Cases

- Frontend tests (Vitest + RTL):
  - Multi-artifact metric aggregation and card rendering.
  - Health indicator behavior for mixed status states.
  - Sync progress detail rendering and fallback when detail fields are missing.
  - Regression checks for unchanged sync completion/result dialogs.
- Rust tests:
  - Progress payload sequencing and index/total correctness.
  - Graceful behavior when current item detail is absent.
- Apply testing rule:
  - Separate happy-path and failure/degraded-path tests.
  - Target >=80% coverage for modified modules, without low-value tests.

### Phase 7: Documentation and Final Verification

- Update `architecture.md` to reflect the new active plan context and remove stale issue-range reference.
- Update `USER_GUIDE.md` dashboard/sync sections for multi-artifact metrics and progress transparency.
- Update any roadmap/changelog references touched by implementation.
- Run full verification commands and perform manual desktop/mobile UX checks.
- Prepare PR notes linking issues `#79` and `#80` with test evidence.

## 3. Execution Checklist

- [ ] Create and switch to feature branch `feat/dashboard-artifact-health-progress`
- [ ] Validate current behavior in `src/components/pages/Dashboard.tsx` against issue #79 and #80 requirements
- [ ] Define and document dashboard metric semantics (counts and health source-of-truth)
- [ ] Implement dashboard metrics data loader with parallel API fetches for rules, commands, skills, and status
- [ ] Add normalized view-model for rules/commands/skills count cards and health indicators
- [ ] Update Dashboard labels/copy to use artifact terminology consistently
- [ ] Render rules, commands, and skills metrics without overloading the card layout
- [ ] Preserve existing rules metrics accuracy while expanding cross-artifact visibility
- [ ] Verify dashboard card readability on desktop and mobile breakpoints
- [ ] Add Rust sync progress event payload type(s) and serialization contract
- [ ] Emit sync start/progress/complete (and failure) events during rules sync execution
- [ ] Ensure sync progress emission is resilient when current item details are unavailable
- [ ] Add Dashboard-side listener and mapper for sync progress events
- [ ] Wire mapped event state into `SyncProgress` rendering
- [ ] Implement fallback rendering when progress detail fields are missing
- [ ] Add guardrails to avoid noisy/flickery progress UI updates
- [ ] Keep sync completion and result dialog behavior unchanged
- [ ] Add frontend lifecycle tests for multi-artifact metric aggregation and rendering
- [ ] Add frontend tests for detailed sync progress rendering updates
- [ ] Add frontend tests for missing-detail fallback behavior in progress UI
- [ ] Add frontend regression tests confirming sync completion/result UX remains intact
- [ ] Add Rust tests for progress sequencing and index/total consistency
- [ ] Add Rust tests for progress payload behavior with missing current-item detail
- [ ] Run `npm run test`
- [ ] Run `npm run test:coverage` and verify >=80% coverage target on modified modules (high-value tests only)
- [ ] Run `npm run test:rust`
- [ ] Update `architecture.md` to remove stale PLAN issue-range reference and reflect current plan context
- [ ] Update `USER_GUIDE.md` with dashboard multi-artifact and sync progress behavior
- [ ] Update any touched roadmap/release documentation to keep docs in sync with implementation
- [ ] Perform manual QA pass for dashboard + sync flow on desktop and mobile widths
- [ ] Prepare PR description linking #79 and #80 with validation evidence
