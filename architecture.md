# System Architecture

AigentManager is designed as a standalone desktop application. It requires deep filesystem access to sync tool configurations globally and locally, as well as network capabilities to host a local server.

## Tech Stack
*   **Framework:** [Tauri](https://tauri.app/) (Provides a lightweight, fast, cross-platform standalone executable by combining a native Rust backend with a web frontend).
*   **Frontend:** React, TypeScript, TailwindCSS (for the GUI).
*   **Backend:** Rust.
*   **Database:** SQLite (Embedded via Rust) or `serde_json` file storage to store the master configurations.

## High-Level Architecture

The system is composed of three main layers:

### 1. The Presentation Layer (Frontend)
The React/TypeScript application running in the Tauri webview.
*   **State Management:** Holds the UI state for editing Rules, Commands, and Skills.
*   **Communication:** Communicates with the Rust backend via Tauri IPC (Inter-Process Communication) to save rules, trigger syncs, and start/stop the MCP server.

### 2. The Core Logic Layer (Rust Backend)
This layer handles all OS-level operations.
*   **Database Manager:** Reads and writes the master configuration (the definitions of Global/Local rules and their textual content).
*   **File Sync Engine (The "Adapters"):** 
    *   Because every AI tool expects a different filename (`GEMINI.md`, `AGENTS.md`, `.clinerules`) or specific frontmatter, the Sync Engine acts as a collection of **Tool-Specific Adapters (Post-Processors)**.
    *   When a sync is triggered, the engine takes the master Rule and runs it through each active adapter. The adapter handles tool-specific formatting (e.g., prepending XML tags for Claude, or formatting TOML headers) and determines the exact target directory based on the "Scope".
    *   Writes file outputs directly to the filesystem.
*   **MCP Server Engine:**
    *   Runs a local HTTP/Stdio MCP server on a designated port.
    *   Reads the "Commands" and "Skills" from the database.
    *   Exposes these as MCP-compliant `tools` to any client that connects.
    *   Executes the shell commands when an AI agent requests a tool invocation.

### 3. The Target Layer (The AI Tools)
*   **File Watchers:** AI tools (like Cline, OpenCode) naturally watch for changes in their rule files. When the Sync Engine updates a file, the AI tool seamlessly picks it up.
*   **MCP Clients:** AI tools (like Claude Code, OpenCode) connect to the Rust MCP Server port. When asked to perform an action, they query the MCP server for available Commands/Skills.

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
  ]
}
```
