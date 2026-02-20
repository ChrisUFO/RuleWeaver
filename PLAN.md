# AigentManager MVP Plan

This document outlines the comprehensive, end-to-end plan to reach a Minimum Viable Product (MVP) for the AigentManager application. The plan is organized by prioritized phases: Foundation/Rules, Commands (MCP), and Skills.

## Phase 1: Foundation and "Rules" Sync Engine (High Priority)
*Objective: Establish the core architecture, GUI, database, and the ability to sync static instruction files (like AGENTS.md, GEMINI.md, .clinerules) globally and locally.*

### Tier 1.1: Project Setup
- [ ] Initialize the Tauri desktop application.
  - Command: `npm create tauri-app@latest` (select React, TypeScript, TailwindCSS).
- [ ] Configure TailwindCSS and integrate a basic UI component library (e.g., shadcn/ui).
- [ ] Setup the Rust backend to support absolute filesystem paths and directory scanning.

### Tier 1.2: Core Data Storage
- [ ] Design the Rust data models for `Rule`, `Scope` (Global vs. Local), and target definitions.
- [ ] Implement a lightweight embedded JSON store (`serde_json`) or SQLite database in the user's local AppData/Config directory to persist rule definitions.
- [ ] Expose Tauri Commands (IPC) to the frontend for CRUD operations on rules.

### Tier 1.3: The GUI
- [ ] Create the Main Dashboard layout (Sidebar for navigation, main content area).
- [ ] Build the "Rules" view.
  - A list interface displaying currently configured rules.
  - A Markdown editor panel to write/edit the rule content.
  - A settings panel per rule to define its Scope (Global or Local repository paths).

### Tier 1.4: The File Sync Engine
- [ ] Define a standard `SyncAdapter` Rust Trait that handles formatting and pathing.
- [ ] Build custom "Post-Processors" (Adapters) implementing the trait for:
  - Antigravity / Gemini CLI (`GEMINI.md` format).
  - OpenCode (`AGENTS.md` format).
  - Cline (`.clinerules` format).
- [ ] Implement robust file writing, safely overwriting/appending rule content if multiple rules target the same adapter.
- [ ] **Conflict Warning:** Implement basic file-hash checking. If the target `.md` or `.toml` file was modified externally since the last sync, prompt the user with a warning before overwriting.
- [ ] Add a "Sync Now" button in the GUI to manually trigger the sync process and show success/error toasts.

---

## Phase 2: "Custom Commands" via Built-In MCP Server (Medium Priority)
*Objective: Introduce the Model Context Protocol (MCP) server running locally within the app to expose user-defined shell scripts and commands securely to AI tools.*

### Tier 2.1: MCP Server Foundation
- [ ] Research and integrate a Rust MCP SDK (or implement standard JSON-RPC over stdio/HTTP).
- [ ] Boot an active MCP Server on a dedicated local port (e.g., `localhost:8080`) when the Tauri app starts.
- [ ] Create connection instructions in the GUI showing users how to add this local server to their Claude Code or OpenCode configs.

### Tier 2.2: Managing Commands & UI Syncing
- [ ] Expand the Rust database to support `Command` models (Name, Description, Executable Script, Arguments).
- [ ] Build the "Commands" view in the GUI.
  - Interface to define the bash/powershell template.
  - Define expected arguments (dynamically mapped to the MCP tool schema).
  - **In-App Testing:** A "Test Run" button that executes the script with dummy arguments in the GUI, capturing and displaying `stdout`/`stderr` inside the app so users can verify the command works without opening another AI tool.
- [ ] **UI Stub Syncng:** Extend the `SyncAdapter` Trait from Phase 1 to also handle Commands. When a command is created, the Adapter will compile and write a `.toml` file for Gemini CLI and a `.md` file for Claude Code so that the slash command `/name` populates in their respective graphical UI dropdowns.

### Tier 2.3: Invocation and Execution
- [ ] Wire the saved `Commands` to dynamically map to the MCP Server's `list_tools` endpoint.
- [ ] Implement the `call_tool` MCP endpoint handler in Rust.
  - Safely extract arguments.
  - Spawn a system process (`std::process::Command`) to run the defined script.
  - Capture standard output and standard error and return them formatted to the requesting AI agent.

---

## Phase 3: "Skills" Pipelines (Lower Priority)
*Objective: Expand the MCP server to support complex, multi-file "Agent Skills" that wrap advanced python/node scripts and resources.*

### Tier 3.1: Skill Architecture
- [ ] Define the `Skill` model, representing a directory containing a `SKILL.md` file and executable helper scripts.
- [ ] Extend the GUI to allow importing/creating a full Skill directory structure, managing metadata.

### Tier 3.2: Advanced MCP Execution
- [ ] Expose Skills as native MCP tools.
- [ ] Implement logic in Rust to execute the entry-point script of a Skill, ensuring it has access to any packaged assets or environment variables defined in the Skill wrapper.

---

## Phase 4: Polish & Production Readiness
*Objective: Ensure the standalone application is robust enough for daily developer usage.*
- [ ] Implement an automatic file-watcher (e.g., `notify` crate in Rust) to automatically trigger the Sync Engine whenever a user edits a master file outside the GUI.
- [ ] Add a systemic tray icon so the database and MCP server can run continuously in the background without keeping the main GUI window open.
- [ ] Add export/import functionality to allow developers to share their rule configurations with their team.
- [ ] Build the production release executables for Windows/Mac/Linux.
