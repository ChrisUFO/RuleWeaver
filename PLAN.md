# Project Strategy: Milestone 15 — MCP Onboarding & Product Trust (#96, #97, #98, #52)

## 1. High-Level Strategy

Milestone 15 should make RuleWeaver feel trustworthy in four connected ways: secure workspace-aware execution, clear standalone MCP onboarding, quieter/high-signal UI tests, and documentation that matches shipped behavior.

Based on issue research and codebase verification:

- `architecture.md` is broadly accurate for the current stack and layering: React/TypeScript frontend in `src`, Rust/Tauri backend in `src-tauri/src`, SQLite-backed settings/data, reconciliation/status engines, and embedded + standalone MCP runtime.
- The documentation section of `architecture.md` is slightly stale because it still points at the previous active `PLAN.md` focus; this milestone should correct that as part of docs polish.
- Frontend structure follows `src/components`, `src/hooks`, `src/lib`, and limited Zustand stores in `src/stores`; most page behavior is coordinated through stateful hooks like `useSettingsState` and `useCommandsState`.
- Testing conventions are already established and should be followed:
  - Frontend: Vitest + React Testing Library in `src/__tests__`, shared setup in `src/test/setup.ts`, targeted lifecycle/hook tests, and parity tests.
  - Backend: Rust unit/integration tests in `src-tauri/tests` using `tokio::test`, `TempDir`, and in-memory DB helpers.
- Current MCP UX already exposes basic status, connection snippets, and recent logs via `useSettingsState`, `McpSettingsCard.tsx`, `mcp_commands.rs`, and `src-tauri/src/mcp/mod.rs`, but it does not yet provide a focused diagnostics model for common failure classes.
- Current secret handling is still global/settings-table based (for example `mcp_secrets_allowlist` and settings-wide injection during MCP skill execution), so issue `#52` requires a real scoped secret model and deterministic resolution path.
- Repository/workspace boundaries are already a first-class concept via `local_rule_paths`, `useRepositoryRoots`, `PathResolver`, and validation helpers; that should be reused for workspace secret scoping instead of inventing a parallel workspace model.

Constraint check against `architecture.md`: no conflict. The milestone aligns with the architecture as long as implementation preserves existing source-of-truth boundaries:

1. Reuse database/path-resolver/reconciliation primitives instead of creating duplicate workspace or status truth sources.
2. Keep standalone MCP status/diagnostics anchored in `McpManager` and existing Tauri command surfaces.
3. Do not hardcode secrets or add fake/example credentials; all secret handling must stay environment-/vault-driven and redaction-safe.
4. Keep documentation truthful to shipped behavior, especially around supported flows and known limitations.

Implementation strategy:

1. Establish secure workspace-aware secret resolution first because it affects execution trust and future diagnostics quality.
2. Expand MCP onboarding/status into structured diagnostics and copyable guidance, building on current Settings surfaces.
3. Remove recurring warning/test noise at the root cause in the highest-value flows instead of muting logs globally.
4. Finish by refreshing README/support/architecture docs so the product story matches reality.

## 2. Implementation Plan

### Phase 1: Feature Branch, Baseline Audit, and Milestone Contract

- Create a dedicated branch for the milestone work (recommended: `feat/15-mcp-onboarding-product-trust`).
- Capture current behavior for:
  - MCP Settings status/instructions/logs
  - warning-producing frontend test flows
  - current secret resolution/execution behavior
  - primary docs (`README.md`, `USER_GUIDE.md`, `docs/PARITY.md`, `architecture.md`)
- Produce a per-issue file map so implementation follows existing seams instead of scattering logic.
- Record stale-doc findings discovered during research so they are closed before milestone completion.

### Phase 2: Workspace-Scoped Secret Foundation (#52)

- Add the persistence/data model needed for workspace-aware secrets and, only if justified by actual usage, narrower artifact-level overrides.
- Define a deterministic resolution service with explicit precedence:
  - global baseline
  - workspace override
  - artifact-level override where supported
- Thread workspace context through command and skill execution using existing repository roots, base paths, and path resolution rules.
- Preserve redaction and logging guarantees so resolved secret values never leak through stdout/stderr history, UI diagnostics, or docs.
- Add Rust tests for precedence, workspace selection, and execution-path integration.

### Phase 3: Secret Management UX and Execution Integration (#52)

- Add UI support for viewing/managing workspace-level secrets and showing inherited vs overridden state.
- Reuse configured repository roots as the workspace selector/input model.
- Show effective scope clearly in the UI so operators can tell whether a value is global, inherited, or locally overridden.
- Update command/skill execution and MCP-related execution paths so they consume the resolved scoped secret set for the active workspace.
- Add focused frontend tests around inheritance display, override editing, and workspace-specific execution behavior.

### Phase 4: Standalone MCP Onboarding, Health, and Diagnostics (#96)

- Extend MCP backend/status modeling beyond simple running/stopped to cover trust-relevant states such as configuration incomplete, degraded startup, port conflict, and likely client misconfiguration.
- Expose structured diagnostics and copyable connection details from Tauri commands, reusing existing status/instruction/log plumbing.
- Upgrade `McpSettingsCard` and `useSettingsState` to provide:
  - explicit standalone setup guidance
  - copyable snippets/connection metadata
  - actionable troubleshooting hints
  - clearer health/status language
