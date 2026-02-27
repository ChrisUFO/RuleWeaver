# AI Tools Commands Reference

A comprehensive guide to how each AI coding tool handles **rules**, **custom commands/slash commands**, and **skills**.

## Overview

| Tool        | Rules Support     | Rule Import Support | Slash Commands    | Command Stubs     | Skills Support    | Config Location                        |
| ----------- | ----------------- | ------------------- | ----------------- | ----------------- | ----------------- | -------------------------------------- |
| OpenCode    | ✅ Global + Local | ✅ Global + Local   | ✅ Global + Local | ✅ Global + Local | ✅ Global + Local | `~/.config/opencode/`, `.opencode/`    |
| Claude Code | ✅ Global + Local | ✅ Global + Local   | ✅ Global + Local | ✅ Global + Local | ✅ Global + Local | `~/.claude/`, `.claude/`               |
| Cline       | ✅ Global + Local | ✅ Global + Local   | ✅ Global + Local | ✅ Global + Local | ✅ Global + Local | `~/.clinerules`, `.clinerules/`        |
| Gemini CLI  | ✅ Global + Local | ✅ Global + Local   | ✅ Global + Local | ✅ Global + Local | ✅ Global + Local | `~/.gemini/`, `.gemini/`               |
| Cursor      | ✅ Global + Local | ✅ Global + Local   | ✅ Global + Local | ❌ No             | ❌ No             | `~/.cursor/`, `.cursor/`               |
| Roo Code    | ✅ Global + Local | ✅ Global + Local   | ✅ Global + Local | ✅ Global + Local | ✅ Global + Local | `~/.roo/`, `.roo/`                     |
| Antigravity | ✅ Global + Local | ✅ Global + Local   | ✅ Global + Local | ✅ Global + Local | ✅ Global + Local | `~/.gemini/antigravity/`, `.agents/`   |
| Windsurf    | ✅ Global + Local | ✅ Global + Local   | ❌ No¹            | ❌ No             | ✅ Global + Local | `.windsurfrules`, `~/.windsurf/`       |
| Kilo Code   | ✅ Global + Local | ✅ Global + Local   | ❌ No             | ❌ No²            | ❌ No²            | `~/.kilocode/`, `.kilocode/`           |
| Codex       | ✅ Global + Local | ✅ Global + Local   | ✅ Global + Local | ✅ Global + Local | ✅ Global + Local | `~/.agents/skills/`, `.agents/skills/` |

