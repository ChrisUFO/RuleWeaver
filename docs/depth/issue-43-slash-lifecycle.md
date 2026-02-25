## Problem

Slash command support exists, but lifecycle behavior still has gaps around cleanup path correctness, autosync ergonomics, and status visibility/repair.

## User Stories

- As a user, I want slash command files to stay in sync automatically when I save a command.
- As an operator, I want rename/delete/retarget cleanup to remove stale slash files across global and local roots.

## Scope

- In scope: slash lifecycle completeness for sync, status, cleanup, and repair UX.
- Out of scope: support for new slash-capable tools.

## Requirements

### Functional

- Harden cleanup/remove behavior for global and per-root local paths.
- Add autosync-on-save option for slash commands with safe debounce.
- Provide per-adapter slash status states (synced/out-of-date/not-synced/error) in command UI.
- Add one-click repair actions for out-of-date slash artifacts.

### Non-Functional

- Keep sync idempotent and atomic where possible.
- Ensure race-safe execution when multiple slash sync operations are triggered rapidly.

## Acceptance Criteria

- [ ] Cleanup resolves global/home-rooted and repo-rooted local slash paths correctly.
- [ ] Command save can trigger optional autosync and reports outcomes.
- [ ] Per-adapter slash status is visible from command editing experience.
- [ ] Rename/delete/deselect does not leave orphan slash files.
- [ ] Tests cover global/local path combinations and autosync behavior.

## Dependencies

- Depends on shared resolver and reconciliation engine.
