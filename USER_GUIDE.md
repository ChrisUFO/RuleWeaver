# RuleWeaver User Guide

## 1) Manage Rules

- Open `Rules` in the sidebar.
- Create or edit a rule with:
  - name
  - markdown content
  - scope (`global` or `local`)
  - enabled adapters
- Save changes.
- Use `Sync` to generate adapter-specific rule files.

Rule storage modes:

- `SQLite` (legacy)
- `File` (recommended): markdown files with YAML frontmatter under:
  - global: `~/.ruleweaver/rules/`
  - local: `{repo}/.ruleweaver/rules/`

You can switch/migrate in `Settings -> Storage`.

## 2) Manage Commands

- Open `Commands` in the sidebar.
- Create a command with:
  - name
  - description
  - script template (supports placeholders like `{{arg}}`)
  - argument definitions
  - `Expose via MCP` toggle
  - **`Generate Slash Commands`** toggle _(New!)_
  - **Target AI Tools** multi-select _(New!)_
- Use `Test Run` to execute locally and inspect stdout/stderr.
- Use `Sync Command Files` to generate command stubs for client UX.
- Use **`Sync Slash Commands`** to generate native `/commandname` triggers in supported AI tools.

### Slash Commands

RuleWeaver can generate **native slash commands** that appear in your AI tool's autocomplete when you type `/`.

**Supported Tools (8):**

- OpenCode - `$ARGUMENTS`, `$1-9`
- Claude Code - `$ARGUMENTS`, `$1-9`
- Cline - Workflows (natural language)
- Gemini CLI - `{{args}}`
- Cursor - Plain markdown
- Roo Code - `argument-hint`
- Antigravity - Workflows (natural language)
- Codex - Skills

**How to enable:**

1. Create or edit a command
2. Toggle `Generate Slash Commands` ON
3. Select which AI tools should receive this command
4. Save the command
5. Click `Sync Slash Commands`

**File Locations:**

- OpenCode: `~/.config/opencode/commands/{name}.md`
- Claude Code: `~/.claude/commands/{name}.md`
- Cline: `.clinerules/workflows/{name}.md`
- Gemini: `~/.gemini/commands/{name}.toml`
- Cursor: `~/.cursor/commands/{name}.md`
- Roo Code: `.roo/commands/{name}.md`
- Antigravity: `.agents/workflows/{name}.md`
- Codex: `.agents/skills/{name}/SKILL.md`

**Note:** Windsurf and Kilo Code do not support slash commands.

Generated command files:

- `~/.gemini/COMMANDS.toml`
- `~/.opencode/COMMANDS.md`
- `~/.claude/COMMANDS.md`
- `~/.cursorrules` (appended)
- `~/.clinerules` (appended)
- `.windsurf/rules/rules.md`
- `.roo/rules/rules.md`

## 3) Manage Skills (Phase 3 Foundation)

- Open `Skills` in the sidebar.
- Create a skill with:
  - name
  - description
  - instructions
  - scope (`global` or `local`)
  - project directory path (if `local`)
  - enabled flag

Current status:

- Skills CRUD and UI are available.
- Advanced MCP skill execution is planned next.

Security note:

- Skills run with your local user permissions.
- Only enable/import Skills you trust.

## 4) MCP Server Modes

RuleWeaver supports two MCP runtime modes:

1. Embedded MCP (desktop app process)
2. Standalone MCP binary (`ruleweaver-mcp`)

### Embedded MCP

- Start RuleWeaver desktop app.
- Go to `Settings -> MCP Server`.
- Click `Start`.
- Optional:
  - enable `Auto-start MCP`
  - enable `Minimize to tray on close` to keep MCP running in background

### Standalone MCP

Run directly:

```bash
cargo run --manifest-path src-tauri/Cargo.toml --bin ruleweaver-mcp -- --port 8080
```

Or use built binary:

```bash
ruleweaver-mcp --port 8080
```

## 5) Add MCP to Agent Tools

Use the snippets shown in `Settings -> MCP Server`.

### Claude Code

Use the generated `claude_code_json` snippet from RuleWeaver settings.

### OpenCode

Use the generated `opencode_json` snippet from RuleWeaver settings.

### Gemini, Cline, Codex, Antigravity, Kilo, Cursor, Windsurf, Roo Code

- Use synced rule/command files from RuleWeaver.
- If a specific MCP client integration is available in that tool, configure it to connect to the same localhost MCP endpoint shown in settings.

## 6) Troubleshooting

- Port conflict: change MCP port or stop the conflicting process.
- No tools listed: confirm commands are `Expose via MCP` and saved.
- No rules updating: verify adapter toggle in `Settings -> Adapters`, then resync.
- App closed unexpectedly: enable `Minimize to tray on close` to keep embedded MCP alive.
