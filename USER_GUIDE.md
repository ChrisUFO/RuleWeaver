# RuleWeaver User Guide

RuleWeaver is a unified desktop application that centrally manages configurations, rules, commands, and skills for AI coding assistants. Instead of juggling different file formats across 10+ AI tools, RuleWeaver acts as a single source of truth with a **Hybrid Synchronization Model**.

## Table of Contents

1. [Overview](#1-overview)
2. [Dashboard](#2-dashboard)
3. [Managing Rules](#3-managing-rules)
4. [Managing Commands](#4-managing-commands)
5. [Managing Skills](#5-managing-skills)
6. [Settings](#6-settings)
7. [MCP Server](#7-mcp-server)
8. [Keyboard Shortcuts](#8-keyboard-shortcuts)
9. [Troubleshooting](#9-troubleshooting)

---

## 1) Overview

### Supported AI Tools (10)

| Tool        | Rules File                     | Slash Commands                     | Command Stubs | Skills                          |
| ----------- | ------------------------------ | ---------------------------------- | ------------- | ------------------------------- |
| Antigravity | `~/.gemini/GEMINI.md`          | `.agents/workflows/*.md`           | ✅            | `~/.gemini/antigravity/skills/` |
| Gemini CLI  | `~/.gemini/GEMINI.md`          | `~/.gemini/commands/*.toml`        | ✅            | `~/.gemini/skills/`             |
| OpenCode    | `~/.config/opencode/AGENTS.md` | `~/.config/opencode/commands/*.md` | ✅            | `~/.config/opencode/skills/`    |
| Cline       | `~/.clinerules`                | `~/Documents/Cline/Workflows/`     | ✅            | `~/.cline/skills/` (Exp.)       |
| Claude Code | `~/.claude/CLAUDE.md`          | `~/.claude/commands/*.md`          | ✅            | `~/.claude/skills/`             |
| Codex       | `~/.codex/AGENTS.md`           | Skills                             | ✅            | `~/.agents/skills/`             |
| Kilo Code   | `~/.kilocode/rules/AGENTS.md`  | —                                  | ❌ (pending)  | ❌ (pending)                    |
| Cursor      | `~/.cursorrules`               | `~/.cursor/commands/*.md`          | ❌            | ❌                              |
| Windsurf    | `~/.windsurf/rules/rules.md`   | — (pending)                        | ❌            | `~/.windsurf/skills/`           |
| Roo Code    | `~/.roo/rules/rules.md`        | `~/.roo/commands/*.md`             | ✅            | `~/.roo/skills/`                |

> **Note:** Cursor does not support command stubs or skills distribution. Windsurf and Kilo Code have capability flags set in the registry but directory paths are not yet configured — distribution will be enabled once they publish their directory specs. See [`docs/PARITY.md`](docs/PARITY.md) for the full capability matrix.

### Navigation

RuleWeaver 2.0 uses a streamlined navigation structure:

- **Dashboard**: Your Command Center. Monitor system health, drift status, and recent activity.
- **Rules**: Manage your core rule definitions and artifact metadata.
- **Commands**: Execute script-based tools and repo-local shortcuts.
- **Skills**: Orchestrate complex workflows through structured skill sets.
- **Settings**: Configure global and local application contexts.

The sidebar can be collapsed/expanded using the chevron buttons.

---

## 2) Dashboard

The Dashboard provides a high-level overview and quick actions.

### Statistics Cards

- **Total Rules** — Number of rules in your database
- **Global Rules** — Rules applied everywhere
- **Local Rules** — Repository-specific rules
- **Last Sync** — When rules were last synchronized

### Quick Actions

- **Sync All** — Preview and sync all rules to AI tool config files
- **New Rule** — Create a new rule

### Quick Start Templates

Pre-built rule templates to get started quickly:

- TypeScript Best Practices
- React Components
- Python Standards
- Git Commit Rules

Click any template to create a rule with pre-filled content.

### Recent Sync History

Shows the last 5 sync operations with:

- Status (success, partial, failed)
- Number of files written
- Timestamp
- Trigger source (manual/auto)

---

## 3) Managing Rules

### Rules List

The Rules page displays all your rules with:

**Filtering:**

- Search by name or content
- Filter by scope (All, Global, Local)
- Filter by adapter (specific AI tool)
- Sort by name, created date, updated date, or enabled status

**Bulk Actions:**

- Select multiple rules using checkboxes
- Bulk enable/disable selected rules
- Bulk delete selected rules

**Individual Actions (per rule):**

- Toggle enable/disable
- Edit
- Duplicate
- Delete (with undo option)

### Creating a Rule

1. Click **New Rule** button
2. Enter a **name** for the rule
3. Write the **markdown content** (your coding standards, guidelines, etc.)
4. Choose **scope**:
   - **Global** — Applies everywhere, stored in `~/.ruleweaver/rules/`
   - **Local** — Repository-specific, stored in `{repo}/.ruleweaver/rules/`
5. If Local, select one or more **target repositories**
6. Select **adapters** (which AI tools should receive this rule)
7. Click **Save Selection**

### Rule Editor Features

- **Live Preview** — See how the rule will appear when synced
- **Adapter Tabs** — Switch between adapter previews
- **Word/Character Count** — Track content size
- **Open in Explorer** — Open the target folder for an adapter
- **Keyboard Shortcut** — `Ctrl+S` to save

### Syncing Rules

Rules must be synced to take effect in AI tools:

1. Click **Sync** from Dashboard or Rules page
2. Review the **Sync Preview** dialog showing:
   - Files to be updated
   - Any conflicts detected
3. Resolve conflicts if any (choose local or remote version)
4. Confirm sync

### Importing Rules

#### Import AI Tool Rules

Scans your system for existing AI tool configurations and imports them:

1. Click **Import AI** button
2. Review discovered rules from various tools
3. Select which to import
4. Choose conflict policy (rename, skip, replace)
5. Optionally override scope and adapters

#### Import from File

Import a single `.md`, `.txt`, `.json`, `.yaml`, or `.yml` file.

#### Import from Folder

Bulk-import supported files from a directory recursively.

#### Import from URL

Fetch and import remote rule content. Only `http`/`https` URLs allowed (localhost/private IPs blocked for security).

#### Import from Clipboard

Import text currently in your clipboard.

#### Drag-and-Drop Import

Drag rule files onto the Rules page to trigger import preview.

### Import Options

- **Scope Override** — Force global or local scope regardless of source
- **Adapter Override** — Assign specific adapters on import
- **Conflict Mode**:
  - `rename` — Add suffix to avoid collisions (default)
  - `skip` — Skip conflicting rules
  - `replace` — Overwrite existing rules

### Rule Storage Modes

- **SQLite** (legacy) — Database storage
- **File** (recommended) — Markdown files with YAML frontmatter:
  - Global: `~/.ruleweaver/rules/`
  - Local: `{repo}/.ruleweaver/rules/`

Switch/migrate in **Settings → Storage**.

### Artifact Status Dashboard

The Status page provides a unified view of all RuleWeaver-managed artifacts across your repositories and AI tools.

- **Summary Row**: At-a-glance counts of Synced, Out of Date, Missing, Conflicted, and Unsupported artifacts.
- **Filtering**: Narrow by artifact type (Rules / Commands / Skills), by AI tool, by scope (Global / Local), or by sync status.
- **Deep-Linking**: Click the icon in any status row to navigate directly to the Rule, Command, or Skill editor for that item.
- **Targeted Repair**: Fix individual sync issues with the per-row Repair button.
- **Repair All**: Resolves all drifted/missing artifacts for the current filter in one click.

### Artifact Lifecycle

Understanding the RuleWeaver artifact lifecycle helps diagnose sync issues:

1. **Create/Edit**: You create or update a rule, command, or skill in the RuleWeaver UI and save it. The artifact is persisted in the database.
2. **Sync/Reconcile**: RuleWeaver computes the _desired state_ (what files should exist based on your configuration) and the _actual state_ (what files are on disk). It then writes, updates, or removes files to bring actual state in line with desired state.
   - Rules are written as tool-specific files (e.g., `CLAUDE.md`, `GEMINI.md`).
   - Commands generate stub files (`COMMANDS.md` / `COMMANDS.toml`) and slash command files.
   - Skills generate `SKILL.md` files in each adapter's skill directory.
3. **Drift**: If files are deleted externally, a tool path changes, or an adapter is removed from targeting, the artifact becomes _Out of Date_ or _Missing_.
4. **Repair**: Use the Status page's Repair button to re-sync a specific artifact, or Repair All to resolve all drift at once.

---

## 4) Managing Commands

Commands are executable scripts that can be triggered via MCP or slash commands.

### Commands List

- Search commands by name
- Create new commands
- Sync command files

### Creating a Command

1. Click **New** button
2. Enter **name** and **description**
3. Write the **script** (shell commands, supports placeholders)
4. Define **arguments** (name, description, required, default value)
5. **Execution Policy**:
   - **Timeout (ms)**: Set a maximum execution time (defaults to 30,000ms).
   - **Max Retries**: Configure up to 3 automatic retries for transient failures (timeouts, network errors).
6. Optionally select **target repositories**
7. Toggle **Expose via MCP** to make available to AI tools
8. Toggle **Generate Slash Commands** for native `/command` support
9. Select **target AI tools** for slash commands
10. Click **Save**

### Testing Commands

- Click **Test Run** to execute locally.
- **Redaction**: RuleWeaver automatically redacts sensitive patterns (tokens, keys, passwords) from your command history to prevent accidental exposure.
- View stdout/stderr output.
- See exit code, duration, and retry attempts.
- Review recent execution history with status badges.

### Slash Commands

RuleWeaver generates native slash commands that appear in AI tool autocomplete.

**Supported Tools (8):**
| Tool | Placeholder Syntax |
|------|-------------------|
| OpenCode | `$ARGUMENTS`, `$1-9` |
| Claude Code | `$ARGUMENTS`, `$1-9` |
| Cline | Natural language workflows |
| Gemini CLI | `{{args}}` |
| Cursor | Plain markdown |
| Roo Code | `argument-hint` |
| Antigravity | Natural language workflows |
| Codex | Skills |

**File Locations:**

- OpenCode: `~/.config/opencode/commands/{name}.md`
- Claude Code: `~/.claude/commands/{name}.md`
- Cline: `~/Documents/Cline/Workflows/{name}.md` (global), `.clinerules/workflows/{name}.md` (local)
- Gemini: `~/.gemini/commands/{name}.toml`
- Cursor: `~/.cursor/commands/{name}.md`
- Roo Code: `.roo/commands/{name}.md`
- Antigravity: `.agents/workflows/{name}.md`
- Codex: `.agents/skills/{name}/SKILL.md`

**To enable:**

1. Create/edit a command
2. Toggle **Generate Slash Commands** ON
3. Select target AI tools
4. Save
5. Click **Sync Slash Commands**

---

## 5) Managing Skills

Skills are complex multi-step workflows distributed as `SKILL.md` files to your AI tools' skill directories. RuleWeaver manages the full lifecycle: create once, distribute everywhere.

### Skills List

- View all installed skills
- Browse and install templates
- Create new skills

### Creating a Skill

1. Click **New** button
2. Enter **name** and **description**
3. Write detailed **instructions** (SKILL.md content)
4. Define **input schema** parameters:
   - Name, description
   - Type (String, Number, Boolean, Enum, Array, Object)
   - Required flag
   - Default value
   - Enum values (for Enum type)
5. Specify **entry point** (e.g., `run.sh`, `index.js`)
6. Choose **scope** (global or local)
7. If local, select one or more **target repositories**
8. Configure **Adapter Distribution** (see below)
9. Toggle **enabled**
10. Click **Save Changes**

### Adapter Distribution (Targeting)

The **Adapter Distribution** section controls which AI tools receive this skill's `SKILL.md` file.

- **All supported adapters (default):** Leave all checkboxes unchecked. The skill is distributed to every tool that supports the Agent Skills standard: Antigravity, Claude Code, Cline, Codex, Gemini, OpenCode, Roo Code, Windsurf.
- **Specific adapters:** Check only the tools you want to receive this skill. Unchecked tools will not receive the file (and any existing file from this skill will be removed on the next reconcile).

**Tools that do not appear in the list** (Cursor, Kilo Code) do not support the Agent Skills standard in a way compatible with RuleWeaver's distribution engine. These tools are silently skipped.

### Skill File Locations

Skills are written as `SKILL.md` inside a named subdirectory under each tool's skill directory:

| Tool        | Global Skills Path                     | Local Skills Path          |
| ----------- | -------------------------------------- | -------------------------- |
| Antigravity | `~/.gemini/antigravity/skills/<name>/` | `.agents/skills/<name>/`   |
| Claude Code | `~/.claude/skills/<name>/`             | `.claude/skills/<name>/`   |
| Cline       | `~/.cline/skills/<name>/`              | `.cline/skills/<name>/`    |
| Codex       | `~/.agents/skills/<name>/`             | `.agents/skills/<name>/`   |
| Gemini CLI  | `~/.gemini/skills/<name>/`             | `.gemini/skills/<name>/`   |
| OpenCode    | `~/.config/opencode/skills/<name>/`    | `.opencode/skills/<name>/` |
| Roo Code    | `~/.roo/skills/<name>/`                | `.roo/skills/<name>/`      |
| Windsurf    | `~/.windsurf/skills/<name>/`           | `.windsurf/skills/<name>/` |

### Template Browser

Install pre-built skill templates:

1. Click **Browse Templates**
2. Review available templates
3. Click **Install** on desired template
4. Wait for compilation

### Security Warning

Skills execute shell commands with your user privileges. Only enable/import skills from trusted sources.

---

## 6) Settings

### App Data

View the application data directory location. Click the folder icon to open in file manager.

### Repository Roots

Configure repositories once for use across the app:

- Add repositories via folder picker
- Remove individual repositories
- Save changes

Repository roots are used for:

- Local rule target selection
- Local command repository targeting
- Local skill directory paths
- Import discovery

### MCP Server

Configure the Model Context Protocol server:

- **Status** — Running/Stopped with port and uptime
- **Start/Stop** — Control the server
- **Auto-start MCP** — Start automatically when RuleWeaver launches
- **Minimize to tray on close** — Keep MCP running when window closes
- **Launch on startup** — Start RuleWeaver on system login
- **Connection snippets** — Copy configuration for Claude Code and OpenCode

### Storage

Manage rule storage:

- View current mode (SQLite or File)
- See storage statistics (rule count, size)
- **Migrate to File Storage** — Move from SQLite to markdown files
- **Verify Migration** — Confirm migration integrity
- **Rollback** — Restore from backup if needed

### Adapters

Enable/disable individual AI tool adapters:

- Toggle which adapters participate in sync
- View adapter file names and paths

### Slash Commands

- Auto-sync on save (coming soon)
- Sync all slash commands manually

### Data Management

- **Export Configuration** — Save rules, commands, and skills to JSON/YAML
- **Import Configuration** — Load from a backup file

### About

- Version information
- Check for updates
- Links to GitHub and issue tracker

---

## 7) MCP Server

RuleWeaver supports two MCP runtime modes:

### Embedded MCP

Runs within the desktop application:

1. Start RuleWeaver
2. Go to **Settings → MCP Server**
3. Click **Start**
4. Enable **Auto-start MCP** for convenience
5. Enable **Minimize to tray on close** to keep running in background

### Standalone MCP

Run the MCP server independently:

```bash
# From source
cargo run --manifest-path src-tauri/Cargo.toml --bin ruleweaver-mcp -- --port 8080

# Using built binary
ruleweaver-mcp --port 8080
```

### Connecting AI Tools

Use the configuration snippets shown in **Settings → MCP Server**:

**Claude Code:**
Add the generated JSON to your Claude Code configuration.

**OpenCode:**
Add the generated JSON to your OpenCode configuration.

**Other tools:**
Use synced rule/command files, or configure MCP client to connect to the localhost endpoint.

---

## 8) Keyboard Shortcuts

| Shortcut       | Action                         |
| -------------- | ------------------------------ |
| `Ctrl+N`       | Create new rule                |
| `Ctrl+Shift+N` | Create new command             |
| `Ctrl+S`       | Save current item              |
| `Ctrl+Shift+S` | Sync all rules                 |
| `Ctrl+F`       | Focus search                   |
| `Ctrl+,`       | Open settings                  |
| `Ctrl+1`       | Go to dashboard                |
| `Ctrl+2`       | Go to rules                    |
| `Ctrl+3`       | Go to commands                 |
| `Ctrl+4`       | Go to skills                   |
| `Shift+?`      | Show keyboard shortcuts dialog |
| `Escape`       | Close dialog                   |

---

## 9) Troubleshooting

### Sync Issues

- **Port conflict** — Change MCP port or stop conflicting process
- **No tools listed** — Confirm commands have "Expose via MCP" enabled
- **Rules not updating** — Verify adapter toggle in Settings → Adapters, then resync
- **Conflicts detected** — Use conflict resolution dialog to choose local or remote version

### Import Issues

- **URL import blocked** — Only `http`/`https` allowed; localhost/private IPs blocked for security
- **Drag-and-drop not working** — Some platforms don't expose file paths; use "Import File" instead
- **No candidates found** — Ensure source files exist and are readable

### MCP Issues

- **Server won't start** — Check port availability
- **Tools not appearing** — Verify MCP connection in AI tool settings
- **App closed unexpectedly** — Enable "Minimize to tray on close" to keep embedded MCP alive

### Storage Issues

- **Migration failed** — Check logs; use rollback to restore
- **Rules missing after migration** — Run "Verify Migration" to check integrity
- **Permission denied** — Ensure RuleWeaver has access to target directories

### General Issues

- **App running slow** — Check number of rules; consider archiving unused ones
- **Changes not persisting** — Ensure you click "Save" before navigating away
- **Toast notifications missing** — Check system notification settings
