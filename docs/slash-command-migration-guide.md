# Slash Command Migration Guide

## Overview

RuleWeaver now supports **native slash commands** for 8 AI tools. This guide helps existing users migrate from the old command documentation files to the new slash command system.

## What's New

### Before (v1.x)

- Commands generated as reference files (`COMMANDS.md`, `COMMANDS.toml`)
- Users manually copy/paste command names
- No autocomplete support in AI tools

### After (v2.x)

- Commands generate as **native slash commands** (`/commandname`)
- Appears in AI tool autocomplete
- Automatic file sync to correct directories
- Supports tool-specific argument formats

## Supported Tools

| Tool        | Slash Command Support | File Location                      |
| ----------- | --------------------- | ---------------------------------- |
| OpenCode    | ✅                    | `~/.config/opencode/commands/*.md` |
| Claude Code | ✅                    | `~/.claude/commands/*.md`          |
| Cline       | ✅                    | `.clinerules/workflows/*.md`       |
| Gemini CLI  | ✅                    | `~/.gemini/commands/*.toml`        |
| Cursor      | ✅                    | `~/.cursor/commands/*.md`          |
| Roo Code    | ✅                    | `.roo/commands/*.md`               |
| Antigravity | ✅                    | `.agents/workflows/*.md`           |
| Codex       | ✅                    | `.agents/skills/*/SKILL.md`        |
| Windsurf    | ❌                    | Rules only                         |
| Kilo Code   | ❌                    | Rules only                         |

## Migration Steps

### Step 1: Update RuleWeaver

Pull the latest changes and rebuild:

```bash
git pull origin main
cd src-tauri
cargo build --release
```

### Step 2: Enable Slash Commands

For each existing command:

1. Open **Commands** in the sidebar
2. Select a command to edit
3. Toggle **"Generate Slash Commands"** to ON
4. Select target AI tools from the multi-select
5. Save the command
6. Click **"Sync Slash Commands"**

### Step 3: Verify Files

Check that files were created in the correct locations:

```bash
# OpenCode
ls ~/.config/opencode/commands/

# Claude Code
ls ~/.claude/commands/

# Cline
ls ~/Documents/Cline/Workflows/  # Global
ls .clinerules/workflows/        # Local

# Gemini CLI
ls ~/.gemini/commands/

# Cursor
ls ~/.cursor/commands/

# Roo Code
ls ~/.roo/commands/              # Global
ls .roo/commands/                # Local

# Antigravity
ls ~/.gemini/antigravity/global_workflows/  # Global
ls .agents/workflows/                      # Local

# Codex (Skills)
ls ~/.agents/skills/             # Global
ls .agents/skills/               # Local
```

### Step 4: Test in AI Tools

Open your AI tool and type `/` to see if your commands appear in the autocomplete.

## Tool-Specific Notes

### OpenCode

- Supports `$ARGUMENTS` for all arguments
- Supports `$1`, `$2`, etc. for positional arguments
- Supports shell command injection with ``!`command` ``
- Supports file references with `@filename`

Example:

```markdown
---
name: deploy
description: Deploy the application
---

Run deployment for: $1
Current branch: !`git branch --show-current`
```

### Claude Code

- Supports `$ARGUMENTS` for all arguments
- Supports `$1`, `$2`, etc. for positional arguments
- Supports `context: fork` for subagent execution

### Cline (Workflows)

- Uses **workflows** (not traditional slash commands)
- Natural language argument handling
- Supports XML tool syntax

### Gemini CLI

- Uses TOML format
- Supports `{{args}}` template variable
- Supports `!{command}` for shell execution

### Cursor

- Plain markdown format
- Arguments passed as natural language after command

### Roo Code

- Supports `argument-hint` in frontmatter
- Arguments passed as natural language after command

### Antigravity (Workflows)

- Uses **workflows** (not traditional slash commands)
- YAML frontmatter required
- Natural language argument handling

### Codex (Skills)

- Uses **Agent Skills** structure
- Directory-based: `{name}/SKILL.md`
- Invoked via `/skills` or `$skill-name`

## Argument Substitution Reference

| Tool        | Pattern              | Example                    |
| ----------- | -------------------- | -------------------------- |
| OpenCode    | `$ARGUMENTS`, `$1-9` | `npm test $1`              |
| Claude Code | `$ARGUMENTS`, `$1-9` | `deploy $ARGUMENTS`        |
| Cline       | Natural language     | Arguments appear in prompt |
| Gemini CLI  | `{{args}}`           | `deploy {{args}}`          |
| Cursor      | Natural language     | Arguments appear in prompt |
| Roo Code    | `argument-hint`      | Shows in UI hint           |
| Antigravity | Natural language     | Arguments appear in prompt |
| Codex       | Natural language     | Skills-based invocation    |

## Cleanup Old Files

After migration, you may want to clean up old command documentation files:

```bash
# Remove old COMMANDS.md files (optional)
rm ~/.opencode/COMMANDS.md
rm ~/.claude/COMMANDS.md
rm ~/.cursor/COMMANDS.md
rm ~/.clinerules
```

**Note:** Keep `.cursorrules` and `.windsurfrules` as they are still needed for Rules.

## Troubleshooting

### Commands not appearing in autocomplete

1. Verify files exist in correct directories
2. Restart your AI tool
3. Check file permissions
4. Verify command name doesn't contain invalid characters

### Files not syncing

1. Check RuleWeaver logs
2. Verify target directories exist and are writable
3. Try "Sync Slash Commands" button again
4. Check for conflicting files

### Argument substitution not working

1. Verify tool supports argument substitution
2. Check argument syntax matches tool requirements
3. Test with simple command first

## Backward Compatibility

The old MCP-based command execution still works. Slash commands are an **additional** feature that provides better discoverability.

## Getting Help

- See full documentation: `docs/ai-tools-commands-reference.md`
- Check implementation plan: `PLAN.md`
- File issues on GitHub

## Migration Checklist

- [ ] Update RuleWeaver to latest version
- [ ] Enable slash commands for each command
- [ ] Select target AI tools
- [ ] Sync slash commands
- [ ] Verify files in target directories
- [ ] Test commands in AI tools
- [ ] Clean up old documentation files (optional)
- [ ] Update team documentation

## Support Matrix Summary

✅ = Full support (slash commands + rules)
❌ = Rules only

| Tool        | Rules | Slash Commands | Skills |
| ----------- | ----- | -------------- | ------ |
| OpenCode    | ✅    | ✅             | ✅     |
| Claude Code | ✅    | ✅             | ✅     |
| Cline       | ✅    | ✅             | ✅     |
| Gemini CLI  | ✅    | ✅             | ✅     |
| Cursor      | ✅    | ✅             | ✅     |
| Roo Code    | ✅    | ✅             | ✅     |
| Antigravity | ✅    | ✅             | ✅     |
| Codex       | ✅    | ✅             | ✅     |
| Windsurf    | ✅    | ❌             | ❌     |
| Kilo Code   | ✅    | ❌             | ❌     |

---

**Version:** 2.0.0+

**Last Updated:** February 2026
