## Problem

Skills are managed and executable, but cross-tool native skills distribution is not yet lifecycle-complete and capability-aware.

## User Stories

- As a user, I want skills delivered natively to tools that support skills while preserving MCP fallback for unsupported tools.
- As an operator, I want per-skill targeting and scope controls consistent with rules/commands behavior.

## Scope

- In scope: native skills sync adapters, targeting controls, validation, and lifecycle reconciliation hooks.
- Out of scope: creating a new skills language or format.

## Requirements

### Functional

- Add native skill adapters for tools with skills support.
- Support per-skill adapter targeting and local/global scope-aware path controls.
- Enforce capability-aware fallback behavior (MCP-only where native skills unsupported).
- Include skills in reconciliation and status pipelines.

### Non-Functional

- Ensure frontend/backend schema parity for skill metadata serialization.
- Keep skills sync deterministic and idempotent.

## Acceptance Criteria

- [ ] Native skill files are generated for supported tools according to registry paths/formats.
- [ ] Per-skill adapter targeting works in UI and backend APIs.
- [ ] Unsupported tools are explicitly marked and handled with MCP fallback behavior.
- [ ] Skills participate in unified status and reconciliation flows.
- [ ] Serialization parity tests pass for frontend/backend skill schema.

## Dependencies

- Depends on canonical registry and shared path resolution.
