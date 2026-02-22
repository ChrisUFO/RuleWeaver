# Project Strategy: RuleWeaver Phase 2 - Custom Commands & MCP

## 1. High-Level Strategy

**Objective:** Transform RuleWeaver from a rule management tool into a complete MCP (Model Context Protocol) server platform. Users will be able to define custom shell commands and expose them as MCP tools to AI assistants like Claude Code and OpenCode.

**Core Deliverables:**

1. **File-First Storage Architecture (#14)** - Migrate from SQLite-primary to markdown files with YAML frontmatter for git-native, transparent rule storage
2. **MCP Server Foundation (#5)** - Embed an MCP server in the Tauri app that starts automatically and serves tools
3. **Command Management System (#6)** - Full CRUD for custom commands with script templates, arguments, and in-app testing
4. **Command Execution Engine (#7)** - Wire commands to MCP server with secure process spawning and output capture

**Architecture Alignment:**
This plan implements Phase 2 of the roadmap: "Custom Commands MVP & MCP Integration". It follows the three-layer architecture (Presentation Layer, Core Logic Layer, Target Layer) defined in `architecture.md`. Note: Skills are defined in the architecture but will be implemented in Phase 3 (not part of this milestone).

**Strategic Approach:**

- **Issue #14 first** - File-first storage is marked high priority and provides a better foundation for the rest of Phase 2
- **Parallel tracks** - MCP Server (#5) and Command Model (#6) can proceed in parallel after #14
- **Integration** - Issue #7 ties everything together, connecting commands to the MCP server

---

## 2. Implementation Plan

### Phase 2.0: Project Setup (PREREQUISITE)

**Goal:** Create feature branch and ensure all tooling is ready.

- [ ] Create feature branch: `git checkout -b feature/phase-2-mcp-commands`
- [ ] Verify all Phase 1 tests pass
- [ ] Run lint and typecheck to establish baseline
- [ ] Review and update any outdated dependencies
- [ ] Ensure CI/CD pipeline is green

---

### Phase 2.1: File-First Storage Architecture (Issue #14) - HIGH PRIORITY

**Goal:** Replace SQLite as primary storage with markdown files containing YAML frontmatter, keeping SQLite only for indexing/cache.

**Why This First:**

- Rules become git-versionable automatically
- Users can edit `.ruleweaver/rules/*.md` in any editor
- Portable, transparent, emergency-accessible
- Better developer experience (review rule changes in PRs)
- Aligns with architecture.md: "Database: SQLite (Embedded via Rust) or `serde_json` file storage"

#### Backend Implementation

**File Structure:**

```
~/.ruleweaver/rules/              # Global rules
  ├── 000-global-coding.md
  └── 001-security-guidelines.md

{repo}/.ruleweaver/rules/         # Local rules (per project)
  ├── 000-project-specific.md
  └── ...
```

**File Format (YAML + Markdown):**

```markdown
---
id: abc-123
name: "TypeScript Rules"
scope: local
targetPaths: ["/src"]
enabledAdapters: [gemini, opencode, cline]
enabled: true
createdAt: 2024-01-15T10:30:00Z
updatedAt: 2024-01-15T10:30:00Z
---

## TypeScript Guidelines

- Use strict mode
- Prefer interfaces over types
```

**Rust Module Structure:**

```
src-tauri/src/
├── file_storage/
│   ├── mod.rs           # Public API
│   ├── parser.rs        # YAML frontmatter parsing
│   ├── serializer.rs    # YAML frontmatter generation
│   ├── watcher.rs       # File system watching
│   └── migration.rs     # SQLite to files migration
```

**Rust Implementation Tasks:**

- Add dependencies to `Cargo.toml`: `serde_yaml`, `notify` (file watcher), `glob`
- Create `src-tauri/src/file_storage/mod.rs` module
- Implement YAML frontmatter parser with error handling
- Implement YAML frontmatter generator
- Create `RuleFile` struct for file-based rule representation
- Implement `load_rules_from_disk()` function with directory traversal
- Implement `save_rule_to_disk()` function with atomic writes
- Handle global rules directory (`~/.ruleweaver/rules/`)
- Handle local rules directories (`{repo}/.ruleweaver/rules/`)
- Create file watcher using `notify` crate for external changes
- Update SQLite schema - migration v3 to remove content field
- Keep SQLite as index/cache (id, file_path, content_hash, last_sync_at, modified_at)
- Implement backward compatibility (fallback to SQLite if files don't exist)
- Create `migrate_to_file_storage()` IPC command with progress reporting
- Create `verify_file_storage_integrity()` health check
- Implement `rollback_migration()` for safety
- Update `Database` to work with file-based rules
- Update all existing IPC commands to use file storage
- Handle edge cases:
  - File permission errors (readable/writable check)
  - Concurrent editing (file vs app) with conflict detection
  - Large files (>1MB) with chunked reading
  - Special characters in rule content (Unicode handling)
  - Duplicate IDs across different directories
  - Orphaned files (no matching rule in index)
  - Git merge conflicts in rule files

#### Frontend Implementation

**New UI Components:**

- `FileStorageStatus` - Shows current storage mode
- `MigrationDialog` - Migration progress and status
- `FilePathDisplay` - Shows file path with copy button
- `RevealInExplorerButton` - Opens file location

**Frontend Tasks:**

- Add "Open in Editor" button to rule cards
- Show file path in rule detail view
- Display file timestamps (created/modified)
- Implement "Reveal in Explorer" for rule files
- Add file system health indicators
- Show migration status/progress dialog
- Add storage mode indicator in settings
- Create migration UI with:
  - Progress bar
  - Current file being migrated
  - Error display
  - Rollback option

#### Error Handling

| Error Type         | User Message                                                       | Recovery                                 |
| ------------------ | ------------------------------------------------------------------ | ---------------------------------------- |
| File permissions   | "Cannot read/write rule file at {path}. Check folder permissions." | "Open folder" button + "Retry"           |
| YAML parse error   | "Invalid YAML frontmatter in {filename}"                           | "View file" button + "Reset to default"  |
| File not found     | "Rule file not found at expected path"                             | "Locate file" or "Recreate"              |
| Concurrent edit    | "Rule file was modified externally"                                | "Reload from disk" or "Overwrite"        |
| Git merge conflict | "Merge conflict detected in {filename}"                            | Show conflict markers, manual resolution |
| Large file         | "Rule file exceeds size limit (1MB)"                               | "Split into smaller rules"               |
| Orphaned file      | "File has no matching rule in database"                            | "Import" or "Delete file"                |
| Duplicate ID       | "Multiple files have the same rule ID"                             | "Resolve duplicates" wizard              |

#### Documentation Tasks

- [ ] Update README.md with new file-based architecture
- [ ] Document `.ruleweaver/rules/` directory structure
- [ ] Document YAML frontmatter schema
- [ ] Add migration guide for existing users
- [ ] Document file watching behavior
- [ ] Document manual editing best practices

#### Tests

**Unit Tests (Rust):**

- [ ] `test_yaml_frontmatter_parse_success` - Happy path
- [ ] `test_yaml_frontmatter_parse_invalid_yaml` - Failure path
- [ ] `test_yaml_frontmatter_parse_missing_required_fields` - Validation
- [ ] `test_yaml_frontmatter_generate` - Serialization
- [ ] `test_load_rules_from_disk_empty_directory` - Edge case
- [ ] `test_load_rules_from_disk_mixed_files` - Multiple rules
- [ ] `test_load_rules_from_disk_special_characters` - Unicode content
- [ ] `test_save_rule_to_disk_atomic_write` - File safety
- [ ] `test_save_rule_to_disk_overwrite_existing` - Update scenario
- [ ] `test_file_watcher_detects_external_change` - External edits
- [ ] `test_migration_preserves_all_data` - Data integrity
- [ ] `test_migration_rollback` - Safety mechanism
- [ ] `test_backward_compatibility_sqlite_fallback` - Compatibility
- [ ] `test_concurrent_edit_detection` - Race conditions

**Integration Tests:**

- [ ] `test_full_migration_flow` - End-to-end migration
- [ ] `test_file_watcher_integration` - Real file system events
- [ ] `test_git_workflow_compatibility` - Commit, pull, merge

**Frontend Tests:**

- [ ] Test MigrationDialog renders progress correctly
- [ ] Test FileStorageStatus shows correct state
- [ ] Test "Open in Editor" button functionality

**Coverage Target:** 80%+ (high-value tests only, no trivial getter/setter tests)

---

### Phase 2.2: MCP Server Foundation (Issue #5)

**Goal:** Research, integrate, and boot an MCP server within the Tauri application.

#### Research & Architecture

**MCP Protocol Basics:**

- JSON-RPC 2.0 over stdio or HTTP
- Key endpoints: `tools/list` (returns available tools), `tools/call` (executes tool)
- Transport: Start with stdio (simpler), consider HTTP later

**Rust SDK Options to Evaluate:**

1. `rmcp` crate (if available)
2. Custom JSON-RPC implementation using `tokio` and `tower`
3. `jsonrpc-core` crate

**Selection Criteria:**

- Active maintenance
- Tauri compatibility
- Async support
- Error handling quality

#### Backend Implementation

**Rust Module Structure:**

```
src-tauri/src/
├── mcp/
│   ├── mod.rs           # Public API
│   ├── server.rs        # Server lifecycle
│   ├── protocol.rs      # JSON-RPC types
│   ├── handlers.rs      # Endpoint handlers
│   └── transport.rs     # Stdio/HTTP transport
```

**Server Implementation:**

```rust
pub struct McpServer {
    port: u16,
    running: Arc<AtomicBool>,
    commands: Arc<RwLock<Vec<Command>>>,
    transport: Box<dyn Transport>,
}

impl McpServer {
    pub fn new(port: u16) -> Result<Self>;
    pub async fn start(&self) -> Result<()>;
    pub async fn stop(&self) -> Result<()>;
    pub fn is_running(&self) -> bool;
    pub fn get_status(&self) -> ServerStatus;
}

pub enum ServerStatus {
    Running { port: u16, uptime: Duration },
    Stopped,
    Error(String),
    Starting,
}
```

**Tasks:**

- Add `tokio` and `serde_json` dependencies to `Cargo.toml`
- Evaluate and integrate chosen MCP SDK (or custom impl)
- Create `src-tauri/src/mcp/mod.rs` module
- Define JSON-RPC request/response types
- Implement `McpServer` struct with state management
- Implement `start()` method with configurable port binding
- Implement `stop()` method with graceful shutdown
- Implement basic `tools/list` endpoint (empty list initially)
- Implement `initialize` endpoint (MCP protocol handshake)
- Add server status tracking (running/stopped/error/starting)
- Handle graceful shutdown on app close (Tauri lifecycle hook)
- Add MCP server configuration persistence (port, auto-start)
- Implement stdio transport layer
- Implement connection logging

**IPC Commands:**

- `get_mcp_status` - Returns server status, port, uptime
- `start_mcp_server` - Manually start server (returns error if already running)
- `stop_mcp_server` - Manually stop server
- `restart_mcp_server` - Stop then start
- `get_mcp_connection_instructions` - Get setup instructions for clients
- `get_mcp_logs` - Get recent connection logs

**Configuration:**

```rust
pub struct McpConfig {
    pub enabled: bool,
    pub port: u16,
    pub auto_start: bool,
    pub log_level: LogLevel,
    pub max_connections: u32,
}
```

#### Frontend Implementation

**New UI Components:**

- `McpStatusIndicator` - Green/red status light with tooltip
- `McpConnectionPanel` - Full connection details
- `ConnectionInstructions` - Copy-able config snippets
- `McpSettingsSection` - Configuration UI
- `McpLogsViewer` - Connection history

**MCP Status Page:**

- Server status indicator (Running/Stopped/Error/Starting)
- Current port display
- Uptime counter
- Connection count (active clients)
- Recent activity log
- Start/Stop/Restart buttons
- Error message display

**Connection Instructions:**

- Claude Code config (`claude_desktop_config.json`):
  ```json
  {
    "mcpServers": {
      "ruleweaver": {
        "command": "ruleweaver-mcp",
        "args": ["--port", "8080"]
      }
    }
  }
  ```
- OpenCode config (`~/.opencode/config.json`):
  ```json
  {
    "mcp": {
      "servers": [
        {
          "name": "ruleweaver",
          "url": "http://localhost:8080"
        }
      ]
    }
  }
  ```
- Copy buttons for each config
- Step-by-step setup wizard

**Settings Page Updates:**

- MCP Server section with:
  - Enable/disable toggle
  - Port input with validation (1024-65535)
  - Port conflict detection
  - Auto-start on app launch toggle
  - Max connections setting
  - Log level selector
  - Connection instructions link

#### Error Handling

| Error Type        | User Message                                           | Recovery                               |
| ----------------- | ------------------------------------------------------ | -------------------------------------- |
| Port in use       | "Port {port} is already in use by another application" | "Use different port" suggestion        |
| Bind error        | "Cannot bind to {port}. Check firewall settings."      | "Try different port" or "Run as admin" |
| Protocol error    | "Invalid MCP message received"                         | Log error, continue                    |
| Transport error   | "Connection error: {details}"                          | Auto-restart server                    |
| Client disconnect | Silent (expected)                                      | None                                   |

#### Documentation Tasks

- [ ] Document MCP protocol support
- [ ] Document connection setup for Claude Code
- [ ] Document connection setup for OpenCode
- [ ] Document MCP troubleshooting
- [ ] Add MCP architecture diagram

#### Tests

**Unit Tests (Rust):**

- [ ] `test_server_initialization` - Happy path
- [ ] `test_server_start_success` - Server starts
- [ ] `test_server_stop_success` - Server stops gracefully
- [ ] `test_server_double_start_fails` - Error handling
- [ ] `test_server_port_already_in_use` - Port conflict
- [ ] `test_jsonrpc_initialize_request` - Protocol handshake
- [ ] `test_jsonrpc_tools_list_request` - Tools endpoint
- [ ] `test_jsonrpc_invalid_request` - Error handling
- [ ] `test_graceful_shutdown` - Cleanup
- [ ] `test_status_tracking` - State machine

**Integration Tests:**

- [ ] `test_mcp_server_lifecycle` - Full start/stop flow
- [ ] `test_mcp_client_connection` - Real client connection
- [ ] `test_concurrent_client_connections` - Multi-client
- [ ] `test_port_conflict_handling` - Real port binding

**Frontend Tests:**

- [ ] Test McpStatusIndicator shows correct colors
- [ ] Test ConnectionInstructions copy buttons work
- [ ] Test port validation in settings

---

### Phase 2.3: Command Management System (Issue #6)

**Goal:** Create full CRUD system for custom commands with script templates, dynamic arguments, and in-app testing.

#### Backend Implementation

**Data Model:**

```rust
// src-tauri/src/models/command.rs
pub struct Command {
    pub id: String,
    pub name: String,
    pub description: String,
    pub script: String,           // Template with {{arg}} placeholders
    pub arguments: Vec<CommandArgument>,
    pub expose_via_mcp: bool,
    pub working_directory: Option<String>,  // Optional cwd
    pub environment_variables: HashMap<String, String>,  // Optional env vars
    pub timeout_seconds: u32,     // Default 60
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct CommandArgument {
    pub name: String,
    pub description: String,
    pub arg_type: ArgumentType,   // string, number, boolean, enum
    pub required: bool,
    pub default_value: Option<String>,
    pub enum_values: Option<Vec<String>>,  // For enum type
}

pub enum ArgumentType {
    String,
    Number,
    Boolean,
    Enum(Vec<String>),
}
```

**File Storage (YAML + Markdown, same pattern as Rules):**

```
~/.ruleweaver/commands/
  ├── 000-format-code.md
  └── 001-run-tests.md
```

**File Format:**

```markdown
---
id: cmd-123
name: "Format Code"
description: "Run prettier on the specified files"
script: "npx prettier --write {{files}}"
arguments:
  - name: files
    description: "Files to format"
    type: string
    required: true
exposeViaMcp: true
workingDirectory: null
environmentVariables: {}
timeoutSeconds: 60
createdAt: 2024-01-15T10:30:00Z
updatedAt: 2024-01-15T10:30:00Z
---

Additional documentation about this command...
```

**Rust Module Structure:**

```
src-tauri/src/
├── commands/
│   ├── mod.rs           # Existing rule commands
│   └── command_commands.rs  # New command CRUD
├── models/
│   ├── mod.rs
│   ├── rule.rs
│   └── command.rs       # New
```

**Tasks:**

- Create `src-tauri/src/models/command.rs` module
- Define `Command` and `CommandArgument` structs with serde
- Define `ArgumentType` enum with JSON Schema conversion
- Add `commands` table to SQLite (for index/cache only)
- Implement `load_commands_from_disk()` using file storage
- Implement `save_command_to_disk()` with YAML frontmatter
- Create commands directory structure (`~/.ruleweaver/commands/`)
- Implement IPC command: `get_all_commands`
- Implement IPC command: `get_command_by_id`
- Implement IPC command: `create_command`
- Implement IPC command: `update_command`
- Implement IPC command: `delete_command`
- Implement script template parser (find `{{arg}}` placeholders)
- Implement template argument validator (ensure all args have definitions)
- Implement script injection prevention:
  - Escape shell metacharacters in arguments
  - Validate argument values against type
  - Block dangerous patterns (rm -rf, etc.)
- Implement IPC command: `test_command` with process spawn
- Implement timeout handling for test execution
- Capture stdout/stderr with size limits (max 10MB)
- Return structured test result with exit code, output, duration

**Script Template Engine:**

```rust
pub struct TemplateEngine;

impl TemplateEngine {
    pub fn extract_placeholders(script: &str) -> Vec<String>;
    pub fn validate_template(script: &str, args: &[CommandArgument]) -> Result<()>;
    pub fn render_script(script: &str, values: &HashMap<String, String>) -> Result<String>;
    pub fn escape_shell_arg(value: &str) -> String;
    pub fn detect_dangerous_patterns(script: &str) -> Vec<String>;
}
```

**Security Measures:**

- Escape all arguments before shell execution
- Validate argument types (string, number, boolean, enum)
- Block dangerous patterns: `rm -rf`, `> /dev/null`, `|`, `;`, `&&`, `` ` ``, `$()`
- Timeout all executions
- Size limit on output (prevent memory exhaustion)

#### Frontend Implementation

**New UI Components:**

- `CommandsPage` - Main commands list
- `CommandCard` - Individual command display
- `CommandEditor` - Full editor
- `ScriptEditor` - Code editor for scripts
- `ArgumentBuilder` - Dynamic argument form builder
- `ArgumentTypeSelector` - Type selection dropdown
- `McpSchemaPreview` - JSON schema preview
- `CommandTester` - In-app testing panel
- `TestOutputViewer` - Stdout/stderr display
- `ExecutionTimeDisplay` - Duration formatting

**Commands Page:**

- Similar layout to Rules page
- List view with: name, description, MCP badge, argument count, last modified
- Search bar with debounced filtering
- Sort dropdown: Name (A-Z, Z-A), Date Modified, Has Arguments
- Filter chips: Exposed via MCP, Has Arguments, Recently Used
- Enable/disable toggle per command
- Quick test button (opens test panel)
- Overflow menu: Edit, Duplicate, Delete, View History
- Bulk selection with bulk actions

**Command Editor:**

- **Header:** Name input, description textarea
- **Script Section:**
  - Monaco/CodeMirror editor with shell syntax highlighting
  - Placeholder highlighting (`{{arg}}` in different color)
  - Line numbers
  - Placeholder validation (underlines undefined placeholders)
- **Arguments Section:**
  - Dynamic list of argument definitions
  - Add/Remove/Drag to reorder
  - Each argument card:
    - Name input (must match placeholder)
    - Description textarea
    - Type selector (String, Number, Boolean, Enum)
    - Required checkbox
    - Default value input (type-specific)
    - Enum values textarea (for enum type)
- **Settings Section:**
  - MCP exposure toggle
  - Working directory input (optional)
  - Environment variables key-value pairs
  - Timeout slider (5-300 seconds)
- **MCP Schema Preview Panel:**
  - Read-only JSON preview of generated tool schema
  - Copy button
  - Validates in real-time
- **Test Panel:**
  - Input fields for each argument (dynamic based on type)
  - "Test Run" button with loading state
  - Output display:
    - Exit code badge (green for 0, red for non-zero)
    - Execution time
    - Stdout tab (with syntax highlighting if detectable)
    - Stderr tab
    - Copy output buttons
  - Clear output button

**TypeScript Types:**

```typescript
// types/command.ts
export type ArgumentType = "string" | "number" | "boolean" | "enum";

export interface CommandArgument {
  name: string;
  description: string;
  type: ArgumentType;
  required: boolean;
  defaultValue?: string;
  enumValues?: string[];
}

export interface Command {
  id: string;
  name: string;
  description: string;
  script: string;
  arguments: CommandArgument[];
  exposeViaMcp: boolean;
  workingDirectory: string | null;
  environmentVariables: Record<string, string>;
  timeoutSeconds: number;
  createdAt: number;
  updatedAt: number;
}

export interface TestCommandResult {
  success: boolean;
  exitCode: number;
  stdout: string;
  stderr: string;
  durationMs: number;
  error?: string;
}
```

**State Management:**

```typescript
// stores/commandsStore.ts
interface CommandsState {
  commands: Command[];
  selectedCommand: Command | null;
  isLoading: boolean;
  error: string | null;
  testResults: Map<string, TestCommandResult>;

  fetchCommands: () => Promise<void>;
  createCommand: (input: CreateCommandInput) => Promise<void>;
  updateCommand: (id: string, input: UpdateCommandInput) => Promise<void>;
  deleteCommand: (id: string) => Promise<void>;
  testCommand: (id: string, args: Record<string, unknown>) => Promise<TestCommandResult>;
  selectCommand: (command: Command | null) => void;
}
```

#### Sync Adapter Updates for Commands

Extend `SyncAdapter` trait:

```rust
pub trait SyncAdapter: Send + Sync {
    // Existing rule methods...

    // New command methods
    fn format_commands_content(&self, commands: &[Command]) -> String;
    fn get_command_file_name(&self) -> &str;
    fn get_command_global_path(&self) -> Result<PathBuf>;
}
```

**Command Sync Output:**

Gemini CLI (`~/.gemini/COMMANDS.toml`):

```toml
# Generated by RuleWeaver - Do not edit manually
# Last synced: 2024-01-15T10:30:00Z

[[command]]
name = "format-code"
description = "Run prettier on files"
script = "npx prettier --write {{files}}"

[command.arguments]
files = { type = "string", required = true }
```

Claude Code (`~/.claude/COMMANDS.md`):

```markdown
<!-- Generated by RuleWeaver - Do not edit manually -->
<!-- Last synced: 2024-01-15T10:30:00Z -->

## format-code

**Description:** Run prettier on files

**Usage:**
```

/format-code files=<path>

```

**Arguments:**
- `files` (string, required): Files to format
```

OpenCode (`~/.opencode/COMMANDS.md`):

```markdown
# RuleWeaver Commands

## format-code

Run prettier on files.

**Command:** `npx prettier --write {{files}}`

**Parameters:**

- `files` (string, required)
```

**Tasks:**

- Extend `SyncAdapter` trait with command methods
- Implement `format_commands_content()` for each adapter
- Implement `get_command_file_name()` for each adapter
- Implement `get_command_global_path()` for each adapter
- Update `SyncEngine` to sync commands alongside rules
- Add command file generation to sync process

#### Error Handling

| Error Type                | User Message                                       | Recovery                  |
| ------------------------- | -------------------------------------------------- | ------------------------- |
| Template syntax error     | "Invalid placeholder syntax: {{invalid}}"          | Highlight error location  |
| Undefined placeholder     | "Placeholder {{arg}} not defined in arguments"     | "Add argument" quick fix  |
| Unused argument           | "Argument 'foo' is defined but not used in script" | Warning only              |
| Script injection detected | "Dangerous pattern detected in script"             | Block save, show patterns |
| Type mismatch             | "Expected string, got number for argument 'foo'"   | Validation error          |
| Timeout                   | "Command exceeded timeout of {X}s"                 | Show partial output       |
| Command not found         | "Script interpreter not found"                     | Check PATH settings       |
| Permission denied         | "Cannot execute script: permission denied"         | "Open folder" button      |

#### Documentation Tasks

- [ ] Document Command file format (YAML frontmatter)
- [ ] Document script template syntax
- [ ] Document available argument types
- [ ] Document security restrictions
- [ ] Add command examples and best practices
- [ ] Document MCP tool naming conventions

#### Tests

**Unit Tests (Rust):**

- [ ] `test_command_serialization` - Round-trip
- [ ] `test_extract_placeholders_simple` - Basic case
- [ ] `test_extract_placeholders_multiple` - Multiple args
- [ ] `test_extract_placeholders_nested` - Complex case
- [ ] `test_validate_template_success` - Happy path
- [ ] `test_validate_template_undefined_placeholder` - Error case
- [ ] `test_render_script_success` - Template rendering
- [ ] `test_render_script_escaping` - Injection prevention
- [ ] `test_escape_shell_arg_special_chars` - Security
- [ ] `test_detect_dangerous_patterns_rm_rf` - Security
- [ ] `test_detect_dangerous_patterns_pipes` - Security
- [ ] `test_test_command_success` - Happy path
- [ ] `test_test_command_failure` - Non-zero exit
- [ ] `test_test_command_timeout` - Timeout handling
- [ ] `test_test_command_large_output` - Size limits
- [ ] `test_crud_create_command` - Create
- [ ] `test_crud_update_command` - Update
- [ ] `test_crud_delete_command` - Delete
- [ ] `test_adapter_format_commands_gemini` - Gemini format
- [ ] `test_adapter_format_commands_opencode` - OpenCode format

**Integration Tests:**

- [ ] `test_command_full_lifecycle` - Create, test, delete
- [ ] `test_command_sync_to_files` - File generation
- [ ] `test_command_security_boundaries` - Injection attempts
- [ ] `test_concurrent_command_editing` - Race conditions

**Frontend Tests:**

- [ ] Test CommandEditor renders all fields
- [ ] Test placeholder validation in ScriptEditor
- [ ] Test ArgumentBuilder add/remove/reorder
- [ ] Test MCP schema preview updates
- [ ] Test CommandTester executes and displays output
- [ ] Test type-specific input fields

---

### Phase 2.4: Command Execution & MCP Integration (Issue #7)

**Goal:** Wire saved commands to the MCP server's `tools/list` and implement secure `tools/call` execution.

#### Backend Implementation

**MCP Tool Schema Generation:**

```rust
impl Command {
    pub fn to_tool_schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name.to_snake_case(),
            description: self.description.clone(),
            input_schema: self.build_input_schema(),
        }
    }

    fn build_input_schema(&self) -> Value {
        let mut properties = Map::new();
        let mut required = Vec::new();

        for arg in &self.arguments {
            let property = match arg.arg_type {
                ArgumentType::String => json!({
                    "type": "string",
                    "description": arg.description
                }),
                ArgumentType::Number => json!({
                    "type": "number",
                    "description": arg.description
                }),
                ArgumentType::Boolean => json!({
                    "type": "boolean",
                    "description": arg.description
                }),
                ArgumentType::Enum(values) => json!({
                    "type": "string",
                    "description": arg.description,
                    "enum": values
                }),
            };

            properties.insert(arg.name.clone(), property);

            if arg.required {
                required.push(json!(arg.name.clone()));
            }
        }

        json!({
            "type": "object",
            "properties": properties,
            "required": required
        })
    }
}
```

**MCP Server Integration:**

```rust
impl McpServer {
    pub fn register_commands(&self, commands: Vec<Command>) {
        let mut cmds = self.commands.write().unwrap();
        *cmds = commands.into_iter()
            .filter(|c| c.expose_via_mcp)
            .collect();
    }

    pub fn list_tools(&self) -> Vec<ToolSchema> {
        let cmds = self.commands.read().unwrap();
        cmds.iter().map(|c| c.to_tool_schema()).collect()
    }

    pub async fn call_tool(&self, name: &str, args: Value) -> Result<CallToolResult> {
        // 1. Find command by snake_case name
        let command = self.find_command(name)?;

        // 2. Validate arguments against schema
        self.validate_args(&command, &args)?;

        // 3. Convert args to HashMap<String, String>
        let arg_map = self.extract_args(&args)?;

        // 4. Render script with arguments
        let script = TemplateEngine::render_script(&command.script, &arg_map)?;

        // 5. Execute with timeout and capture output
        let output = self.execute_script(&script, &command).await?;

        // 6. Log execution
        self.log_execution(&command, &args, &output).await?;

        // 7. Return MCP-compliant result
        Ok(CallToolResult {
            content: vec![TextContent {
                text: format!("Exit code: {}\n\nstdout:\n{}\n\nstderr:\n{}",
                    output.exit_code,
                    output.stdout,
                    output.stderr
                ),
            }],
            is_error: output.exit_code != 0,
        })
    }
}
```

**Secure Execution Engine:**

```rust
pub struct ExecutionEngine;

impl ExecutionEngine {
    pub async fn execute(
        script: &str,
        working_dir: Option<&str>,
        env_vars: &HashMap<String, String>,
        timeout_secs: u32,
    ) -> Result<ExecutionOutput> {
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = std::process::Command::new("cmd");
            c.arg("/C").arg(script);
            c
        } else {
            let mut c = std::process::Command::new("sh");
            c.arg("-c").arg(script);
            c
        };

        // Set working directory
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // Set environment variables
        cmd.envs(env_vars);

        // Execute with timeout
        let output = tokio::time::timeout(
            Duration::from_secs(timeout_secs as u64),
            cmd.output()
        ).await.map_err(|_| AppError::ExecutionTimeout)?;

        let output = output?;

        // Limit output size
        let stdout = String::from_utf8_lossy(&output.stdout)
            .into_owned()
            .chars()
            .take(10_000)
            .collect();

        let stderr = String::from_utf8_lossy(&output.stderr)
            .into_owned()
            .chars()
            .take(10_000)
            .collect();

        Ok(ExecutionOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout,
            stderr,
        })
    }
}
```

**Execution Logging:**

```rust
pub struct ExecutionLog {
    pub id: String,
    pub command_id: String,
    pub command_name: String,
    pub arguments: Value,           // JSON
    pub script_rendered: String,    // Script with args injected
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub executed_at: DateTime<Utc>,
    pub triggered_by: String,       // "mcp" or "test"
    pub client_info: Option<String>, // MCP client identifier
}

// Database schema
CREATE TABLE execution_logs (
    id TEXT PRIMARY KEY NOT NULL,
    command_id TEXT NOT NULL,
    command_name TEXT NOT NULL,
    arguments TEXT NOT NULL,
    script_rendered TEXT NOT NULL,
    stdout TEXT,
    stderr TEXT,
    exit_code INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL,
    executed_at INTEGER NOT NULL,
    triggered_by TEXT NOT NULL,
    client_info TEXT
);

CREATE INDEX idx_execution_logs_command_id ON execution_logs(command_id);
CREATE INDEX idx_execution_logs_executed_at ON execution_logs(executed_at);
```

**IPC Commands:**

- `get_execution_history` - Get paginated execution logs
- `get_execution_by_id` - Get single execution details
- `clear_execution_history` - Delete old logs (with confirmation)
- `get_command_execution_stats` - Stats per command (success rate, avg duration)

#### Frontend Implementation

**New UI Components:**

- `ExecutionHistoryPage` - Full history view
- `ExecutionLogTable` - Paginated table
- `ExecutionLogDetail` - Expanded view with full output
- `ExecutionStats` - Charts/graphs of command usage
- `McpConnectionStatus` - Real-time connection indicator
- `RecentExecutions` - Live feed of recent calls

**Execution History Page:**

- Table columns: Timestamp, Command, Arguments, Exit Code, Duration, Triggered By
- Expandable rows showing full output
- Filter by: Command, Date Range, Success/Failure, Trigger Source
- Export to CSV/JSON
- Bulk delete old logs
- Pagination (25/50/100 per page)

**Dashboard Updates:**

- Recent executions widget (last 10)
- MCP connection status indicator
- Commands exposed via MCP count
- Total executions today
- Success rate chart

**Real-Time Updates:**

- WebSocket or polling for live execution feed
- Toast notifications for new MCP invocations
- Sound notification option (configurable)

#### Error Handling

| Error Type        | User Message                                        | Recovery                |
| ----------------- | --------------------------------------------------- | ----------------------- |
| Command not found | "Command '{name}' not found or not exposed via MCP" | Return error to client  |
| Invalid arguments | "Invalid arguments: {validation_errors}"            | Return schema to client |
| Execution timeout | "Command timed out after {X}s"                      | Return partial output   |
| Execution error   | "Command failed with exit code {N}"                 | Return stderr to client |
| Output too large  | "Output truncated (exceeded 10MB limit)"            | Return first/last 5MB   |
| Logging error     | "Failed to log execution"                           | Continue without log    |
| Client disconnect | Silent (expected during long execution)             | Continue execution      |

#### Documentation Tasks

- [ ] Document MCP tool execution flow
- [ ] Document execution logging
- [ ] Document error handling from client perspective
- [ ] Add troubleshooting guide for MCP connections
- [ ] Document performance considerations (timeouts, output limits)

#### Tests

**Unit Tests (Rust):**

- [ ] `test_to_tool_schema_simple` - Basic schema
- [ ] `test_to_tool_schema_all_types` - All argument types
- [ ] `test_to_tool_schema_required_optional` - Required fields
- [ ] `test_find_command_by_name` - Lookup
- [ ] `test_validate_args_success` - Valid args
- [ ] `test_validate_args_missing_required` - Validation error
- [ ] `test_validate_args_wrong_type` - Type error
- [ ] `test_extract_args_conversion` - Arg extraction
- [ ] `test_execute_success` - Happy path
- [ ] `test_execute_failure` - Non-zero exit
- [ ] `test_execute_timeout` - Timeout
- [ ] `test_execute_large_output` - Truncation
- [ ] `test_log_execution` - Audit trail
- [ ] `test_windows_command` - Windows shell
- [ ] `test_unix_command` - Unix shell

**Integration Tests:**

- [ ] `test_mcp_full_execution_flow` - End-to-end
- [ ] `test_mcp_concurrent_executions` - Parallel commands
- [ ] `test_mcp_client_reconnect` - Resilience
- [ ] `test_execution_history_persistence` - Database

**Frontend Tests:**

- [ ] Test ExecutionHistoryPage renders correctly
- [ ] Test filtering and pagination
- [ ] Test real-time updates
- [ ] Test export functionality

---

### Phase 2.5: GUI Polish & World-Class UX

**Goal:** Ensure the UI meets "world-class UI/UX" standards per our rules.

#### Visual Polish

- [ ] Verify NO hardcoded colors (use Tailwind tokens/CSS variables)
- [ ] Audit all components for color consistency
- [ ] Ensure responsive design (1024px minimum, mobile-friendly where applicable)
- [ ] Verify dark mode support throughout
- [ ] Add smooth transitions/animations (respect reduced-motion)
- [ ] Ensure consistent spacing (use design tokens)
- [ ] Verify typography hierarchy

#### Accessibility

- [ ] All interactive elements keyboard accessible
- [ ] Focus visible outlines on all focusable elements
- [ ] ARIA labels for all icons and non-text buttons
- [ ] Screen reader announcements for toasts and status changes
- [ ] Color contrast WCAG AA compliance (use contrast checker)
- [ ] Skip navigation links
- [ ] Form field labels and error associations
- [ ] Modal/dialog focus trapping

#### Keyboard Shortcuts

- [ ] `Ctrl/Cmd + N` - New rule
- [ ] `Ctrl/Cmd + Shift + N` - New command
- [ ] `Ctrl/Cmd + S` - Save current item
- [ ] `Ctrl/Cmd + Shift + S` - Sync all
- [ ] `Ctrl/Cmd + F` - Focus search
- [ ] `Ctrl/Cmd + ,` - Open settings
- [ ] `Ctrl/Cmd + 1` - Go to Dashboard
- [ ] `Ctrl/Cmd + 2` - Go to Rules
- [ ] `Ctrl/Cmd + 3` - Go to Commands
- [ ] `Escape` - Close dialogs/cancel
- [ ] `?` - Show keyboard shortcuts help

#### Feedback & Loading States

- [ ] Skeleton loaders for all async data fetching
- [ ] Loading spinners for actions
- [ ] Progress indicators for long operations
- [ ] Optimistic UI updates where appropriate
- [ ] Error boundaries with friendly error messages
- [ ] Toast notifications for all CRUD operations
- [ ] Undo functionality for delete operations

#### UI Component Inventory

**Phase 2 New Components:**

| Component              | Purpose                | Location             |
| ---------------------- | ---------------------- | -------------------- |
| FileStorageStatus      | Storage mode indicator | Header               |
| MigrationDialog        | Migration UI           | Dialog               |
| FilePathDisplay        | File path with copy    | Rule cards           |
| McpStatusIndicator     | MCP connection status  | Header/Dashboard     |
| McpConnectionPanel     | Connection details     | Settings             |
| ConnectionInstructions | Setup guide            | Settings             |
| McpLogsViewer          | Connection logs        | Settings             |
| CommandsPage           | Commands list          | Page                 |
| CommandCard            | Command display        | CommandsPage         |
| CommandEditor          | Full command editor    | Page                 |
| ScriptEditor           | Code editor            | CommandEditor        |
| ArgumentBuilder        | Dynamic args form      | CommandEditor        |
| ArgumentTypeSelector   | Type dropdown          | ArgumentBuilder      |
| McpSchemaPreview       | JSON preview           | CommandEditor        |
| CommandTester          | Test panel             | CommandEditor        |
| TestOutputViewer       | Output display         | CommandTester        |
| ExecutionHistoryPage   | History view           | Page                 |
| ExecutionLogTable      | Paginated table        | ExecutionHistoryPage |
| ExecutionLogDetail     | Expanded view          | ExecutionLogTable    |
| ExecutionStats         | Usage charts           | Dashboard            |
| RecentExecutions       | Live feed              | Dashboard            |

---

## 3. Execution Checklist

### Phase 2.0: Project Setup

- [ ] Create feature branch `feature/phase-2-mcp-commands`
- [ ] Run all Phase 1 tests to establish baseline
- [ ] Run lint and typecheck (should pass)
- [ ] Verify CI/CD pipeline is green
- [ ] Review and update any outdated dependencies

### Phase 2.1: File-First Storage (Issue #14)

**Backend:**

- [ ] Add `serde_yaml`, `notify`, `glob` to Cargo.toml
- [ ] Create `src-tauri/src/file_storage/mod.rs`
- [ ] Create `src-tauri/src/file_storage/parser.rs` - YAML parsing
- [ ] Create `src-tauri/src/file_storage/serializer.rs` - YAML generation
- [ ] Create `src-tauri/src/file_storage/watcher.rs` - File watching
- [ ] Create `src-tauri/src/file_storage/migration.rs` - Migration logic
- [ ] Implement `load_rules_from_disk()` with directory traversal
- [ ] Implement `save_rule_to_disk()` with atomic writes
- [ ] Implement file watcher with debounced events
- [ ] Create database migration v3 (remove content field from rules)
- [ ] Update SQLite schema for index-only storage
- [ ] Create `migrate_to_file_storage()` IPC command
- [ ] Create `verify_file_storage_integrity()` health check
- [ ] Implement `rollback_migration()` for safety
- [ ] Update all existing rule IPC commands
- [ ] Handle file permission errors
- [ ] Handle concurrent editing conflicts
- [ ] Handle large files with chunked reading
- [ ] Handle special characters and Unicode
- [ ] Handle duplicate IDs across directories
- [ ] Handle orphaned files
- [ ] Handle git merge conflicts

**Frontend:**

- [ ] Create `FileStorageStatus` component
- [ ] Create `MigrationDialog` component with progress
- [ ] Create `FilePathDisplay` component
- [ ] Create `RevealInExplorerButton` component
- [ ] Add "Open in Editor" button to rule cards
- [ ] Show file path in rule detail view
- [ ] Display file timestamps in UI
- [ ] Add file system health indicators
- [ ] Add storage mode indicator in settings

**Documentation:**

- [ ] Update README.md with file-based architecture
- [ ] Document `.ruleweaver/rules/` directory structure
- [ ] Document YAML frontmatter schema
- [ ] Add migration guide for existing users
- [ ] Document file watching behavior
- [ ] Document manual editing best practices

**Tests:**

- [ ] Unit tests for YAML parsing (happy path, invalid, missing fields)
- [ ] Unit tests for file I/O operations
- [ ] Unit tests for migration (forward and rollback)
- [ ] Unit tests for file watcher
- [ ] Unit tests for edge cases (permissions, large files, Unicode)
- [ ] Integration tests for full migration flow
- [ ] Integration tests for git workflow compatibility
- [ ] Frontend component tests
- [ ] 80% coverage target (high-value tests only)

### Phase 2.2: MCP Server (Issue #5)

**Backend:**

- [ ] Research and select MCP SDK (or custom implementation)
- [ ] Add `tokio` dependency to Cargo.toml
- [ ] Create `src-tauri/src/mcp/mod.rs`
- [ ] Create `src-tauri/src/mcp/server.rs`
- [ ] Create `src-tauri/src/mcp/protocol.rs` - JSON-RPC types
- [ ] Create `src-tauri/src/mcp/handlers.rs` - Endpoint handlers
- [ ] Create `src-tauri/src/mcp/transport.rs` - Stdio transport
- [ ] Define JSON-RPC request/response types
- [ ] Implement `McpServer` struct with state management
- [ ] Implement `start()` with port binding
- [ ] Implement `stop()` with graceful shutdown
- [ ] Implement `initialize` endpoint
- [ ] Implement `tools/list` endpoint (initially empty)
- [ ] Implement connection logging
- [ ] Add MCP configuration persistence
- [ ] Create IPC commands: get_mcp_status, start_mcp_server, stop_mcp_server
- [ ] Create IPC commands: restart_mcp_server, get_mcp_connection_instructions
- [ ] Create IPC commands: get_mcp_logs
- [ ] Implement port conflict detection
- [ ] Handle graceful shutdown on app close

**Frontend:**

- [ ] Create `McpStatusIndicator` component
- [ ] Create `McpConnectionPanel` component
- [ ] Create `ConnectionInstructions` component
- [ ] Create `McpLogsViewer` component
- [ ] Create `McpSettingsSection` component
- [ ] Build MCP status page with controls
- [ ] Build connection instructions panel
- [ ] Add MCP settings to Settings page
- [ ] Add port configuration with validation
- [ ] Add auto-start toggle
- [ ] Add log level selector
- [ ] Create copy buttons for client configs

**Documentation:**

- [ ] Document MCP protocol support
- [ ] Document connection setup for Claude Code
- [ ] Document connection setup for OpenCode
- [ ] Document MCP troubleshooting
- [ ] Add MCP architecture diagram

**Tests:**

- [ ] Unit tests for server initialization
- [ ] Unit tests for start/stop lifecycle
- [ ] Unit tests for JSON-RPC message handling
- [ ] Unit tests for error handling
- [ ] Integration tests for server lifecycle
- [ ] Integration tests for client connection
- [ ] Integration tests for concurrent clients
- [ ] Integration tests for port conflicts
- [ ] Frontend component tests
- [ ] 80% coverage target

### Phase 2.3: Command Management (Issue #6)

**Backend:**

- [ ] Create `src-tauri/src/models/command.rs`
- [ ] Define `Command` struct with all fields
- [ ] Define `CommandArgument` struct
- [ ] Define `ArgumentType` enum
- [ ] Add `commands` table to SQLite (index only)
- [ ] Implement `load_commands_from_disk()`
- [ ] Implement `save_command_to_disk()`
- [ ] Create commands directory structure
- [ ] Implement IPC: get_all_commands
- [ ] Implement IPC: get_command_by_id
- [ ] Implement IPC: create_command
- [ ] Implement IPC: update_command
- [ ] Implement IPC: delete_command
- [ ] Create `TemplateEngine` struct
- [ ] Implement `extract_placeholders()`
- [ ] Implement `validate_template()`
- [ ] Implement `render_script()`
- [ ] Implement `escape_shell_arg()`
- [ ] Implement `detect_dangerous_patterns()`
- [ ] Block dangerous patterns (rm -rf, pipes, etc.)
- [ ] Implement IPC: test_command
- [ ] Implement timeout handling
- [ ] Capture stdout/stderr with size limits
- [ ] Extend `SyncAdapter` trait for commands
- [ ] Implement command sync for Gemini (TOML)
- [ ] Implement command sync for OpenCode (MD)
- [ ] Implement command sync for Claude Code (MD)
- [ ] Implement command sync for Cline (MD)

**Frontend:**

- [ ] Create TypeScript types for Command model
- [ ] Create `commandsStore.ts` with Zustand
- [ ] Create `CommandsPage` component
- [ ] Create `CommandCard` component
- [ ] Create `CommandEditor` component
- [ ] Create `ScriptEditor` component (Monaco/CodeMirror)
- [ ] Create `ArgumentBuilder` component
- [ ] Create `ArgumentTypeSelector` component
- [ ] Create `McpSchemaPreview` component
- [ ] Create `CommandTester` component
- [ ] Create `TestOutputViewer` component
- [ ] Implement commands list view with search/sort/filter
- [ ] Implement command editor with all sections
- [ ] Implement dynamic argument builder
- [ ] Implement script template validation
- [ ] Implement MCP schema preview
- [ ] Implement in-app testing with output display
- [ ] Add TypeScript types for all components

**Documentation:**

- [ ] Document Command file format (YAML frontmatter)
- [ ] Document script template syntax
- [ ] Document available argument types
- [ ] Document security restrictions
- [ ] Add command examples and best practices
- [ ] Document MCP tool naming conventions

**Tests:**

- [ ] Unit tests for Command model serialization
- [ ] Unit tests for template parsing
- [ ] Unit tests for template validation
- [ ] Unit tests for script rendering
- [ ] Unit tests for shell argument escaping
- [ ] Unit tests for dangerous pattern detection
- [ ] Unit tests for test execution (happy path)
- [ ] Unit tests for test execution (failure paths)
- [ ] Unit tests for timeout handling
- [ ] Unit tests for CRUD operations
- [ ] Unit tests for sync adapter extensions
- [ ] Integration tests for full command lifecycle
- [ ] Integration tests for security boundaries
- [ ] Frontend component tests
- [ ] 80% coverage target

### Phase 2.4: MCP Execution (Issue #7)

**Backend:**

- [ ] Implement `to_tool_schema()` for Command
- [ ] Implement `build_input_schema()` for JSON Schema
- [ ] Implement `register_commands()` in McpServer
- [ ] Implement `list_tools()` in McpServer
- [ ] Implement `find_command_by_name()`
- [ ] Implement argument validation against schema
- [ ] Implement `extract_args()` for MCP requests
- [ ] Implement `call_tool()` handler
- [ ] Create `ExecutionEngine` struct
- [ ] Implement `execute()` with Windows/Unix support
- [ ] Implement working directory handling
- [ ] Implement environment variable handling
- [ ] Implement execution timeout
- [ ] Implement stdout/stderr capture
- [ ] Implement output size limiting
- [ ] Create `execution_logs` table in SQLite
- [ ] Implement execution logging
- [ ] Create IPC: get_execution_history
- [ ] Create IPC: get_execution_by_id
- [ ] Create IPC: clear_execution_history
- [ ] Create IPC: get_command_execution_stats
- [ ] Handle execution errors gracefully
- [ ] Return structured MCP responses

**Frontend:**

- [ ] Create `ExecutionHistoryPage` component
- [ ] Create `ExecutionLogTable` component
- [ ] Create `ExecutionLogDetail` component
- [ ] Create `ExecutionStats` component
- [ ] Create `McpConnectionStatus` component
- [ ] Create `RecentExecutions` component
- [ ] Build execution history page
- [ ] Build paginated log table
- [ ] Build expanded detail view
- [ ] Build stats/charts for command usage
- [ ] Add recent executions to Dashboard
- [ ] Implement real-time updates
- [ ] Add toast notifications for executions

**Documentation:**

- [ ] Document MCP tool execution flow
- [ ] Document execution logging
- [ ] Document error handling from client perspective
- [ ] Add troubleshooting guide for MCP connections
- [ ] Document performance considerations

**Tests:**

- [ ] Unit tests for tool schema generation
- [ ] Unit tests for JSON Schema building
- [ ] Unit tests for argument validation
- [ ] Unit tests for argument extraction
- [ ] Unit tests for script rendering with args
- [ ] Unit tests for execution engine
- [ ] Unit tests for Windows shell execution
- [ ] Unit tests for Unix shell execution
- [ ] Unit tests for timeout handling
- [ ] Unit tests for output limiting
- [ ] Unit tests for execution logging
- [ ] Integration tests for full MCP execution flow
- [ ] Integration tests for concurrent executions
- [ ] Integration tests for client reconnection
- [ ] Frontend component tests
- [ ] 80% coverage target

### Phase 2.5: Polish & Verification

**Visual Polish:**

- [ ] Verify NO hardcoded colors anywhere
- [ ] Audit all components for color consistency
- [ ] Verify responsive design (1024px minimum)
- [ ] Verify dark mode support throughout
- [ ] Add smooth transitions (respect reduced-motion)
- [ ] Ensure consistent spacing
- [ ] Verify typography hierarchy

**Accessibility:**

- [ ] Keyboard accessibility audit
- [ ] Focus visible on all elements
- [ ] ARIA labels for icons
- [ ] Screen reader announcements
- [ ] Color contrast WCAG AA check
- [ ] Skip navigation links
- [ ] Form labels and errors
- [ ] Modal focus trapping

**Keyboard Shortcuts:**

- [ ] Implement Ctrl/Cmd + N (New rule)
- [ ] Implement Ctrl/Cmd + Shift + N (New command)
- [ ] Implement Ctrl/Cmd + S (Save)
- [ ] Implement Ctrl/Cmd + Shift + S (Sync)
- [ ] Implement Ctrl/Cmd + F (Search)
- [ ] Implement Ctrl/Cmd + , (Settings)
- [ ] Implement Ctrl/Cmd + 1/2/3 (Navigation)
- [ ] Implement Escape (Close)
- [ ] Implement ? (Shortcuts help)

**Feedback & Loading:**

- [ ] Skeleton loaders for async data
- [ ] Loading spinners for actions
- [ ] Progress indicators for long ops
- [ ] Optimistic UI updates
- [ ] Error boundaries
- [ ] Toast notifications
- [ ] Undo for delete

**Quality Assurance:**

- [ ] Run `npm run lint` - no warnings
- [ ] Run `npm run typecheck` - no errors
- [ ] Run `npm run lint:rust` - no warnings
- [ ] Run `npm run test` - all passing
- [ ] Run `npm run test:rust` - all passing
- [ ] Verify 80% test coverage on new code
- [ ] Manual testing on Windows
- [ ] Test file migration from SQLite to files
- [ ] Test MCP server connection from Claude Code
- [ ] Test MCP server connection from OpenCode
- [ ] Test command execution via MCP
- [ ] Test in-app command testing
- [ ] Test execution history and logging
- [ ] Test sync of commands to files
- [ ] Test all keyboard shortcuts
- [ ] Test accessibility (keyboard-only navigation)
- [ ] Test responsive design
- [ ] Test dark mode
- [ ] Test error scenarios
- [ ] Verify all edge cases handled
- [ ] Review UI consistency
- [ ] Verify no hardcoded colors
- [ ] Accessibility audit with screen reader

**Documentation:**

- [ ] Update main README.md with Phase 2 features
- [ ] Document breaking changes
- [ ] Update architecture diagram
- [ ] Add troubleshooting section
- [ ] Add FAQ for common issues
- [ ] Update screenshots/GIFs

---

## 4. Technical Specifications

### Dependencies (Rust)

```toml
# Add to src-tauri/Cargo.toml

# File Storage
serde_yaml = "0.9"              # YAML serialization
notify = "6"                    # File system watching
glob = "0.3"                    # File pattern matching

# MCP Server
tokio = { version = "1", features = ["full"] }  # Async runtime
# Option 1: rmcp = "0.1"        # Rust MCP SDK (if available)
# Option 2: jsonrpc-core = "18" # JSON-RPC framework
# Option 3: Custom implementation with hyper/axum

# Additional utilities
indexmap = "2"                  # Ordered HashMap for YAML
unicode-segmentation = "1"      # Unicode handling
```

### Dependencies (Frontend)

```json
// package.json additions
{
  "dependencies": {
    "@monaco-editor/react": "^4.6.0" // For script editing (optional)
  }
}
```

### File Locations

```
# Rule Files (Global)
~/.ruleweaver/rules/                    # User home directory
  ├── 000-global-coding.md
  └── 001-security-guidelines.md

# Rule Files (Local - per project)
{project}/.ruleweaver/rules/            # Project root
  ├── 000-project-specific.md
  └── ...

# Command Files (Global only for now)
~/.ruleweaver/commands/                 # User home directory
  ├── 000-format-code.md
  └── 001-run-tests.md

# Database (Index/Cache only)
%APPDATA%/RuleWeaver/ruleweaver.db      # Windows
~/.local/share/RuleWeaver/ruleweaver.db # Linux
~/Library/Application Support/RuleWeaver/ruleweaver.db # macOS

# Synced Rule Outputs
~/.gemini/GEMINI.md                     # Gemini CLI rules
~/.opencode/AGENTS.md                   # OpenCode rules
~/.clinerules                           # Cline rules
~/.claude/CLAUDE.md                     # Claude Code rules
~/.codex/CODEX.md                       # Codex rules
~/.antigravity/ANTIGRAVITY.md           # Antigravity rules

# Synced Command Outputs
~/.gemini/COMMANDS.toml                 # Gemini CLI commands
~/.opencode/COMMANDS.md                 # OpenCode commands
~/.claude/COMMANDS.md                   # Claude Code commands
```

### YAML Frontmatter Schema

**Rules:**

```yaml
---
id: string (UUID)              # Required: unique identifier
name: string                   # Required: display name
scope: "global" | "local"      # Required: scope type
targetPaths: string[]          # Optional: for local scope
enabledAdapters: string[]      # Required: gemini, opencode, cline, etc.
enabled: boolean               # Required: active status
createdAt: ISO8601             # Required: creation timestamp
updatedAt: ISO8601             # Required: last modified
---
```

**Commands:**

```yaml
---
id: string (UUID) # Required: unique identifier
name: string # Required: display name
description: string # Required: command description
script: string # Required: script template
arguments: # Required: argument definitions
  - name: string
    description: string
    type: string | number | boolean | enum
    required: boolean
    defaultValue: string # Optional
    enumValues: string[] # Optional (for enum type)
exposeViaMcp: boolean # Required: expose as MCP tool
workingDirectory: string # Optional: default working dir
environmentVariables: # Optional: env vars
  KEY: value
timeoutSeconds: number # Required: default 60
createdAt: ISO8601 # Required: creation timestamp
updatedAt: ISO8601 # Required: last modified
---
```

### IPC Commands Reference

**File Storage:**

- `migrate_to_file_storage() -> Result<MigrationReport>` - One-time migration
- `verify_file_storage_integrity() -> Result<HealthReport>` - Health check
- `rollback_migration() -> Result<()>` - Rollback to SQLite
- `get_storage_type() -> StorageType` - "file" or "sqlite"

**MCP Server:**

- `get_mcp_status() -> McpStatus` - Server status, port, uptime
- `start_mcp_server() -> Result<()>` - Start server
- `stop_mcp_server() -> Result<()>` - Stop server
- `restart_mcp_server() -> Result<()>` - Restart server
- `get_mcp_connection_instructions() -> ConnectionInstructions` - Setup guide
- `get_mcp_logs(limit: u32) -> Vec<McpLogEntry>` - Connection logs

**Commands:**

- `get_all_commands() -> Vec<Command>` - List all commands
- `get_command_by_id(id: String) -> Command` - Get single command
- `create_command(input: CreateCommandInput) -> Command` - Create command
- `update_command(id: String, input: UpdateCommandInput) -> Command` - Update command
- `delete_command(id: String) -> Result<()>` - Delete command
- `test_command(id: String, args: Value) -> TestCommandResult` - Test execute

**Execution:**

- `get_execution_history(limit: u32, offset: u32) -> Vec<ExecutionLog>` - Paginated history
- `get_execution_by_id(id: String) -> ExecutionLog` - Single execution details
- `clear_execution_history(before: Option<DateTime>) -> Result<()>` - Delete old logs
- `get_command_execution_stats(command_id: String) -> CommandStats` - Usage stats

### Security Checklist

**File System:**

- [ ] All file paths validated before operations
- [ ] Path traversal prevention (only allow within .ruleweaver directories)
- [ ] File permission checks before read/write
- [ ] Atomic file writes (write to temp, then rename)

**Command Execution:**

- [ ] All arguments escaped before shell execution
- [ ] Dangerous patterns blocked (rm -rf, pipes, redirections, command substitution)
- [ ] Timeout on all executions (configurable, max 5 minutes)
- [ ] Output size limits (prevent memory exhaustion)
- [ ] Working directory validation
- [ ] Environment variable sanitization

**MCP Protocol:**

- [ ] Server binds to localhost only (127.0.0.1)
- [ ] No external network exposure
- [ ] Input validation on all JSON-RPC messages
- [ ] Rate limiting on tool calls (prevent abuse)
- [ ] Execution logging for audit trail

**Input Validation:**

- [ ] Command names: alphanumeric, dashes, underscores only
- [ ] Argument names: same restrictions
- [ ] Script length limit (10,000 chars)
- [ ] Argument count limit (20 per command)
- [ ] Description length limit (500 chars)

---

## 5. Implementation Order & Dependencies

```
Phase 2.0: Project Setup
    │
    ▼
Phase 2.1: File-First Storage (#14)
    │
    ├──► Phase 2.2: MCP Server (#5) ──┐
    │                                  │
    └──► Phase 2.3: Commands (#6) ─────┤
                                       │
                                       ▼
                            Phase 2.4: MCP Execution (#7)
                                       │
                                       ▼
                            Phase 2.5: Polish & Verification
```

### Dependency Notes

- **#14 must complete before #6** - Commands use same file storage pattern
- **#5 and #6 can run in parallel** after #14
- **#7 requires both #5 and #6** - Connects commands to MCP server
- **#5 can start research early** - SDK selection doesn't depend on #14

### Timeline Estimate

| Phase | Issue  | Priority | Duration | Dependencies  |
| ----- | ------ | -------- | -------- | ------------- |
| 2.0   | Setup  | HIGH     | 0.5 day  | None          |
| 2.1   | #14    | HIGH     | 4-5 days | 2.0           |
| 2.2   | #5     | MEDIUM   | 3-4 days | 2.1 (partial) |
| 2.3   | #6     | MEDIUM   | 4-5 days | 2.1           |
| 2.4   | #7     | MEDIUM   | 3-4 days | 2.2, 2.3      |
| 2.5   | Polish | HIGH     | 2-3 days | 2.4           |

**Total: ~17-22 days** with parallelization on #5 and #6.

---

## 6. Risk Assessment & Mitigation

| Risk                             | Likelihood | Impact   | Mitigation                                            |
| -------------------------------- | ---------- | -------- | ----------------------------------------------------- |
| MCP SDK not mature               | Medium     | High     | Build custom JSON-RPC implementation as fallback      |
| File watcher performance         | Low        | Medium   | Use debouncing, test with large directories           |
| Migration data loss              | Low        | Critical | Backup SQLite before migration, implement rollback    |
| Script injection vulnerability   | Low        | Critical | Thorough security review, dangerous pattern detection |
| Cross-platform shell differences | Medium     | Medium   | Test on Windows, macOS, Linux; use appropriate shells |
| Large output memory issues       | Medium     | Medium   | Implement output size limits and streaming            |
| Concurrent edit conflicts        | Medium     | Low      | Clear UI warnings, conflict resolution options        |

---

## 7. Definition of Done

Each issue (#14, #5, #6, #7) is complete when:

### Code Completeness

- [ ] All features implemented per specifications
- [ ] No TODO, FIXME, or placeholder comments
- [ ] All error cases handled with user-friendly messages
- [ ] No unwrap() or expect() in production code (use proper error handling)
- [ ] All edge cases addressed

### Testing

- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] 80%+ test coverage (high-value tests only)
- [ ] Happy paths tested
- [ ] Failure paths tested
- [ ] Edge cases tested
- [ ] Security boundaries tested

### Quality

- [ ] Lint and typecheck passing with no warnings
- [ ] No hardcoded colors (using design tokens)
- [ ] Responsive design verified
- [ ] Dark mode support verified
- [ ] Accessibility audit passed
- [ ] Keyboard shortcuts working
- [ ] Performance acceptable (no obvious lag)

### Documentation

- [ ] README updated
- [ ] Architecture docs updated if changed
- [ ] User-facing documentation complete
- [ ] Code comments for complex logic
- [ ] API documentation (IPC commands)

### Manual Verification

- [ ] Manual testing completed on Windows
- [ ] All features work end-to-end
- [ ] Error scenarios tested manually
- [ ] UI polish verified
- [ ] No console errors

---

## 8. Future Work (Phase 3+ Acknowledgment)

Per `architecture.md` and `roadmap.md`, the following are **acknowledged but NOT in this plan**:

### Phase 3: Skills MVP (Roadmap)

- Support for complex, multi-file execution environments
- "Skills Manager" UI for bundled workflows
- Skills as directories with SKILL.md and auxiliary scripts
- Multi-stage python/bash script execution
- Templates for common skills

### Phase 4: Polish & Extensibility (Roadmap)

- System tray icon for background operation
- Enhanced conflict resolution
- Import/export configurations

### Phase 5: Advanced Ecosystem (Roadmap)

- **Secrets & Vault Management** - NIST SP 800-63B password security compliance
- Live execution logging dashboard
- Community Hub / Registry for sharing rules

These are explicitly out of scope for Milestone #2 but acknowledged in the architecture.

---

## 9. Notes

### Why File-First Storage is Critical

Issue #14 is marked HIGH PRIORITY because:

1. Git-native version control for rules
2. Transparency - users can edit files directly
3. Portability - no database export needed
4. Emergency access - rules remain readable even if app breaks
5. Developer-friendly - review rule changes in PRs
6. Aligns with architecture flexibility (SQLite OR file storage)

### MCP Transport Decision

Start with **stdio transport** (simpler, no port conflicts):

- Claude Code and OpenCode support stdio MCP servers
- HTTP can be added later as enhancement
- stdio is more secure (no network exposure)

### Security First

Command execution is the highest security risk:

- All user input must be escaped
- Dangerous patterns must be blocked
- Timeouts prevent runaway processes
- Audit logging for accountability
- Principle of least privilege (minimal permissions)

### Testing Philosophy

Per our rules:

- High-value tests > coverage metrics
- No low-value tests (getters/setters, trivial functions)
- Both happy and failure paths
- If 80% coverage requires low-value tests, stop at lower percentage

### Documentation Standards

- Update README.md for user-facing changes
- Document architecture decisions in comments
- Keep AGENTS.md updated (if exists)
- Screenshots/GIFs for major UI changes

---

## 10. Robust MCP Mode Addendum

To make MCP reliable for daily workflows and tool reconnects, we add a dual-mode runtime:

### A. Embedded MCP (Desktop App Runtime)

- Keep MCP server embedded in Tauri for integrated UX and settings controls.
- Add `mcp_auto_start` setting so MCP can start automatically when app launches.
- Expose status/start/stop/restart and connection instructions in Settings.

### B. Standalone MCP Binary (`ruleweaver-mcp`)

- Ship a separate CLI/EXE entrypoint that starts MCP without opening the desktop UI.
- Command shape: `ruleweaver-mcp --port <PORT>`.
- Reuse the same database and command registry as the desktop app.
- Include standalone launch snippet in MCP connection instructions so AI tools can call it directly.

### C. Process & Background Expectations

- Embedded mode requires RuleWeaver app process alive.
- Standalone mode requires `ruleweaver-mcp` process alive.
- No Docker container is required.

### D. Remaining hardening after implementation

- Add system tray/background mode to keep embedded MCP alive when window closes.
- Add startup conflict handling between embedded MCP and standalone MCP on same port.
- Add reconnect-resilience tests for long-lived MCP clients.

### E. Implementation status in current branch

- Tray/background mode implemented (`minimize_to_tray` + tray menu controls).
- Standalone MCP binary implemented (`ruleweaver-mcp`).
- Command stub sync implemented (`sync_commands` for Gemini/OpenCode/Claude Code files).
- Phase 3 foundation started: Skills CRUD backend + initial Skills page/navigation.
