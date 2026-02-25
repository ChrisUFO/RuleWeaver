## Problem

RuleWeaver currently duplicates tool capability and path metadata across backend logic, frontend constants, and documentation. This creates behavior drift, inconsistent sync/import support, and avoidable regressions when adding or changing adapters.

## User Stories

- As an operator, I want RuleWeaver to behave consistently across rules, commands, slash commands, and skills regardless of which tool adapter I choose.
- As a maintainer, I want one canonical source of tool capabilities so I can safely update adapters without breaking docs/UI/runtime parity.

## Scope

- In scope: canonical registry model, runtime consumption in sync/import/reconcile paths, frontend metadata generation/consumption, docs export hooks.
- Out of scope: adding brand-new AI tools.

## Requirements

### Functional

- Define a backend canonical registry for each tool covering:
  - artifact support (`rules`, `command_stubs`, `slash_commands`, `skills`)
  - scope support (`global`, `local`)
  - import support by artifact
  - path templates for global/local targets
  - format metadata (e.g., markdown/toml/frontmatter requirements)
- Replace hardcoded adapter capability/path constants in backend flows with registry lookups.
- Expose a typed registry payload for frontend consumption to remove duplicated metadata.
- Add startup/runtime validation that rejects unsupported artifact+tool combinations.

### Non-Functional

- Registry lookups should be deterministic and cheap (no network dependency).
- Changes to registry schema require test updates and migration notes.

## Implementation Notes

- Backend: introduce `tool_registry` module and migrate existing adapter/path resolution call sites.
- Frontend: consume generated/shared registry model instead of static capability constants.
- Documentation: add generation path for support matrix from registry payload.

## Acceptance Criteria

- [ ] One canonical registry drives backend behavior, frontend capability display, and support matrix docs.
- [ ] No duplicated hardcoded capability tables remain in frontend/backend.
- [ ] Unsupported combinations are blocked with actionable validation errors.
- [ ] Regression tests verify registry coherence and path template validity.

## Dependencies

- Enables reliable implementation of reconciliation, import parity, status UX, and docs truthfulness work.
