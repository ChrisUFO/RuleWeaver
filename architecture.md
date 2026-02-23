# System Architecture

RuleWeaver is designed as a standalone desktop application. It requires deep filesystem access to sync tool configurations globally and locally, as well as network capabilities to host a local server.

## Versioning Strategy

RuleWeaver uses an **auto-incrementing timestamp-based versioning scheme** to avoid formal semantic versioning during rapid development phases.

**Format:** `MAJOR.MINOR.PATCH-DDMM`

- **Example:** `0.0.1-2302` (first build on Feb 23)

**Version Components:**

- `MAJOR.MINOR.PATCH`: Auto-incremented on each build (e.g., 0.0.1, 0.0.2, ...)
- `DDMM`: Day and month as prerelease identifier (max 3112, fits MSI bundler limit of 65535)
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
- **MCP Runtime:** Embedded MCP manager in app process and standalone MCP binary (`ruleweaver-mcp`).

## High-Level Architecture

The system is composed of three main layers:

### 1. The Presentation Layer (Frontend)

The React/TypeScript application running in the Tauri webview.

- **State Management:** Holds the UI state for editing Rules, Commands, and Skills.
- **Communication:** Communicates with the Rust backend via Tauri IPC (Inter-Process Communication) to save rules, trigger syncs, and start/stop the MCP server.

### 2. The Core Logic Layer (Rust Backend)

This layer handles all OS-level operations.

- **Database Manager:** Stores indexed metadata, command definitions, execution logs, and settings (including MCP settings and storage mode).
- **File Storage Engine:** Reads/writes rule markdown files with YAML frontmatter, supports migration/rollback, and handles local+global rule roots.
- **File Sync Engine (The "Adapters"):**
  - Because every AI tool expects a different filename (`GEMINI.md`, `AGENTS.md`, `.clinerules`) or specific frontmatter, the Sync Engine acts as a collection of **Tool-Specific Adapters (Post-Processors)**.
  - When a sync is triggered, the engine takes the master Rule and runs it through each active adapter. The adapter handles tool-specific formatting (e.g., prepending XML tags for Claude, or formatting TOML headers) and determines the exact target directory based on the "Scope".
  - Writes file outputs directly to the filesystem.
- **Command Stub Sync Engine:**
  - Generates tool-facing command definition files (`COMMANDS.toml` / `COMMANDS.md`) for supported adapters.
  - Keeps command UX in client tools while execution remains centralized in MCP.
- **MCP Server Engine:**
  - Runs a local MCP-compatible HTTP JSON-RPC server on localhost.
  - Reads commands from database and exposes them as `tools/list`.
  - Handles `tools/call` execution and returns stdout/stderr payloads.
  - Available in two modes:
    - **Embedded mode:** runs inside RuleWeaver desktop app.
    - **Standalone mode:** runs via `ruleweaver-mcp --port <PORT>`.
- **Skills Engine (Phase 3 Foundation):**
  - Stores and manages Skills metadata/instructions in database.
  - Exposes CRUD in UI with MCP execution expansion planned for full Phase 3.

### 3. The Target Layer (The AI Tools)

- **File Watchers:** AI tools (like Cline, OpenCode) naturally watch for changes in their rule files. When the Sync Engine updates a file, the AI tool seamlessly picks it up.
- **MCP Clients:** AI tools (Claude Code, OpenCode, etc.) connect to localhost MCP endpoint or launch standalone `ruleweaver-mcp` binary. They use `tools/list` + `tools/call` to invoke commands.

## Runtime Topology

```text
Option A: Embedded MCP
AI Tool -> localhost:PORT -> RuleWeaver Desktop App (MCP manager)

Option B: Standalone MCP
AI Tool -> launches `ruleweaver-mcp` -> localhost:PORT -> MCP manager

Window Lifecycle:
- With `minimize_to_tray = true`, close requests hide the window and keep the app/MCP process alive.
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
      "expose_via_mcp": true
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
