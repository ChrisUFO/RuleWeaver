## Problem

Command execution exists (test run + history), but reliability/safety/diagnostic depth is still limited for production-like operator workflows.

## User Stories

- As a user, I want reliable command runs with clear failure diagnostics so I can trust automation outcomes.
- As an operator, I want execution visibility with safe handling of sensitive data.

## Scope

- In scope: command runtime hardening, structured logs/diagnostics, safety controls.
- Out of scope: remote hosted execution.

## Requirements

### Functional

- Add structured execution events with context: command id/name, adapter context, arguments, duration, exit code, triggered-by.
- Introduce retry/timeout policy controls per command (bounded, opt-in).
- Add output redaction pipeline for known secret patterns before persistence/display.
- Classify common failure modes (validation, timeout, permission, missing binary, non-zero exit).
- Improve execution history UX for filtering and triage.

### Non-Functional

- Logging overhead should not materially degrade execution performance.
- Redaction should be deterministic and test-covered.

## Acceptance Criteria

- [ ] Execution logs include structured, queryable context for diagnosis.
- [ ] Timeout/retry behavior is configurable and respected.
- [ ] Sensitive output is redacted in persisted/displayed logs.
- [ ] Failure classes are surfaced in UI and API responses.
- [ ] Tests cover timeout, retry, redaction, and failure classification.

## Dependencies

- Integrates with unified status/observability direction.
