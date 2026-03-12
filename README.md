# RuleWeaver

RuleWeaver is a unified, standalone desktop application designed to centrally manage configurations, rules, commands, and skills for various AI coding assistants (Antigravity, Gemini CLI, OpenCode, Cline, Claude Code, Codex, Kilo Code, Cursor, Windsurf, Roo Code).

Managing different file formats and local/global settings across 10+ AI tools is a nightmare. RuleWeaver solves this by acting as a single source of truth using a **Hybrid Synchronization Model**.

## The Hybrid Approach

Different types of AI configurations require different management strategies:

1. **Rules (Static Context):** Managed via **File Sync**. You write your global or repo-specific rules in the RuleWeaver UI. The app then uses **Tool-Specific Adapters (Post-Processors)** to automatically translate and copy these rules into the specific proprietary formats and directories required by each target tool (e.g., configuring `.clinerules` for Cline, `.cursorrules` for Cursor, or `AGENTS.md` for OpenCode).
2. **Commands & Skills (Executable Actions):** Managed via a **Local MCP Server** combined with **UI Stub Syncing**. RuleWeaver supports two MCP runtime modes:
   - **Embedded mode:** MCP runs inside the desktop app process.
   - **Standalone mode:** MCP runs as a separate binary (`ruleweaver-mcp --port 8080`).

   RuleWeaver generates the `.md`/`.toml` command stubs for tool UX, while command execution happens through MCP.

## Features

- **Standalone GUI:** A fast, native desktop application (built with Tauri).
- **Scope Management:** Clearly define if a Configuration is "Global" (applied everywhere) or "Local" (applied only when the AI is operating within specific defined repository paths).
- **Workspace-Scoped Secrets:** Store shared credentials globally, override them per repository root, and reuse the resolved values in command tests, MCP tools, and skills without leaking raw secrets into logs. Values are kept in OS secure storage and never included in configuration exports.
- **Dual MCP Runtime:** Embedded MCP in app process or standalone `ruleweaver-mcp` process.
- **MCP Trust Surface:** Settings now exposes readiness/degraded/error status, actionable diagnostics, endpoint/token copy actions, and ready-to-paste Claude Code / OpenCode snippets.
- **Observability Logs:** A dedicated Logs page captures MCP lifecycle/client activity plus command and skill runs, with filters and redaction-safe JSON export.
- **Command Manager:** CRUD commands, test runs, MCP exposure toggles, and execution history.
- **Command Stub Sync:** Generates command files for supported tools (`COMMANDS.toml` / `COMMANDS.md`).
- **Native Slash Commands:** Generate native `/commandname` triggers for 8 AI tools with automatic file generation and incremental sync.
- **Background Keep-Alive:** Optional close-to-tray behavior keeps MCP available.
- **Native Skills Distribution:** Skills are synced as `SKILL.md` files directly into each AI tool's skill directory (Claude Code, OpenCode, Cline, Gemini, Roo Code, Windsurf, Antigravity, Codex). Per-skill adapter targeting lets you control which tools receive each skill. Global and local scope both supported.
- **Unified Status & Repair:** Single operator view across all artifact types (rules, commands, skills). Filter by adapter, artifact type, or sync status. One-click repair re-syncs drifted or missing artifacts.
- **Priority Tiering:**
  1. Rules First (System Prompts, Code Standards)
  2. Custom Commands Second (Single scripts, quick actions)
  3. Skills Third (Complex, multi-file execution workflows)

## Security Note

- Scoped secret values are masked in the UI and API responses and stored in the OS credential manager / secure keychain instead of plaintext app persistence.
- Configuration export/import transfers rules, commands, and skills metadata only. Secret values are never exported or imported, so re-enter them locally on each machine.
- Logs and exports only retain redacted execution excerpts and metadata; masked secrets are never re-materialized in observability output.

## Getting Started

_(Installation instructions will be added as the MVP is developed)_

## Development

## User Documentation

- See `USER_GUIDE.md` for:
  - rules and skills management
  - MCP setup and runtime modes
  - workspace-scoped secret management
  - logs filtering and export workflow
  - agent connection guidance

