# RuleWeaver Roadmap

This roadmap defines the path to the Minimum Viable Product (MVP), structured around the requested priority: Rules > Commands > Skills.

## Phase 1: Foundation & "Rules" MVP (High Priority)

_Goal: Establish the GUI, database, and the File Sync Engine to manage static context._

- [ ] Initialize Tauri (Rust + React/TypeScript) project.
- [ ] Set up local embedded database (SQLite or JSON store) for configuration state.
- [ ] **UI:** Build the main dashboard, Scope Selection (Global vs. Local repo paths), and a Markdown editor.
- [ ] **Feature:** "Rules Manager". Create, edit, and delete rules.
- [ ] **Core Engine:** "Sync Engine". Detect changes in the database and run rules through **Tool-Specific Adapters/Post-Processors** to automatically transpile and copy rule files to the correct target directories on the host operating system (e.g., `.clinerules`, `AGENTS.md`, `~/.gemini/GEMINI.md`).

## Phase 2: "Custom Commands" MVP & MCP Integration (Medium Priority)

_Goal: Introduce the built-in MCP server to serve simple executable commands._

- [x] **Core Engine:** Integrate a lightweight MCP Server running in the Rust backend of the Tauri app.
- [x] **UI:** Build the "Commands Manager". Interface to define shell commands, scripts, and their required arguments.
- [x] **Feature:** Map defined Custom Commands to dynamically generated MCP `tools`.
- [x] Document instructions on how to connect the target AI tools (Claude Code, OpenCode, etc.) to the local RuleWeaver MCP port.
- [x] Add standalone MCP binary mode (`ruleweaver-mcp`) for tool-managed process startup.
- [x] Add MCP auto-start setting in desktop app.
- [x] Add command stub sync output generation (`COMMANDS.toml`/`COMMANDS.md`) for supported tools.
- [x] Add in-app command execution history view.

## Phase 3: "Skills" MVP (Lower Priority)

_Goal: Support complex, multi-file execution environments (Agent Skills)._

- [ ] **UI:** Build the "Skills Manager". UI to manage bundled workflows (directories with a `SKILL.md` and auxiliary scripts).
- [ ] **Feature:** Expand the MCP Server to expose complex "Skills" as tools that return structured output or execute multi-stage python/bash scripts securely.
- [ ] Provide templates for common skills (e.g., "Run Linter and auto-fix", "Fetch Jira Ticket").
- [x] **Backend Foundation:** Add Skills data model + CRUD persistence scaffolding.

## Phase 4: Polish & Extensibility

_Goal: Harden the application for daily use._

- [x] Add system tray icon to keep embedded MCP running in background.
- [ ] Conflict resolution (if a tool overwrites a synced file manually).
- [ ] Support for importing/exporting configurations to share with teams.

## Phase 5: Advanced Ecosystem (Post-MVP)

_Goal: Make the central repository a true power-user control center._

- [ ] **Secrets & Vault Management:** Integrating a secure key-store so "Skills" (like creating a GitHub issue or fetching a Jira ticket) can access API tokens securely without hardcoding them in the sync rules.
- [ ] **Live Execution Logging:** A "Logs" dashboard in the GUI that tracks every single connection to the MCP Server. You can audit exactly _which_ AI agent ran _what_ command and when, and read the `stdout` they were provided.
- [ ] **Community Hub / Registry:** Support for pasting a GitHub URL to instantly import community-created rule sets (e.g., "Google Chrome Official Extension Developer Pattern") or advanced Skills directly into your database.
