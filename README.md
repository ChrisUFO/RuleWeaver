# RuleWeaver

RuleWeaver is a unified, standalone desktop application designed to centrally manage configurations, rules, commands, and skills for various AI coding assistants (Antigravity, Gemini CLI, OpenCode, Cline, Claude Code, Codex).

Managing different file formats and local/global settings across 6+ AI tools is a nightmare. RuleWeaver solves this by acting as a single source of truth using a **Hybrid Synchronization Model**.

## The Hybrid Approach

Different types of AI configurations require different management strategies:

1. **Rules (Static Context):** Managed via **File Sync**. You write your global or repo-specific rules in the RuleWeaver UI. The app then uses **Tool-Specific Adapters (Post-Processors)** to automatically translate and copy these rules into the specific proprietary formats and directories required by each target tool (e.g., configuring TOML for Gemini CLI, `.clinerules` for Cline, or `AGENTS.md` for OpenCode).
2. **Commands & Skills (Executable Actions):** Managed via a **Local MCP Server** combined with **UI Stub Syncing**. RuleWeaver supports two MCP runtime modes:
   - **Embedded mode:** MCP runs inside the desktop app process.
   - **Standalone mode:** MCP runs as a separate binary (`ruleweaver-mcp --port 8080`).

   RuleWeaver generates the `.md`/`.toml` command stubs for tool UX, while command execution happens through MCP.

## Features

- **Standalone GUI:** A fast, native desktop application (built with Tauri).
- **Scope Management:** Clearly define if a Configuration is "Global" (applied everywhere) or "Local" (applied only when the AI is operating within specific defined repository paths).
- **Dual MCP Runtime:** Embedded MCP in app process or standalone `ruleweaver-mcp` process.
- **Command Manager:** CRUD commands, test runs, MCP exposure toggles, and execution history.
- **Command Stub Sync:** Generates command files for supported tools (`COMMANDS.toml` / `COMMANDS.md`).
- **Background Keep-Alive:** Optional close-to-tray behavior keeps MCP available.
- **Skills Foundation:** Initial Skills CRUD and UI scaffolding is available.
- **Priority Tiering:**
  1. Rules First (System Prompts, Code Standards)
  2. Custom Commands Second (Single scripts, quick actions)
  3. Skills Third (Complex, multi-file execution workflows)

## Getting Started

_(Installation instructions will be added as the MVP is developed)_

## Development

## User Documentation

- See `USER_GUIDE.md` for:
  - rules and skills management
  - MCP setup and runtime modes
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
- **Standalone MCP:** Build and run:

```bash
cargo run --manifest-path src-tauri/Cargo.toml --bin ruleweaver-mcp -- --port 8080
```

Use the connection snippets shown in Settings to configure Claude Code/OpenCode.

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

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
