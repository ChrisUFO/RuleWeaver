# RuleWeaver Roadmap

This roadmap defines the path to the Minimum Viable Product (MVP), structured around the requested priority: Rules > Commands > Skills.

## Phase 1: Foundation & "Rules" MVP (High Priority)
*Goal: Establish the GUI, database, and the File Sync Engine to manage static context.*
- [ ] Initialize Tauri (Rust + React/TypeScript) project.
- [ ] Set up local embedded database (SQLite or JSON store) for configuration state.
- [ ] **UI:** Build the main dashboard, Scope Selection (Global vs. Local repo paths), and a Markdown editor.
- [ ] **Feature:** "Rules Manager". Create, edit, and delete rules.
- [ ] **Core Engine:** "Sync Engine". Detect changes in the database and run rules through **Tool-Specific Adapters/Post-Processors** to automatically transpile and copy rule files to the correct target directories on the host operating system (e.g., `.clinerules`, `AGENTS.md`, `~/.gemini/GEMINI.md`).

## Phase 2: "Custom Commands" MVP & MCP Integration (Medium Priority)
*Goal: Introduce the built-in MCP server to serve simple executable commands.*
- [ ] **Core Engine:** Integrate a lightweight MCP Server running in the Rust backend of the Tauri app.
- [ ] **UI:** Build the "Commands Manager". Interface to define shell commands, scripts, and their required arguments.
- [ ] **Feature:** Map defined Custom Commands to dynamically generated MCP `tools`.
- [ ] Document instructions on how to connect the target AI tools (Claude Code, OpenCode, etc.) to the local RuleWeaver MCP port.

## Phase 3: "Skills" MVP (Lower Priority)
*Goal: Support complex, multi-file execution environments (Agent Skills).*
- [ ] **UI:** Build the "Skills Manager". UI to manage bundled workflows (directories with a `SKILL.md` and auxiliary scripts).
- [ ] **Feature:** Expand the MCP Server to expose complex "Skills" as tools that return structured output or execute multi-stage python/bash scripts securely.
- [ ] Provide templates for common skills (e.g., "Run Linter and auto-fix", "Fetch Jira Ticket").

## Phase 4: Polish & Extensibility
*Goal: Harden the application for daily use.*
- [ ] Add systemic tray icon to run in background.
- [ ] Conflict resolution (if a tool overwrites a synced file manually).
- [ ] Support for importing/exporting configurations to share with teams.

## Phase 5: Advanced Ecosystem (Post-MVP)
*Goal: Make the central repository a true power-user control center.*
- [ ] **Secrets & Vault Management:** Integrating a secure key-store so "Skills" (like creating a GitHub issue or fetching a Jira ticket) can access API tokens securely without hardcoding them in the sync rules.
- [ ] **Live Execution Logging:** A "Logs" dashboard in the GUI that tracks every single connection to the MCP Server. You can audit exactly *which* AI agent ran *what* command and when, and read the `stdout` they were provided.
- [ ] **Community Hub / Registry:** Support for pasting a GitHub URL to instantly import community-created rule sets (e.g., "Google Chrome Official Extension Developer Pattern") or advanced Skills directly into your database.
