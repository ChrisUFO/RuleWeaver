# System Architecture

RuleWeaver is designed as a standalone desktop application. It requires deep filesystem access to sync tool configurations globally and locally, as well as network capabilities to host a local server.

## Documentation

- **AI Tools Reference:** See [`docs/ai-tools-commands-reference.md`](./docs/ai-tools-commands-reference.md) for comprehensive documentation on how each supported AI tool handles rules, custom commands, and skills.
- **Implementation Plan:** [`PLAN.md`](./PLAN.md) is the temporary local milestone plan when present; treat this architecture document and the user-facing docs as the lasting product source of truth.

## Versioning Strategy

RuleWeaver uses an **auto-incrementing timestamp-based versioning scheme** to avoid formal semantic versioning during rapid development phases.

**Format:** `MAJOR.MINOR.PATCH-DDMM`

- **Example:** `0.0.1-232` (first build on Feb 23)
- **Note:** DD and MM are not zero-padded to comply with semver pre-release rules (leading zeros not allowed)

**Version Components:**

- `MAJOR.MINOR.PATCH`: Auto-incremented on each build (e.g., 0.0.1, 0.0.2, ...)
- `DDMM`: Day and month as prerelease identifier (no leading zeros, max 3112, fits MSI bundler limit of 65535)
- **Rollover:** When PATCH reaches 255, it resets to 0 and increments MINOR (0.0.255 → 0.1.0). Same for MINOR → MAJOR.

**Why this format?**

- Valid semver compatible with Tauri bundler
- MSI bundler compatible (prerelease ≤ 65535)
- Windows VERSIONINFO compatible (all components ≤ 255)
- Shows version progression and build date
- No manual version management needed during development

**Build Artifacts:**

Installers include the full timestamp in the filename (e.g., `ruleweaver_0.0.1_2602231155.exe`), allowing precise build identification while keeping the version string MSI-compatible.

**Build Scripts:**

- `./build` (Unix/macOS) and `./build.bat` (Windows) automatically:
  1. Parse current version from `package.json`
  2. Increment PATCH (with rollover logic)
  3. Generate DDMM prerelease and full YYMMDDHHMM timestamp
  4. Update all version files (`package.json`, `Cargo.toml`, `tauri.conf.json`)

## Tech Stack

