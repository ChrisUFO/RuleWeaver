## Problem

Conflict detection/resolution currently has edge cases (including hash-read race windows) that can produce low-confidence previews and unclear outcomes.

## User Stories

- As a user, I want sync previews and conflict results to be deterministic so I can confidently choose overwrite vs keep-remote.
- As an operator, I want conflict diagnostics that clearly explain what changed and why a conflict was raised.

## Scope

- In scope: conflict detection consistency, race mitigation, improved diffing and resolution diagnostics.
- Out of scope: version-control-style full merge system.

## Requirements

### Functional

- Eliminate or minimize hash read/compute race windows in preview and sync paths.
- Ensure conflict checks compare against the exact content basis used for planned writes.
- Improve conflict diff quality (line mapping stability, adapter-aware local content composition).
- Emit structured conflict events with artifact, adapter, scope, path, and hash metadata.

### Non-Functional

- Preview path should remain responsive for larger artifact sets.
- Conflict resolution should be safely retryable and idempotent.

## Acceptance Criteria

- [ ] Preview and sync conflict outcomes are deterministic across repeated runs without file changes.
- [ ] Race condition in hash comparison path is addressed with validated strategy.
- [ ] Conflict UI receives structured metadata and clearer diagnostics.
- [ ] Automated tests cover conflict detection/resolution edge cases.

## Dependencies

- Relies on shared resolver and reconciliation contracts.