- Distinguish between server availability, watcher state, and configuration readiness without inventing a second status system.
- Add Rust and Vitest coverage for status transitions, diagnostics mapping, and troubleshooting copy.

### Phase 5: Frontend Warning and Test-Noise Cleanup (#97)

- Inventory recurring warning/noise sources in the highest-value flows, especially Settings/MCP, lifecycle tests, and hook tests.
- Fix root causes such as incomplete mocks, missing cleanup, noisy parsing/error paths, invalid state transitions, or deprecated usage.
- Add focused test helpers/assertions that fail on the warning classes we intentionally clean up, without suppressing useful diagnostics globally.
- Document any remaining accepted warning classes only if they are truly unavoidable in the current release.

### Phase 6: Documentation Truthfulness and Support Path Refresh (#98)

- Audit and update `README.md`, `USER_GUIDE.md`, `docs/PARITY.md`, `architecture.md`, and any high-traffic support docs touched by the milestone.
- Align terminology for embedded vs standalone MCP, diagnostics/health states, repository roots, and scoped secret inheritance.
- Remove stale claims and ensure onboarding/troubleshooting guidance matches the implemented UX.
- Add/repair cross-links so users can move cleanly between README, user guide, parity notes, and troubleshooting guidance.
- Regenerate machine-authored docs only if registry or generated docs inputs actually change.

### Phase 7: Hardening, Coverage, and Final Verification

- Run targeted tests first, then broader verification:
  - relevant Vitest files
  - `npm run test`
  - `npm run test:coverage`
  - `npm run test:rust`
- Verify the 80% coverage target on modified modules using medium/high-value tests only.
- Perform manual trust checks:
  - standalone MCP start/stop/refresh
  - copy/paste onboarding snippets
  - representative failure diagnostics
  - Repo A vs Repo B secret resolution correctness
  - quieter/high-signal logs in targeted flows
- Prepare PR notes linking issues `#96`, `#97`, `#98`, and `#52` with validation evidence.

## 3. Execution Checklist

- [ ] Create and switch to feature branch `feat/15-mcp-onboarding-product-trust`
- [ ] Capture a baseline of current MCP Settings status, instructions, and log behavior
- [ ] Capture a baseline of recurring warning/noise in targeted frontend tests and core UI flows
- [ ] Document current global-only secret behavior and the affected execution paths
- [ ] Confirm the exact file map for milestone work across `src`, `src-tauri/src`, and docs
- [ ] Design and implement workspace-scoped secret persistence for issue `#52`
- [ ] Implement deterministic secret resolution order: global → workspace → artifact override (where supported)
- [ ] Reuse repository roots/path resolver as the workspace boundary model
- [ ] Update command execution to use resolved scoped secrets for the active workspace
- [ ] Update skill execution/MCP execution paths to use resolved scoped secrets for the active workspace
- [ ] Ensure secret values remain redacted from logs/history/diagnostics surfaces
- [ ] Add Rust tests for scoped secret precedence and workspace resolution behavior
- [ ] Add UI for viewing/managing workspace-level secret values
- [ ] Show inherited vs overridden secret state clearly in the UI
- [ ] Validate workspace selection/editing against configured repository roots
- [ ] Add frontend tests for secret inheritance and override behavior
- [ ] Expand MCP backend status/diagnostics modeling for issue `#96`
- [ ] Add structured troubleshooting output for common failure modes (misconfiguration, port conflict, startup failure, mismatch)
- [ ] Keep MCP status truth anchored in existing manager/command surfaces
- [ ] Upgrade the Settings MCP card with clearer onboarding guidance and copyable connection details
- [ ] Surface health/degraded/config-incomplete states clearly in the UI
- [ ] Add frontend and Rust tests for MCP status/diagnostic transitions
- [ ] Identify the highest-value recurring frontend warning/noise sources for issue `#97`
- [ ] Fix warning root causes instead of broadly muting console output
- [ ] Add targeted warning-regression guards in high-value tests
- [ ] Document any intentionally accepted remaining warning classes
- [ ] Audit `README.md` against implemented behavior for issue `#98`
- [ ] Audit `USER_GUIDE.md` onboarding and troubleshooting sections against implemented behavior
- [ ] Audit `docs/PARITY.md` and related support docs for stale/conflicting claims
- [ ] Update `architecture.md` to reflect the new active plan context and any milestone-driven behavior changes
- [ ] Align cross-links and terminology across primary docs
- [ ] Run targeted Vitest suites for touched hooks/components/pages
- [ ] Run `npm run test`
- [ ] Run `npm run test:coverage` and confirm strong coverage on modified modules
- [ ] Run `npm run test:rust`
- [ ] Run `npm run gen:docs` only if generated capability docs become stale
- [ ] Perform manual QA for standalone MCP onboarding, diagnostics, and scoped-secret behavior
- [ ] Verify targeted test output is materially quieter and higher signal than baseline
- [ ] Prepare PR description linking issues `#96`, `#97`, `#98`, and `#52` with validation evidence
