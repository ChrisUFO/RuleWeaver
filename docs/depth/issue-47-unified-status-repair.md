## Problem

Health/drift visibility is currently fragmented and rule-centric. Users need a single operator view across all artifacts with immediate repair actions.

## User Stories

- As a user, I want one place to see sync health for rules, command stubs, slash commands, and skills.
- As an operator, I want one-click repair actions that resolve drift without manual file hunting.

## Scope

- In scope: unified artifact status model, UI surface, and repair actions.
- Out of scope: remote/cloud telemetry.

## Requirements

### Functional

- Define common status states across artifacts: `synced`, `out_of_date`, `missing`, `conflicted`, `unsupported`.
- Add unified Artifact Status view with filtering by artifact/tool/scope/repo.
- Provide one-click repair actions: sync, cleanup, resolve-conflict, reconcile.
- Include path previews and latest operation outcome details per artifact entry.

### Non-Functional

- Status refresh should be incremental to avoid blocking large repositories.
- UI should clearly differentiate detection errors from actual drift.

## Acceptance Criteria

- [ ] Unified status view includes all artifact types and adapters.
- [ ] All status entries include actionable repair affordances.
- [ ] Repair actions update status state immediately and produce user-visible results.
- [ ] Status logic reuses reconciliation engine outputs to avoid duplicated truth sources.

## Dependencies

- Depends on canonical registry, path resolver, and reconciliation engine.
