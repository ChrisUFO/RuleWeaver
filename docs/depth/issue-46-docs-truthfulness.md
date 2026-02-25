## Problem

User-facing and architecture docs can drift from actual runtime behavior, reducing trust and increasing support friction.

## User Stories

- As a user, I want README/guide/reference docs to match what the app actually does.
- As a maintainer, I want automated checks that prevent stale capability/path documentation.

## Scope

- In scope: docs/runtime parity automation, CI checks, generated support matrices.
- Out of scope: broad documentation redesign.

## Requirements

### Functional

- Generate support matrix sections from canonical tool registry data.
- Add CI check that fails when generated docs artifacts are stale.
- Align README, USER_GUIDE, architecture docs with shipped behavior and lifecycle boundaries.
- Document known unsupported combinations explicitly.

### Non-Functional

- Generation workflow should be simple and deterministic.
- CI messages should clearly explain how to regenerate/update docs.

## Acceptance Criteria

- [ ] Docs support matrix is generated from canonical registry source.
- [ ] CI catches docs/runtime divergence before merge.
- [ ] README, USER_GUIDE, architecture docs accurately describe implemented lifecycle behavior.
- [ ] Operator-facing migration notes are published for any breaking path/capability changes.

## Dependencies

- Depends on canonical registry completion.
