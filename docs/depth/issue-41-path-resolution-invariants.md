## Problem

Path resolution behavior differs by artifact and adapter, especially for local targets. This causes mismatches between preview vs write paths, stale artifacts, and confusing cross-repo behavior.

## User Stories

- As a user, I want local/global targeting to resolve identically for rules, commands, slash commands, and skills.
- As an operator, I want predictable path previews so I can trust what will be written or cleaned.

## Scope

- In scope: adapter-specific local path resolution standardization across all artifact types.
- Out of scope: introducing new scope models beyond global/local.

## Requirements

### Functional

- Implement one shared path resolution service driven by canonical registry templates.
- Apply same resolver for preview, sync, cleanup, and reconciliation flows.
- Enforce repository-root constraints for local targets for all artifact types.
- Normalize separators/canonicalization for Windows and Unix paths.

### Non-Functional

- Resolver should be pure and testable with fixture-based matrix tests.
- Error messages must identify artifact, adapter, scope, and path that failed.

## Acceptance Criteria

- [ ] Preview and actual write paths always match for rules/commands/slash/skills.
- [ ] Local target validation is consistent across all artifact editors and APIs.
- [ ] Tests cover full adapter x artifact x scope matrix with platform-specific path cases.
- [ ] No stale files remain from resolver mismatch after migration.

## Dependencies

- Depends on canonical registry work.
- Feeds reconciliation and status/repair UX accuracy.
