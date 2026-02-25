## Problem

Rules import is robust, but command/workflow and skills import are not yet at the same lifecycle maturity. This reduces migration value and causes inconsistent onboarding experiences.

## User Stories

- As a migrating user, I want the same scan/preview/conflict-resolution workflow for commands and skills that already exists for rules.
- As an operator, I want imported commands and skills to reconcile immediately so no stale files remain.

## Scope

- In scope: full import lifecycle parity for commands/workflows and skills.
- Out of scope: importing entirely new artifact categories.

## Requirements

### Functional

- Implement command/workflow import pipeline:
  - global+local discovery by configured roots
  - scan preview with selection
  - conflict modes (`rename`, `skip`, `replace`)
- Implement skills import pipeline:
  - metadata + instructions extraction and validation
  - idempotent re-import behavior
  - schema/entrypoint safety checks
- Trigger post-import reconciliation for all affected artifacts.
- Persist import source mapping/history consistently with rules import.

### Non-Functional

- Import must enforce existing safety limits (size, path, URL constraints where applicable).
- Import operations should produce clear summary and error reporting.

## Acceptance Criteria

- [ ] Commands/workflows support scan + preview + execute import with conflict handling.
- [ ] Skills support scan + preview + execute import with conflict handling.
- [ ] Post-import reconciliation runs and removes stale artifacts.
- [ ] Import history/source mapping is available for commands and skills.
- [ ] End-to-end migration tests cover at least 3 major tools with local+global scenarios.

## Dependencies

- Depends on canonical registry and path resolver standardization.
