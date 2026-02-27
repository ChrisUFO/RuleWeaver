# Objective

Harden and polish the drift detection feature.

# Key Files

- `src-tauri/src/mcp/watcher.rs`
- `src-tauri/src/mcp/mod.rs`
- `src/components/pages/Skills.tsx`
- `src/components/commands/CommandList.tsx`
- `src/hooks/useCommandsState.ts`

# Steps

### Phase 1: Backend Hardening (`src-tauri`)

1. **Enhance `WatcherManager` (`watcher.rs`)**:
   - Implement path normalization using `std::fs::canonicalize` to ensure cross-platform consistency.
   - Add an ignore filter for common high-traffic directories (`.git`, `node_modules`, `target`, `.agents`).
   - Improve error handling to skip invalid/unauthorized directories rather than failing the entire watcher.
2. **Dynamic Watcher & Events (`mod.rs`)**:
   - Extract path collection logic into a reusable `collect_watch_paths` method.
   - Update `refresh_commands` to re-sync the active watchers whenever tool data is refreshed.
   - Emit a Tauri event `mcp-artifacts-refreshed` upon successful refresh to notify the frontend.

### Phase 2: Frontend Polish (`src`)

1. **Instant Feedback**:
   - Implement listeners for the `mcp-artifacts-refreshed` event in `useCommandsState.ts` and `Skills.tsx`.
   - Trigger immediate data re-fetching when the event is received, removing the 5s polling delay.
2. **Visual Excellence**:
   - Add a "Just Refreshed" animation/state to the `Eye` icon to confirm a background sync occurred.
   - Enhance the `Eye` icon tooltip to display the specific path being monitored.