¹ Windsurf has `supports_slash_commands: true` in the registry capability flags, but no slash command directory path is configured. No slash command files are distributed until Windsurf publishes their spec. See [Windsurf](#windsurf) for details.

² Kilo Code has `supports_command_stubs: true` and `supports_skills: true` in registry capability flags, but neither command stub paths nor skill directory paths are configured. Nothing is distributed until Kilo Code publishes their directory spec. See [Kilo Code](#kilo-code) for details.

---

## Rule Import Notes

- RuleWeaver supports import from known AI tool rule locations in both global and local scopes.
- Import supports scan + preview + selection before execution.
- Import supports additional override options during execution: scope override and adapter override.
- Duplicate content is skipped automatically.
- Conflict handling supports `rename`, `skip`, and `replace`.
- Same-name cross-tool imports can be automatically disambiguated with a tool suffix (for example, `quality-cline`).
- Rules page includes a drag-and-drop import target for direct file scan/import preview.
- Configure repository roots in Settings to power local artifact selection and local AI tool path discovery.
- Commands and skills can also reference the same repository roots for consistent local-target configuration.

---

## Artifact Auto-Refresh

RuleWeaver includes a background file watcher that automatically detects changes to your local skill and command artifacts.

### How it Works

- **Discovery:** When the MCP server starts, RuleWeaver identifies all local directories used by enabled skills (`directoryPath`) and commands with `targetPaths`.
- **Watching:** Recursive filesystem watchers are established for these directories using the `notify` crate.
- **Auto-Refresh:** When a file modification or rename is detected, RuleWeaver automatically triggers a refresh of the MCP tool registry.
- **Debouncing:** Changes are debounced (500ms) to ensure multiple rapid saves only trigger a single refresh.

### UI Indicators

- **Watching Icon:** A pulsing "Eye" icon (👁️) appears next to enabled skills and commands in their respective lists when the MCP server is actively watching them.
- **MCP Status:** The "Watching" badge in the MCP Settings card indicates if the background watcher is operational.

---

## OpenCode

### Rules

- **Global Rules:** `~/.config/opencode/rules/*.md`
- **Local Rules:** `.opencode/rules/*.md`
- **Format:** Markdown with YAML frontmatter
- **Behavior:** Both global and local rules are merged, with local rules taking precedence

### Custom Commands (Slash Commands)

- **Global Commands:** `~/.config/opencode/commands/*.md`
- **Local Commands:** `.opencode/commands/*.md`
- **Format:** Markdown with YAML frontmatter
- **Command Name:** Filename becomes the command (e.g., `test.md` → `/test`)
- **Frontmatter:**
  - `description`: Shown in TUI command list
  - `agent`: Optional agent to execute command
  - `model`: Optional model override
  - `subtask`: Force subagent invocation (boolean)
- **Arguments:**
  - `$ARGUMENTS`: All arguments as string
  - `$1`, `$2`, etc.: Positional arguments
  - ``!`command` ``: Shell command injection
  - `@filename`: File references

### Skills

- **Global Skills:** `~/.config/opencode/skills/<skill-name>/SKILL.md`
- **Local Skills:** `.opencode/skills/<skill-name>/SKILL.md`
- **Claude-Compatible:** `~/.claude/skills/<skill-name>/SKILL.md`, `.claude/skills/<skill-name>/SKILL.md`
- **Agents-Compatible:** `~/.agents/skills/<skill-name>/SKILL.md`, `.agents/skills/<skill-name>/SKILL.md`
- **Format:** Markdown with YAML frontmatter (Agent Skills standard)
- **Frontmatter:**
  - `name`: Required, 1-64 chars, lowercase alphanumeric with hyphens
  - `description`: Required, 1-1024 characters
  - `license`: Optional
  - `compatibility`: Optional
  - `metadata`: Optional key-value map
- **Discovery:** OpenCode walks up from CWD to git worktree, loading skills from all matching directories
- **Invocation:** Loaded via `skill` tool when agent decides it's relevant
- **Permissions:** Controlled via `opencode.json` with pattern-based rules (`allow`, `deny`, `ask`)

**Documentation:** https://opencode.ai/docs/skills/

---

## Claude Code

### Rules

- **Global Rules:** `~/.claude/CLAUDE.md`
- **Local Rules:** `.claude/CLAUDE.md`
- **Format:** Markdown with optional YAML frontmatter
- **Behavior:** Local rules override global rules for that project

### Custom Commands (Slash Commands)

- **Global Commands:** `~/.claude/commands/*.md`
- **Local Commands:** `.claude/commands/*.md`
- **Format:** Markdown with YAML frontmatter (Agent Skills standard)
- **Command Name:** Filename becomes the command (e.g., `review.md` → `/review`)
- **Frontmatter:**
  - `name`: Command name (optional, defaults to filename)
  - `description`: Command description
  - `tools`: List of allowed tools
  - `context`: Execution context (e.g., `fork` for subagent)
- **Arguments:**
  - `$ARGUMENTS`: All arguments
  - `$1`, `$2`, etc.: Positional arguments
  - `!command`: Dynamic context injection

### Skills

- **Global Skills:** `~/.claude/skills/<skill-name>/SKILL.md`
- **Local Skills:** `.claude/skills/<skill-name>/SKILL.md`
- **Format:** Agent Skills standard (YAML frontmatter + Markdown)
- **Structure:** Directory-based with SKILL.md file
- **Frontmatter:**
  - `name`: Required, used for invocation
  - `description`: Required, used for matching
- **Invocation:**
  - Explicit: Type skill name or use `$skill-name`
  - Implicit: Based on description matching

**Documentation:** https://code.claude.com/docs/en/skills

---

## Cline

### Rules

- **Global Rules:** `~/Documents/Cline/Rules/*.md` (macOS/Windows/Linux)
- **Local Rules:** `.clinerules/*.md`
- **Format:** Markdown files
- **Behavior:** Local rules override global rules for that project

### Custom Commands (Slash Commands)

- **Global Commands:** `~/Documents/Cline/Workflows/*.md`
- **Local Commands:** `.clinerules/workflows/*.md`
- **Format:** Markdown with numbered steps
- **Command Name:** Filename becomes the command (e.g., `deploy.md` → `/deploy`)
- **Features:**
  - Step-by-step task automation
  - Natural language instructions
  - XML tool syntax for precise control
  - MCP tool integration
- **Example:**

  ````markdown
  # Deploy Workflow

  ## Step 1: Check prerequisites

  Verify the environment is ready.

  ## Step 2: Run the build

  ```bash
  npm run build
  ```

  ## Step 3: Verify results

  Check that the build completed successfully.
  ````

### Skills

- **Status:** Experimental feature (enable in Settings → Features → Enable Skills)
- **Global Skills:** `~/.cline/skills/<skill-name>/SKILL.md`
- **Local Skills:** `.cline/skills/<skill-name>/SKILL.md`
- **Claude-Compatible:** `~/.claude/skills/<skill-name>/SKILL.md`, `.claude/skills/<skill-name>/SKILL.md`
- **Format:** Markdown with YAML frontmatter (Agent Skills standard)
- **Frontmatter:**
  - `name`: Required, must match directory name
  - `description`: Required, max 1024 characters, determines when skill triggers
- **Structure:**
  ```
  my-skill/
  ├── SKILL.md          # Required: main instructions
  ├── docs/             # Optional: additional documentation
  ├── templates/        # Optional: config templates
  └── scripts/          # Optional: utility scripts
  ```
- **Loading:** Progressive disclosure - metadata (~100 tokens) always loaded, full instructions only when triggered
- **Activation:** Via `use_skill` tool when request matches description
- **Toggling:** Skills can be enabled/disabled per skill
- **Precedence:** Global skills take precedence over project skills with same name

**Documentation:** https://docs.cline.bot/customization/skills

---

## Gemini CLI

### Rules

- **Global Rules:** `~/.gemini/GEMINI.md`
- **Local Rules:** `.gemini/GEMINI.md`
- **Format:** Markdown files
- **Behavior:** Both global and local rules are merged

### Custom Commands (Slash Commands)

- **Global Commands:** `~/.gemini/commands/*.toml`
- **Local Commands:** `.gemini/commands/*.toml`
- **Format:** TOML files (one per command)
- **Command Name:** Filename becomes the command (e.g., `plan.toml` → `/plan`)
- **Namespacing:** Subdirectories create namespaced commands (e.g., `git/commit.toml` → `/git:commit`)
- **Structure:**
  ```toml
  description = "Creates a strategic plan"
  prompt = """
  Your primary role is that of a strategist...
  {{args}}
  """
  ```
- **Arguments:**
  - `{{args}}`: All arguments
  - `!{command}`: Shell command execution

### Skills

- **Global Skills:** `~/.gemini/skills/<skill-name>/SKILL.md`
- **Local Skills:** `.gemini/skills/<skill-name>/SKILL.md`
- **Agents-Alias:** `~/.agents/skills/<skill-name>/SKILL.md`, `.agents/skills/<skill-name>/SKILL.md`
- **Extension Skills:** Bundled within installed extensions
- **Format:** Markdown with YAML frontmatter (Agent Skills standard)
- **Frontmatter:**
  - `name`: Required, skill identifier
  - `description`: Required, used by agent to determine relevance
- **Structure:**
  ```
  my-skill/
  ├── SKILL.md          # Required: instructions + metadata
  ├── scripts/          # Optional: executable code
  ├── references/       # Optional: documentation
  └── assets/           # Optional: templates, resources
  ```
- **Discovery Tiers:**
  1. Workspace: `.gemini/skills/` or `.agents/skills/`
  2. User: `~/.gemini/skills/` or `~/.agents/skills/`
  3. Extension: Skills bundled in extensions
- **Precedence:** Workspace > User > Extension
- **Activation:** Agent calls `activate_skill` tool when task matches description
- **Management:** Via `/skills` slash command or `gemini skills` CLI:
  - `gemini skills list`: Show discovered skills
  - `gemini skills link <path>`: Link skills from directory
  - `gemini skills install <url>`: Install from Git repo
  - `gemini skills disable/enable <name>`: Toggle skills

**Documentation:** https://geminicli.com/docs/cli/skills/

---

## Cursor

### Rules

- **Global Rules:** `~/.cursorrules` or `~/.cursor/rules/*.mdc`
- **Local Rules:** `.cursorrules` or `.cursor/rules/*.mdc`
- **Format:** Markdown (`.mdc` for Cursor-specific format)
- **Behavior:** Local rules override global rules
- **Deprecation Note:** `.cursorrules` is deprecated in favor of `.cursor/rules/*.mdc`

### Custom Commands (Slash Commands)

- **Global Commands:** `~/.cursor/commands/*.md`
- **Local Commands:** `.cursor/commands/*.md`
- **Format:** Plain Markdown
- **Command Name:** Filename becomes the command (e.g., `review-code.md` → `/review-code`)
- **Features:**
  - Simple markdown content
  - Team commands available on Team/Enterprise plans
  - Parameters supported (text after command name)
- **Example:**

  ```markdown
  # Code Review Checklist

  ## Review Categories

  - [ ] Code does what it's supposed to do
  - [ ] Edge cases are handled
  - [ ] Security vulnerabilities checked
  ```

### Custom Commands (Slash Commands) — Cursor

Cursor supports native slash commands via markdown files.

- **Global Commands:** `~/.cursor/commands/*.md`
- **Local Commands:** `.cursor/commands/*.md`
- **Format:** Plain Markdown
- **Command Name:** Filename becomes the command (e.g., `review-code.md` → `/review-code`)

> **Note:** RuleWeaver distributes slash command files to Cursor. However, Cursor does **not** support command stubs (`COMMANDS.md`) — the `supports_command_stubs: false` registry flag applies.

### Skills — Cursor

**Not distributed by RuleWeaver.** Cursor does not support the Agent Skills standard in a way compatible with RuleWeaver's skills distribution engine (`supports_skills: false` in registry). No `SKILL.md` files are written to any Cursor directory.

Cursor has its own skills-like feature in its UI, but it is not interoperable with the Agent Skills standard used by Claude Code, Cline, Gemini, OpenCode, and others. Skills created in RuleWeaver are distributed to other supported tools only.

**Documentation:** https://cursor.com/docs/context/rules

---

## Roo Code

### Rules

- **Global Rules:** `~/.roo/rules/*.md` or `~/.roo/rules-{slug}/*.md`
- **Local Rules:** `.roo/rules/*.md` or `.roo/rules-{slug}/*.md`
- **Format:** Markdown files
- **Behavior:** Local rules override global rules

### Custom Commands (Slash Commands)

- **Global Commands:** `~/.roo/commands/*.md`
- **Local Commands:** `.roo/commands/*.md`
- **Format:** Markdown with YAML frontmatter
- **Command Name:** Filename becomes the command (slugified, lowercase)
- **Frontmatter:**
  - `description`: Appears in command menu
  - `argument-hint`: Shows hint for expected arguments (e.g., `<file-path>`)
  - `mode`: Optional mode slug to switch to before executing
- **Features:**
  - Fuzzy search and autocomplete
  - Project commands override global commands with the same name

### Skills

- **Global Skills:** `~/.roo/skills/<skill-name>/SKILL.md`
- **Local Skills:** `.roo/skills/<skill-name>/SKILL.md`
- **Format:** Markdown with YAML frontmatter
- **Structure:** Directory-based with SKILL.md file
- **Invocation:**
  - Explicit: Via skills menu
  - Implicit: Based on context

**Documentation:** https://docs.roocode.com/features/slash-commands

---

## Antigravity

### Rules

- **Global Rules:** `~/.gemini/antigravity/rules/*.md`
- **Local Rules:** `.agents/rules/*.md`
- **Format:** Markdown files
- **Behavior:** Local rules override global rules

### Custom Commands (Slash Commands)

- **Global Commands:** `~/.gemini/antigravity/global_workflows/*.md`
- **Local Commands:** `.agents/workflows/*.md`
- **Format:** Markdown with YAML frontmatter
- **Command Name:** Filename becomes the command
- **Features:**
  - Similar to Cline workflows
  - YAML frontmatter with description
  - Markdown content with steps

### Skills

- **Global Skills:** `~/.gemini/antigravity/skills/<skill-folder>/SKILL.md`
- **Local Skills:** `.agents/skills/<skill-folder>/SKILL.md`
- **Format:** Markdown with YAML frontmatter (Agent Skills standard)
- **Frontmatter:**
  - `name`: Optional, unique identifier (lowercase, hyphens). Defaults to folder name.
  - `description`: **Required**, clear description of what the skill does and when to use it
- **Structure:**
  ```
  .agents/skills/my-skill/
  ├── SKILL.md          # Main instructions (required)
  ├── scripts/          # Helper scripts (optional)
  ├── examples/         # Reference implementations (optional)
  └── resources/        # Templates and other assets (optional)
  ```
- **Discovery:** Agent sees skill list at conversation start; activates based on context
- **Precedence:** Workspace skills override global skills with same name

**Documentation:** https://antigravity.google/docs/skills

---

## Windsurf

### Rules

- **Global Rules:** `~/.windsurf/rules/*.md` or similar
- **Local Rules:** `.windsurfrules` or `.windsurf/rules/*.md`
- **Format:** Markdown files
- **Behavior:** Local rules override global rules

### Custom Commands (Slash Commands) — Windsurf

- **Status:** Not currently distributed by RuleWeaver.
- Windsurf has `supports_slash_commands: true` in the registry capability flags, but no slash command directory path or file extension is configured. No slash command files are written until Windsurf publishes their slash command directory spec.
- **Alternative:** Use Cascade AI panel with natural language.

### Skills

- **Global Skills:** `~/.windsurf/skills/<skill-name>/SKILL.md`
- **Local Skills:** `.windsurf/skills/<skill-name>/SKILL.md`
- **Format:** Markdown with YAML frontmatter (Agent Skills standard)
- **Frontmatter:**
  - `name`: Required, skill identifier
  - `description`: Required, used by agent to determine relevance
- **Distribution:** RuleWeaver writes `SKILL.md` files to Windsurf's global and local skill directories. Skills with default targeting (`target_adapters: []`) are distributed to Windsurf automatically. Skills with explicit adapter targeting must include `windsurf` to be distributed here.
- **Invocation:** Windsurf loads skills from the configured skill directories and activates them based on context.

---

## Kilo Code

### Rules

- **Global Rules:** `~/.kilocode/rules/*.md` or `~/.kilocode/`
- **Local Rules:** `.kilocode/rules/*.md` or `.kilocode/`
- **Format:** Markdown files
- **Behavior:** Local rules override global rules

### Custom Commands (Slash Commands) — Kilo Code

- **Status:** Not currently distributed by RuleWeaver.
- Kilo Code has `supports_command_stubs: true` in the registry capability flags, but neither a global nor local command directory path is configured (`global_commands_dir: None`, `local_commands_dir: None`). No command stub files are written.
- Kilo Code is a fork of Cline but does not fully implement the Cline workflows feature.
- **Alternative:** Use modes or natural language instructions.

### Skills — Kilo Code

- **Status:** Not currently distributed by RuleWeaver.
- Kilo Code has `supports_skills: true` in the registry capability flags, but neither a global nor local skill directory path is configured (`global_skills_dir: None`, `local_skills_dir: None`). No `SKILL.md` files are written.
- **Action required:** When Kilo Code publishes their skill directory spec, update `PathTemplates` in `src-tauri/src/models/registry.rs` to set `global_skills_dir` and `local_skills_dir`. Skills will then be automatically distributed on the next reconcile.

---

## Codex

### Rules

- **Global Rules:** `~/.codex/rules/` or via config
- **Local Rules:** `.codex/rules/` or AGENTS.md
- **Format:** Markdown files or config-based
- **Behavior:** Rules can be configured in `~/.codex/config.toml`

### Custom Commands (Slash Commands)

- **Status:** Codex uses **Skills** instead of custom commands
- **Deprecated:** Custom prompts (`~/.codex/prompts/*.md`) are deprecated in favor of Skills

### Skills

- **Global Skills:** `~/.agents/skills/<skill-name>/SKILL.md`
- **Local Skills:** `.agents/skills/<skill-name>/SKILL.md` (project-level, can be nested)
- **Repository Skills:** `$CWD/.agents/skills/<skill-name>/SKILL.md`
- **Admin Skills:** `/etc/codex/skills/<skill-name>/SKILL.md`
- **Format:** Markdown with YAML frontmatter (Agent Skills standard)
- **Structure:**
  ```
  my-skill/
  ├── SKILL.md (required)
  ├── scripts/ (optional)
  ├── references/ (optional)
  ├── assets/ (optional)
  └── agents/
      └── openai.yaml (optional)
  ```
- **Frontmatter:**
  ```yaml
  ---
  name: skill-name
  description: Explain exactly when this skill should trigger
  ---
  ```
- **Invocation:**
  - Explicit: `/skills` or type `$skill-name`
  - Implicit: Based on description matching
- **UI Metadata:** Optional `agents/openai.yaml` for Codex app integration
  ```yaml
  interface:
    display_name: "Skill Name"
    short_description: "What this skill does"
    icon_small: "./assets/icon.svg"
    brand_color: "#3B82F6"
  policy:
    allow_implicit_invocation: true
  dependencies:
    tools:
      - type: "mcp"
        value: "server-name"
  ```

**Documentation:** https://developers.openai.com/codex/skills

---

## Arguments in Custom Commands

Different AI tools handle command arguments differently. Here's a comprehensive guide:

### OpenCode

**Syntax:** `$ARGUMENTS`, `$1`, `$2`, ... `$9`

- `$ARGUMENTS`: All arguments as a space-separated string
- `$1` - `$9`: Positional arguments
- `$$`: Literal dollar sign
- ``!`command` ``: Shell command injection (backtick-wrapped)
- `@filename`: File reference (content inserted)

**Example:**

```markdown
---
name: deploy
description: Deploy to a specific environment
---

Deploy to: $1

Full command: ./scripts/deploy.sh $ARGUMENTS

Current git branch: !`git branch --show-current`
```

**Usage:** `/deploy staging --force`

### Claude Code

**Syntax:** `$ARGUMENTS`, `$1`, `$2`, ... `$9`

- `$ARGUMENTS`: All arguments as a space-separated string
- `$1` - `$9`: Positional arguments
- Named variables: Define in frontmatter (e.g., `$ENVIRONMENT`)

**Example:**

```markdown
---
name: deploy
description: Deploy to environment
tools:
  - bash
---

Deploy to $1 using the $ARGUMENTS flags.
```

**Usage:** `/deploy staging --force`

### Cline

**Syntax:** None (natural language)

- Cline does not support argument substitution in workflows
- Arguments are passed as natural language after the command
- Use named entities in instructions

**Example:**

```markdown
# Deploy

Ask the user which environment to deploy to if not specified.
Then run the deployment script with the appropriate settings.
```

**Usage:** `/deploy staging` ("staging" appears in prompt as natural language)

### Gemini CLI

**Syntax:** `{{args}}`

- `{{args}}`: All arguments as a single string
- Shell execution: `!{command}` (exclamation-brace syntax)

**Example:**

```toml
description = "Run tests"
prompt = """
Run the test suite with these arguments: {{args}}

Current branch: !{git branch --show-current}
"""
```

**Usage:** `/test --watch --coverage`

### Cursor

**Syntax:** None (automatic)

- Cursor automatically includes all text after the command name
- Arguments appear as natural language in the prompt
- No special substitution syntax needed

**Example:**

```markdown
# Review Code

Review the code changes for:

- Logic errors
- Style violations
- Security issues
```

**Usage:** `/review-code src/components/Button.tsx` (file path appears in prompt)

### Roo Code

**Syntax:** Frontmatter-defined

- `argument-hint`: Shows expected arguments in UI (e.g., `<file-path>`)
- Arguments passed as natural language after command

**Example:**

```markdown
---
name: lint
description: Run linter on files
argument-hint: <files...>
---

Run eslint on the specified files.
```

**Usage:** `/lint src/**/*.ts` (arguments appear as text)

### Antigravity

**Syntax:** None (natural language)

- Arguments passed as natural language after command
- No substitution syntax available

### Codex

**Syntax:** `$ARGUMENTS`, `$1`, `$2`, ... `$9`, named variables

- `$ARGUMENTS`: All arguments
- `$1` - `$9`: Positional arguments
- Named: `$FILES`, `$PR_TITLE`, etc.
- `$$`: Literal dollar sign

**Example (Custom Prompts - Deprecated):**

```markdown
---
description: Create a PR
argument-hint: [FILES=<paths>] [PR_TITLE="<title>"]
---

Create PR for: $FILES
Title: $PR_TITLE
```

**Note:** Skills (preferred) don't use argument substitution - they're context-based.

---

## Comparison Summary

### Rules Handling

| Tool        | Global Rules                                | Local Rules                             | Format          |
| ----------- | ------------------------------------------- | --------------------------------------- | --------------- |
| OpenCode    | `~/.config/opencode/rules/*.md`             | `.opencode/rules/*.md`                  | Markdown + YAML |
| Claude Code | `~/.claude/CLAUDE.md`                       | `.claude/CLAUDE.md`                     | Markdown        |
| Cline       | `~/Documents/Cline/Rules/*.md`              | `.clinerules/*.md`                      | Markdown        |
| Gemini      | `~/.gemini/GEMINI.md`                       | `.gemini/GEMINI.md`                     | Markdown        |
| Cursor      | `~/.cursorrules` or `~/.cursor/rules/*.mdc` | `.cursorrules` or `.cursor/rules/*.mdc` | Markdown/MDC    |
| Roo Code    | `~/.roo/rules/*.md`                         | `.roo/rules/*.md`                       | Markdown        |
| Antigravity | `~/.gemini/antigravity/rules/*.md`          | `.agents/rules/*.md`                    | Markdown        |
| Windsurf    | `~/.windsurf/rules/*.md`                    | `.windsurfrules`                        | Markdown        |
| Codex       | Config-based                                | `.codex/rules/` or AGENTS.md            | Markdown/Config |

### Custom Commands Handling

| Tool        | Global Commands                               | Local Commands               | Format          | Arguments                |
| ----------- | --------------------------------------------- | ---------------------------- | --------------- | ------------------------ |
| OpenCode    | `~/.config/opencode/commands/*.md`            | `.opencode/commands/*.md`    | Markdown + YAML | `$ARGUMENTS`, `$1`, `$2` |
| Claude Code | `~/.claude/commands/*.md`                     | `.claude/commands/*.md`      | Markdown + YAML | `$ARGUMENTS`, `$1`, `$2` |
| Cline       | `~/Documents/Cline/Workflows/*.md`            | `.clinerules/workflows/*.md` | Markdown        | None (natural language)  |
| Gemini      | `~/.gemini/commands/*.toml`                   | `.gemini/commands/*.toml`    | TOML            | `{{args}}`               |
| Cursor      | `~/.cursor/commands/*.md`                     | `.cursor/commands/*.md`      | Markdown        | Text after command       |
| Roo Code    | `~/.roo/commands/*.md`                        | `.roo/commands/*.md`         | Markdown + YAML | Via frontmatter          |
| Antigravity | `~/.gemini/antigravity/global_workflows/*.md` | `.agents/workflows/*.md`     | Markdown + YAML | None (natural language)  |
| Windsurf    | ❌ Not Supported                              | ❌ Not Supported             | N/A             | N/A                      |
| Codex       | ❌ Deprecated                                 | ❌ Deprecated                | N/A             | N/A                      |

### Skills Handling

| Tool        | Skills Support          | Global Skills                             | Local Skills                  | Format       |
| ----------- | ----------------------- | ----------------------------------------- | ----------------------------- | ------------ |
| OpenCode    | ✅ Yes                  | `~/.config/opencode/skills/*/SKILL.md`    | `.opencode/skills/*/SKILL.md` | Agent Skills |
| Claude Code | ✅ Yes                  | `~/.claude/skills/*/SKILL.md`             | `.claude/skills/*/SKILL.md`   | Agent Skills |
| Cline       | ✅ Yes (Exp.)           | `~/.cline/skills/*/SKILL.md`              | `.cline/skills/*/SKILL.md`    | Agent Skills |
| Gemini      | ✅ Yes                  | `~/.gemini/skills/*/SKILL.md`             | `.gemini/skills/*/SKILL.md`   | Agent Skills |
| Cursor      | ❌ No (not distributed) | N/A                                       | N/A                           | N/A          |
| Roo Code    | ✅ Yes                  | `~/.roo/skills/*/SKILL.md`                | `.roo/skills/*/SKILL.md`      | Agent Skills |
| Antigravity | ✅ Yes                  | `~/.gemini/antigravity/skills/*/SKILL.md` | `.agents/skills/*/SKILL.md`   | Agent Skills |
| Windsurf    | ✅ Yes                  | `~/.windsurf/skills/*/SKILL.md`           | `.windsurf/skills/*/SKILL.md` | Agent Skills |
| Kilo Code   | ❌ No (paths pending)   | N/A                                       | N/A                           | N/A          |
| Codex       | ✅ Yes                  | `~/.agents/skills/*/SKILL.md`             | `.agents/skills/*/SKILL.md`   | Agent Skills |

---

## Key Takeaways

1. **Universal Pattern:** Most tools follow the same pattern: global config in home directory, local config in project root
2. **Precedence:** Local configurations always override global configurations
3. **Agent Skills Standard:** Claude Code, Roo Code, Codex, Cursor, OpenCode, Cline, Gemini, and Antigravity all use the Agent Skills standard
4. **No Command Stubs:** Cursor, Windsurf, and Kilo Code do not receive `COMMANDS.md` command stub files. Cursor supports slash command files (`.md` in `.cursor/commands/`) but not the `COMMANDS.md` format. Windsurf and Kilo Code have no configured command paths at this time.
5. **Skills Preferred:** Codex has deprecated custom prompts in favor of Skills
6. **TOML Exception:** Gemini CLI uniquely uses TOML files for commands instead of Markdown
7. **Argument Substitution:**
   - **Positional:** OpenCode, Claude Code, Codex (`$1`, `$2`, `$ARGUMENTS`)
   - **Template:** Gemini CLI (`{{args}}`)
   - **Natural Language:** Cline, Cursor, Roo Code, Antigravity (no substitution)
8. **Shell Injection:** OpenCode (``!`cmd` ``), Gemini CLI (`!{cmd}`)
9. **File References:** OpenCode supports `@filename` for file content injection
10. **Most Tools Support Skills:** 8 out of 10 tools have skills distributed by RuleWeaver. Cursor does not support the Agent Skills standard for distribution (`supports_skills: false`). Kilo Code has the capability flag set but no paths configured yet — will be enabled when they publish their directory spec.