- **Framework:** [Tauri](https://tauri.app/) (desktop shell + native Rust backend).
- **Frontend:** React, TypeScript, TailwindCSS.
- **Backend:** Rust.
- **Persistence:** File-first markdown + YAML frontmatter (`~/.ruleweaver/rules/*.md`, `{repo}/.ruleweaver/rules/*.md`) with SQLite as index/cache and settings store.

## High-Level Architecture

The system is composed of three main layers:

### 1. The Presentation Layer (Frontend)

The React/TypeScript application running in the Tauri webview.

- **State Management:** Holds the UI state for editing Rules, Commands, Skills, and operator diagnostics such as the Logs screen.
- **Communication:** Communicates with the Rust backend via Tauri IPC (Inter-Process Communication) to save rules, trigger syncs, and query/export observability events.

### 2. The Core Logic Layer (Rust Backend)

This layer handles all OS-level operations.

- **Database Manager:** Stores indexed metadata, command definitions, execution logs, structured observability events, settings, scoped secret metadata, and sync tracking data. Raw secret values remain in OS secure storage and are referenced by opaque keys.
  - **Tables:**
    - `rules`, `commands`, `skills` — Core artifact storage
    - `sync_manifest` — Tracks all files written by RuleWeaver with content hashes for drift detection and clean uninstall
    - `tool_sync_preferences` — Per-tool global sync toggles (rules/commands/skills) for granular control
    - `settings` — Key-value store for app preferences including `reconciliation_mode`
    - `secrets` — Scoped secret metadata (values in OS secure storage)
    - `reconciliation_logs` — Audit trail of all sync operations
- **File Storage Engine:** Reads/writes rule markdown files with YAML frontmatter, supports migration/rollback, and handles local+global rule roots.
- **File Sync Engine (The "Adapters"):**
  - Because every AI tool expects a different filename (`GEMINI.md`, `AGENTS.md`, `.clinerules`) or specific frontmatter, the Sync Engine acts as a collection of **Tool-Specific Adapters (Post-Processors)**.
  - Rule synchronization supports two models per adapter/scope: `single_file` (all rules aggregated into one file) and `per_rule_dir` (one `<slug>.md` file per rule inside a rules directory).
  - When a sync is triggered, the engine takes the master Rule and runs it through each active adapter. The adapter handles tool-specific formatting (e.g., prepending XML tags for Claude, or formatting TOML headers) and determines the exact target directory based on the "Scope".
  - Writes file outputs directly to the filesystem.
- **Rule Import Engine (Bidirectional Sync):**
  - Scans existing AI tool rule locations and external sources.
  - Supports import sources: AI tool directories, single files, directories, URLs, and clipboard text.
  - Normalizes imported content, applies duplicate detection and conflict policy (`skip`, `rename`, `replace`), and stores import history/source mapping.
  - Writes imported rules to DB and file storage mode (if enabled), then runs sync to keep generated tool files current.
- **Command Sync Engine:**
  - Generates native slash command files (`.md`/`.toml`) directly into each AI tool's command directory.
  - Commands are executed natively by AI tools without a separate server.
  - Supports adapter-specific formatting for each tool's command syntax.
- **Skills Engine:**
  - Stores and manages Skills metadata/instructions in database.
  - Exposes full CRUD in UI and participates in reconciliation/status projections.
  - Supports adapter-targeted native skill distribution via Skills Sync.
- **Reconciliation Engine:**
  - Computes desired state from all database artifacts (rules, commands, skills).
  - Scans actual filesystem state across all adapter paths.
  - Detects and removes orphaned/stale artifacts when items are deleted, disabled, or retargeted.
  - Runs automatically after mutations (create/update/delete) and imports.
  - Supports dry-run preview mode for safe inspection before changes.
  - Logs all operations to `reconciliation_logs` table for audit trail.
  - Uses atomic writes (temp file + rename) to prevent partial state corruption.
- **Skills Sync Engine:**
  - Extends the adapter pattern to native skill distribution.
  - Each skill can target specific adapters (or all supported adapters by default).
  - Generates SKILL.md files directly in AI tool skill directories (e.g., `~/Documents/Cline/Skills/`).
  - Respects adapter capabilities — skips unsupported tools silently.
- **Status Engine (Unified Artifact Status):**
  - Single operator view across all artifact types (rules, commands, skills).
  - Projects reconciliation state into status entries with sync health indicators.
  - Supports filtering by adapter, artifact type, scope, and sync status.
  - Provides one-click repair actions to re-sync drifted or missing artifacts.
  - Identifies conflicts and orphaned files without duplicate truth sources.
- **Execution Engine:**
  - Centralized command execution with timeout and retry policies.
  - Per-command configuration for `timeout_ms` and `max_retries` (bounded to 3).
  - Resolves secrets with precedence across global, workspace, and artifact-specific scopes before command execution.
  - Secret redaction pipeline scans stdout/stderr for API keys, tokens, and credentials.
  - Structured failure classification (`Timeout`, `PermissionDenied`, `MissingBinary`, `NonZeroExit`).
  - Emits redaction-safe observability records for command and skill runs so the Logs page can filter and export operational history without exposing raw secrets.
  - Only retryable failures (transient errors) trigger retries; validation/binary errors fail fast.
  - Logs all execution attempts with metadata (failure class, redaction flag, attempt number).
  - **AI Integration Engine:**
    - Provides AI-assisted rule improvement and generation capabilities.
    - Supports multiple providers: OpenAI, Anthropic, Google AI Studio, OpenRouter, DeepSeek, Together AI, MiniMax, Z.ai, and custom OpenAI-compatible endpoints.
    - API keys stored securely using OS-native credential storage (keyring on Windows/macOS, secret-service on Linux).
    - Provider-aware client routing: Anthropic uses native `/messages` endpoint, others use OpenAI-compatible `/chat/completions`.
    - Automatic model discovery for providers that support `/models` endpoint; static model lists for Anthropic.
    - Exponential backoff retry logic (up to 3 retries) for transient errors (rate limits, network issues, timeouts).
    - Configurable custom prompts for improvement and generation, with sensible defaults.
    - User-friendly error mapping translates technical errors into actionable guidance.

### 3. The Target Layer (The AI Tools)

- **File Watchers:** AI tools (like Cline, OpenCode) naturally watch for changes in their rule files. When the Sync Engine updates a file, the AI tool seamlessly picks it up.
- **Manual Import UX:** The Rules page exposes import actions for AI tools, files, folders, URLs, and clipboard content, with result summaries.
  - Includes drag-and-drop file import and import-time option overrides (scope/adapters/conflict mode).
- **Repository Roots Registry:** Settings persist a managed list of local repository roots (`local_rule_paths`) used for local artifact selection and local import discovery.
  - Rules, commands, skills, and workspace-scoped secrets reference these configured roots instead of free-form path entry.

## Runtime Topology

```text
AI Tool -> reads rule/command/skill files -> RuleWeaver generates and syncs files

Window Lifecycle:
- With `minimize_to_tray = true`, close requests hide the window and keep the app process alive.
- System tray menu controls show/hide and quit behavior.
```

## Data Model (Conceptual)

```json
{
  "rules": [
    {
      "id": "123",
      "name": "General Tech Stack",
      "content": "Always use TypeScript.",
      "scope": "GLOBAL"
    },
    {
      "id": "456",
      "name": "Monorepo Standards",
      "content": "Use turborepo caching.",
      "scope": "LOCAL",
      "target_paths": ["C:/Users/chris/AgentManager"]
    }
  ],
  "commands": [
    {
      "id": "789",
      "name": "Format Code",
      "script": "npm run format",
      "generate_slash_commands": true
    }
  ],
  "skills": [
    {
      "id": "abc",
      "name": "Lint and Fix",
      "description": "Run lint and auto-fix pipeline",
      "instructions": "Run npm run lint then npm run format",
      "enabled": true
    }
  ]
}
```
