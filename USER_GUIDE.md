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
- Use `Test Run` to execute locally and inspect stdout/stderr.
- Use `Sync Command Files` to generate command stubs for client UX.

Generated command files:

- `~/.gemini/COMMANDS.toml`
- `~/.opencode/COMMANDS.md`
- `~/.claude/COMMANDS.md`

## 3) Manage Skills (Phase 3 Foundation)

- Open `Skills` in the sidebar.
- Create a skill with:
  - name
  - description
  - instructions
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

### Gemini, Cline, Codex, Antigravity

- Use synced rule/command files from RuleWeaver.
- If a specific MCP client integration is available in that tool, configure it to connect to the same localhost MCP endpoint shown in settings.

## 6) Troubleshooting

- Port conflict: change MCP port or stop the conflicting process.
- No tools listed: confirm commands are `Expose via MCP` and saved.
- No rules updating: verify adapter toggle in `Settings -> Adapters`, then resync.
- App closed unexpectedly: enable `Minimize to tray on close` to keep embedded MCP alive.
