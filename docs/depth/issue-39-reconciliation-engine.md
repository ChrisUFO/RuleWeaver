## Problem

RuleWeaver currently focuses on write-time sync but lacks a complete post-change reconciliation engine across all artifacts. Rename/delete/deselect/import changes can leave stale generated files behind.

## User Stories

- As a user, I want RuleWeaver to clean up stale artifacts automatically when I rename, delete, or retarget items.
- As an operator, I want deterministic post-mutation state so tools never run outdated definitions.

## Scope

- In scope: reconciliation for rules, command stubs, slash commands, and skills.
- Out of scope: external cloud sync.

## Requirements

### Functional

- Build a desired-state model per artifact+adapter+scope and compare against filesystem actual state.
- Reconcile on CRUD mutations, adapter target changes, and import/export operations.
- Support actions: create/update/cleanup with explicit operation logging.
- Include dry-run mode for preview UI and diagnostics.

### Non-Functional

- Reconciliation must be idempotent and safe to retry.
- Reconciliation errors should not leave partial state unreported.

## Acceptance Criteria

- [ ] Rename/delete/deselect operations remove stale generated artifacts deterministically.
- [ ] Reconciliation runs after artifact mutations and import/export flows.
- [ ] Dry-run output is available and matches actual reconcile execution.
- [ ] Audit logs capture created/updated/removed artifacts with adapter/scope context.

## Dependencies

- Requires canonical registry and shared path resolver.
- Enables reliable unified status and one-click repair actions.