### Prerequisites

- [Node.js](https://nodejs.org/) (v20+)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/setup/vite/)

### Setup

```bash
npm install
npm run tauri:dev
```

### MCP Runtime Modes

- **Embedded MCP:** Start RuleWeaver desktop app and use Settings -> MCP Server controls.
- **Logs & Export:** Open the Logs screen to filter MCP lifecycle/client events plus command and skill runs, then export the current filtered view or selected entries as JSON.
- **Standalone MCP:** Build and run:

```bash
cargo run --manifest-path src-tauri/Cargo.toml --bin ruleweaver-mcp -- --port 8080
```

Use the connection snippets shown in Settings to configure Claude Code/OpenCode.

The MCP Settings card also shows endpoint + token copy actions, recent logs, and diagnostics for common failure modes such as port conflicts, missing exposed tools, and stale client configuration.

If **Minimize to tray on close** is enabled (Settings -> MCP Server), closing the window keeps RuleWeaver and embedded MCP running in the background.

### Build Scripts

| Script                                 | Description                               |
| -------------------------------------- | ----------------------------------------- |
| `./build` or `./build.bat`             | Full production build (lint, test, build) |
| `./build-quick` or `./build-quick.bat` | Quick build (skips linting and tests)     |
| `./dev` or `./dev.bat`                 | Start development server                  |

### NPM Scripts

- `npm run dev` - Start Vite dev server
- `npm run tauri:dev` - Start Tauri in development mode
- `npm run build` - Build frontend for production
- `npm run tauri:build` - Build Tauri app for production
- `npm run lint` - Run ESLint
- `npm run lint:rust` - Run Rust clippy
- `npm run typecheck` - Run TypeScript type checking
- `npm run test` - Run frontend tests
- `npm run test:rust` - Run Rust tests
- `npm run gen:docs` - Regenerate `docs/SUPPORT_MATRIX.md` from the canonical tool registry

### Pre-push Troubleshooting

- Hooks are path-aware:
  - `pre-commit` runs only fast staged-file checks
  - `pre-push` runs heavier validation only for changed frontend / Rust / support-matrix areas
  - frontend pushes use changed-file ESLint plus direct matching test-file selection when safe, and fall back to the full frontend suite when a changed source file has no obvious matching test
  - Rust pushes run targeted integration-test validation when only `src-tauri/tests/*.rs` changed
  - a small allowlist of self-contained Rust source modules (`feature_flags`, `redaction`, `single_instance`, `status`) uses `cargo clippy --lib --bins` plus filtered `cargo test --lib <module>::`
  - other Rust source changes still fall back to the full Rust suite
- Git runs these hooks as shell scripts, so normal `git commit` / `git push` works from
  both Git Bash and PowerShell. On Windows, the hook files are pinned to LF endings so
  Bash execution stays reliable even with `core.autocrlf=true`.
- Pre-push output is logged to `.git/hooks-logs/pre-push.log` by default.
- If a step fails, the hook prints the failing command and exit code in the final lines.
- You can override the log path for one run: `HOOK_LOG_PATH=/tmp/ruleweaver-pre-push.log git push`.
- You can dry-run hook routing without executing commands:
  - `HOOK_DRY_RUN=1 HOOK_PUSH_FILES_OVERRIDE=$'src/App.tsx' bash .husky/pre-push </dev/null`
  - `HOOK_DRY_RUN=1 HOOK_STAGED_FILES_OVERRIDE=$'src-tauri/src/lib.rs' bash .husky/pre-commit`
- PowerShell equivalents:
  - `$env:HOOK_DRY_RUN='1'; $env:HOOK_PUSH_FILES_OVERRIDE="src/App.tsx"; bash .husky/pre-push`
  - `$env:HOOK_DRY_RUN='1'; $env:HOOK_STAGED_FILES_OVERRIDE="src-tauri/src/lib.rs"; bash .husky/pre-commit`
- To inspect recent output quickly: `tail -n 200 .git/hooks-logs/pre-push.log`.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
