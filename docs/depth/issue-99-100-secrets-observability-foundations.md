## Problem

Milestone 17 needs to finish two trust gaps: secret values still persist in plaintext app storage, and observability is still split across execution history plus in-memory MCP logs.

## Baseline Audit

- `architecture.md` is broadly accurate on layering and scope boundaries.
- `src-tauri/src/secrets/mod.rs` already provides scoped-secret precedence and runtime resolution.
- `src-tauri/src/database/mod.rs` still stores `scoped_secrets.value` in plaintext.
- `src/components/settings/ScopedSecretsCard.tsx` already enforces masked/re-entry UX at the UI layer.
- `src-tauri/src/mcp/mod.rs` has recent MCP logs, but they are in-memory strings rather than a structured persisted event stream.
- `src-tauri/src/database/mod.rs` already persists command execution history in `execution_logs`.
- `src/components/pages/Settings.tsx` exposes MCP status/log snippets and config export/import, but there is no dedicated logs page.

## Constraints

- Reuse repository-root and path-resolution seams for workspace-aware secret behavior.
- Keep raw secret values out of logs, exports, generated files, and synced artifacts.
- Extend the existing execution/MCP plumbing instead of creating a second observability subsystem.
- Keep docs truthful about migration, export scope, and redaction limitations.

## Verified Change Map

### Secrets (`#99`)

- `src-tauri/src/database/mod.rs`
  - replace plaintext secret persistence with metadata + secure-storage reference
  - add migration for existing plaintext secrets
- `src-tauri/src/models/secret.rs`
  - update secret models for metadata/reference-only persistence
- `src-tauri/src/secrets/mod.rs`
  - resolve raw values from secure storage only at runtime
- `src-tauri/src/mcp/mod.rs`
  - keep command/skill runtime injection using resolved values only
- `src-tauri/src/commands/migration_commands.rs`
  - ensure config export/import never leaks secret values
- `src/hooks/useSettingsState.ts`
- `src/components/settings/ScopedSecretsCard.tsx`
  - preserve masked replace-only UX while reflecting secure-storage behavior

### Observability (`#100`)

- `src-tauri/src/database/mod.rs`
  - add persisted structured observability event storage and filtering queries
- `src-tauri/src/models/command.rs`
  - add shared event types for MCP lifecycle and execution observability
- `src-tauri/src/mcp/mod.rs`
  - emit structured MCP lifecycle and tool-execution events
- `src-tauri/src/commands/system_commands.rs`
- `src-tauri/src/commands/mcp_commands.rs`
  - expose filtered queries and export commands
- `src/lib/tauri.ts`
- `src/types/command.ts`
  - add frontend API/types for observability events and filters
- `src/App.tsx`
- `src/components/layout/Sidebar.tsx`
  - add dedicated logs navigation
- `src/components/pages`
- `src/hooks`
  - add a full-screen logs surface and supporting state hook

## Phase Ordering

1. Baseline audit and branch setup
2. Secure storage metadata + migration
3. Runtime secret injection hardening + secret-safe UX/export behavior
4. Structured observability event layer
5. Dedicated logs dashboard, filters, and export
6. Documentation refresh and full verification