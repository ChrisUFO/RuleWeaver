# Project Strategy: RuleWeaver

## 1. High-Level Strategy

The objective of Milestone 3 is to introduce and fully implement the "Skills" functionality. A Skill is a complex multi-file workflow, encapsulated in a directory with a `SKILL.md` description and executable helper scripts (e.g., Python, Node.js, Shell).
The strategy involves four main pillars, plus a cross-cutting security pillar:

1. **Skill Architecture & Management (Tier 3.1):** Building the core Rust data models, SQLite storage, Tauri IPC commands, and frontend UI to import, create, and manage skills.
2. **Advanced MCP Execution (Tier 3.2):** Securely exposing these skills as MCP tools, injecting necessary environment variables, handling sandboxed execution across different runtimes, and capturing structured output.
3. **Skill Input Schema & Runtime Validation (Tier 3.4):** Enforcing rich input schemas (JSON Schema styles) for skill parameters, strongly typed MCP argument validation, execution policies, and timeout constraints.
4. **Skill Templates Library (Tier 3.3):** Shipping built-in, production-ready skill templates that users can install/enable with a single click, complete with idempotent updates and versioning.
5. **Security & Secrets Management (Cross-cutting):** Given the execution of multi-language scripts, ensuring we have secure secrets passing (e.g. API keys for templates), user-approved execution policies (allowlist/denylist), and strict logging of all stdout/stderr and validation failures.

## 2. Implementation Plan

### Phase 0: Setup & Preparation

- Initialize feature branch: `feature/milestone-3-skills`.
- Verify current test suite passes (`npm run test`, `cargo test`, `npm run lint`).

### Phase 1: Core Architecture, Schema & UI (Tier 3.1 & Tier 3.4 Schemas)

- Define the `Skill` struct in Rust and the SQLite database schema fields for indexing, adding properties for schemas and constraints (max args size, max output size, runtime environments).
- Implement File Storage synchronization (`src-tauri/src/file_storage/skills.rs`) so that the file system (`SKILL.md`/`skill.json`) acts as the source of truth, matching the hybrid architecture used for rules and commands.
- Include schema definition fields (for Tier 3.4) such as expected parameters, types (string/number/boolean/enum/array/object), defaults, and requirements.
- Implement Tauri IPC commands (`get_all_skills`, `get_skill_by_id`, `create_skill`, `import_skill`, `update_skill`, `delete_skill`, `validate_skill`).
- Build the Frontend UI components to list, create, edit, and validate skills. Provide intuitive editing of skill metadata and parameter schemas.
- **Testing:** Unit tests for Rust models, SQLite CRUD operations, and schema validation logic.

### Phase 2: Execution Engine, Validation & Sandboxing (Tier 3.2 & Tier 3.4 Execution)

- Expose skills dynamically in the MCP `tools/list` endpoint by mapping the skill parameter schema to MCP JSON-RPC tool schemas.
- Implement the comprehensive execution engine in Rust (`src-tauri/src/execution.rs`) to route and run skills based on runtime (Node.js, Python, Bash, PowerShell).
- Implement strict runtime validation using the defined schemas _before_ execution. Return structured validation errors on failure.
- Inject environment variables (`RULEWEAVER_SKILL_ID`, `RULEWEAVER_SKILL_NAME`, `RULEWEAVER_SKILL_DIR`) and set the working directory context to the skill root.
- Implement execution security: Allowlist/denylist policies per skill.
- Implement runtime constraints: enforce max output size truncation and strict skill-specific timeout policies.
- Implement structured output capture and centralized execution logging (recording args, stdout, stderr, exit codes).
- **Testing:** Integration tests for successful execution, multi-stage scripts, timeouts, schema validation failures, and policy denials. Coverage must include both happy and failure paths.

### Phase 3: Template Library, Polish & Secrets (Tier 3.3)

- Create a registry of built-in skill templates (e.g., `lint-and-fix`, `test-generation`, `code-review`).
- Build Frontend Template Browser UI in the Skills page to allow 1-click install, enable, and disable of templates.
- Implement idempotent install/update logic in Rust to prevent duplicates and respect user overrides across versions.
- Handle Secrets: Implement secure parameter overriding so users can safely supply credentials (if required by templates) without committing them to the skill definition.
- Polish: Ensure all UI components adhere to RuleWeaver's premium standard (glassmorphism, excellent loading states, no unhandled exceptions).
- Produce user documentation for skill authoring, schema definition, and template usage.
- **Testing:** Unit and integration tests for template registry loading, idempotency during re-install, and version migrations.

## 3. Execution Checklist

### Phase 0

- [ ] `git checkout -b feature/milestone-3-skills`
- [ ] Run base tests and linters to verify clean state

### Phase 1

- [ ] Define Rust `Skill` data model, schema constraints, and SQLite schema for indexing.
- [ ] Implement File Storage parsing and DB synchronization for Skills.
- [ ] Implement Rust database queries and Skill CRUD IPC commands.
- [ ] Build Frontend "Skills Manager" UI for listing/viewing.
- [ ] Build Frontend UI to import, create, and edit Skill metadata/schema.
- [ ] Implement Skill Schema validation logic in Rust.
- [ ] Write DB and CRUD unit tests for Rust.
- [ ] Write component tests for Skills UI.

### Phase 2

- [ ] Expose Skills dynamically via MCP `tools/list`.
- [ ] Implement multi-runtime Skill Execution Engine (Python/Node/Shell/Pwsh).
- [ ] Implement pre-execution argument validation against Skill schema.
- [ ] Implement execution constraints (Sandboxing, timeouts, max output size).
- [ ] Implement execution policies (Allowlist/denylist execution blocking).
- [ ] Implement comprehensive logging (args, stdout, stderr, policy blocks).
- [ ] Write execution and schema validation integration tests (happy & failure paths).

### Phase 3

- [ ] Create built-in Skill templates registry with curated examples.
- [ ] Build Template Browser UI with one-click install/enable functionality.
- [ ] Implement idempotent template installation and update migration behavior.
- [ ] Add Secrets/Credentials input handling for Skills requiring keys.
- [ ] Add User docs for template usage, schema authoring, and lifecycle.
- [ ] Run full test suite (`cargo test`, `vitest`, `cargo clippy`, `eslint`).
- [ ] Final UI/UX Polish and QA review.
