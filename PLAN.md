# Project Strategy: RuleWeaver Phase 1

## 1. High-Level Strategy

Build the Phase 1 MVP for RuleWeaver - a Tauri desktop application that enables users to manage AI coding assistant rules through a unified GUI and sync them to various tools via adapters.

**Core Objectives:**

1. Establish a solid Tauri + React + TypeScript + TailwindCSS foundation with shadcn/ui components
2. Implement persistent data storage for rules using Rust backend with SQLite
3. Build a polished GUI with dashboard, rules management, and markdown editing
4. Create a sync engine with tool-specific adapters (GEMINI.md, AGENTS.md, .clinerules)

**Technical Approach:**

- Frontend: React 18+ with TypeScript, TailwindCSS, shadcn/ui component library
- Backend: Rust with Tauri IPC commands for frontend communication
- Database: SQLite embedded via `rusqlite` crate in user's AppData directory
- File Sync: Trait-based adapter pattern for extensibility

---

## 2. Implementation Plan

### Phase 1.1: Project Setup (Issue #1)

**Goal:** Initialize Tauri project with all tooling configured.

**Tasks:**

1. Run `npm create tauri-app@latest` with React, TypeScript, TailwindCSS options
2. Configure TailwindCSS with custom design tokens (no hardcoded colors)
3. Install and configure shadcn/ui component library
4. Setup ESLint, Prettier, and TypeScript strict mode
5. Configure Rust backend for absolute filesystem paths
6. Setup Vitest for frontend testing
7. Setup Rust tests for backend testing
8. Create base directory structure matching architecture.md

**Directory Structure:**

```
src/
├── components/          # React components
│   ├── ui/             # shadcn/ui primitives
│   └── layout/         # Layout components
├── hooks/              # Custom React hooks
├── lib/                # Utilities and helpers
├── pages/              # Page components
├── stores/             # State management
└── types/              # TypeScript types
src-tauri/
├── src/
│   ├── commands/       # Tauri IPC commands
│   ├── database/       # SQLite database layer
│   ├── models/         # Rust data models
│   ├── sync/           # Sync engine and adapters
│   └── lib.rs          # Main entry point
└── Cargo.toml
```

---

### Phase 1.2: Core Data Storage (Issue #2)

**Goal:** Implement Rust data models and persistent storage with IPC.

**Tasks:**

1. Define Rust structs for `Rule`, `Scope`, `TargetDefinition` with serde serialization
2. Implement SQLite database manager with migrations
3. Create database migration system for schema versioning
4. Implement CRUD operations for rules in Rust
5. Create Tauri IPC commands for all rule operations
6. Setup database path in platform-specific AppData/Config directory
7. Add error handling with custom error types
8. Write comprehensive Rust unit tests for database operations

**Data Models:**

```rust
pub enum Scope { Global, Local }

pub struct Rule {
    pub id: String,
    pub name: String,
    pub content: String,
    pub scope: Scope,
    pub target_paths: Option<Vec<String>>,
    pub enabled_adapters: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
```

**IPC Commands:**

- `get_all_rules` - Fetch all rules
- `get_rule_by_id` - Fetch single rule
- `create_rule` - Create new rule
- `update_rule` - Update existing rule
- `delete_rule` - Delete rule
- `get_app_data_path` - Get storage path for display

---

### Phase 1.3: The GUI (Issue #3)

**Goal:** Build a world-class, intuitive GUI with excellent UX for managing rules.

#### Core Layout & Navigation

- [ ] MainLayout with collapsible sidebar navigation
- [ ] Sidebar items: Dashboard, Rules, Settings
- [ ] Responsive design (works at 1024px+ width)
- [ ] Dark/Light/System theme toggle in header
- [ ] Keyboard navigation support throughout

#### Dashboard Page

- [ ] Welcome card with app description and "How it works" diagram
- [ ] Quick stats: Total rules, Global rules, Local rules, Last sync time
- [ ] "Create your first rule" CTA when no rules exist (empty state)
- [ ] Quick-start templates carousel:
  - TypeScript Best Practices
  - React Component Guidelines
  - Python Standards
  - Git Commit Conventions
- [ ] Recent activity feed (last 5 sync operations)
- [ ] Quick actions: "New Rule", "Sync All"

#### Rules Page

- [ ] **Rules List View:**
  - Search bar with instant filtering
  - Sort dropdown (Name A-Z, Name Z-A, Date Modified, Scope)
  - Filter chips: Global | Local
  - Adapter filter dropdown
  - Rule cards showing: name, scope badge, adapter icons, last modified
  - Enable/disable toggle per rule (without deleting)
  - Overflow menu: Edit, Duplicate, Delete
  - Bulk selection with bulk actions (Enable/Disable/Delete)
- [ ] **Empty State:**
  - Illustration + "No rules yet"
  - "Create your first rule" primary button
  - "Browse templates" secondary button

- [ ] **Rule Editor (dedicated page/view):**
  - Three-panel layout: Sidebar (rule list) | Editor | Preview
  - Editor panel:
    - Rule name input (required, validated)
    - Markdown editor with formatting toolbar
    - Word/character count
    - Autosave indicator ("Saved" or "Saving...")
  - Settings panel (collapsible):
    - Scope toggle: Global | Local
    - Local paths: Directory picker with add/remove
    - Adapter selection: Checkboxes with icons and tooltips
  - Preview panel:
    - Tabbed view showing output for each selected adapter
    - File path display for each adapter output
    - "View in file system" link (opens folder)

#### Settings Page

- [ ] **General Settings:**
  - App data location (read-only + "Open in Explorer" button)
  - Default scope for new rules
  - Default adapter selections (remembered)
  - Theme: Light | Dark | System
- [ ] **Adapters Section:**
  - List of all adapters with enable/disable toggles
  - Each adapter shows: Name, Icon, Output filename, Global path
  - Tooltips explaining what each adapter is for
- [ ] **About Section:**
  - App version
  - "Check for updates" button
  - Links: GitHub, Documentation, Report Issue

#### Dialogs & Modals

- [ ] **Create Rule Dialog:**
  - Name input
  - "Start from template" dropdown
  - "Create blank" or "Use template" buttons
- [ ] **Delete Confirmation Dialog:**
  - Rule name prominently displayed
  - "Delete" (destructive) and "Cancel" buttons
  - Warning message about file sync implications
- [ ] **Conflict Resolution Dialog:**
  - Show file path and adapter
  - Show diff view (current vs incoming)
  - Options: "Overwrite", "Keep External", "Cancel"
- [ ] **Sync Preview Dialog:**
  - List of files that will be written
  - Green checkmark for new/modified files
  - Yellow warning for conflicts
  - "Sync Now" or "Cancel" buttons

#### Toast Notifications

- [ ] Success: Rule saved, Rule deleted, Sync completed
- [ ] Warning: File conflict detected, Large content warning
- [ ] Error: Save failed, Sync failed, Permission denied
- [ ] Undo toast for delete (10 second undo window)

#### Keyboard Shortcuts

- [ ] `Ctrl/Cmd + N` - New rule
- [ ] `Ctrl/Cmd + S` - Save current rule
- [ ] `Ctrl/Cmd + Shift + S` - Sync all
- [ ] `Ctrl/Cmd + F` - Focus search
- [ ] `Ctrl/Cmd + ,` - Open settings
- [ ] `Escape` - Close dialogs/cancel
- [ ] `?` - Show keyboard shortcuts help

#### Loading States & Feedback

- [ ] Skeleton loaders for rules list
- [ ] Spinner during sync operations
- [ ] Progress bar for bulk sync (X of Y files)
- [ ] Optimistic UI updates where appropriate
- [ ] Error boundaries with friendly error messages

#### Accessibility

- [ ] All interactive elements keyboard accessible
- [ ] Focus visible outlines
- [ ] ARIA labels for icons
- [ ] Screen reader announcements for toasts
- [ ] Sufficient color contrast (WCAG AA)
- [ ] Reduced motion respect

#### UI Component Inventory

| Component         | Purpose                              |
| ----------------- | ------------------------------------ |
| MainLayout        | App shell with sidebar               |
| Sidebar           | Navigation + collapse toggle         |
| Header            | Title, theme toggle, user menu       |
| Dashboard         | Welcome, stats, templates, activity  |
| RulesList         | Searchable, sortable list of rules   |
| RuleCard          | Individual rule display with actions |
| RuleEditor        | Markdown editing with toolbar        |
| RuleSettings      | Scope, paths, adapters config        |
| PreviewPanel      | Tabbed adapter output preview        |
| SettingsPage      | App preferences                      |
| AdapterConfig     | Adapter enable/disable + info        |
| EmptyState        | Illustrated empty rules state        |
| ConfirmDialog     | Destructive action confirmation      |
| ConflictDialog    | File conflict resolution             |
| SyncPreviewDialog | Pre-sync file list                   |
| Toast             | Success/warning/error notifications  |
| TemplatePicker    | Quick-start rule templates           |
| DirectoryPicker   | File system directory selection      |
| SearchInput       | Filter rules with debounce           |
| Toggle            | Enable/disable switches              |
| Badge             | Scope/adapter labels                 |
| Skeleton          | Loading placeholders                 |

#### Wireframes (ASCII Layout)

**Dashboard Page:**

```
┌────────────────────────────────────────────────────────────────────┐
│  RuleWeaver                                    🌙 Dark  ⚙ Settings │
├────────────┬───────────────────────────────────────────────────────┤
│            │  ┌─────────────────────────────────────────────────┐  │
│  Dashboard │  │  Welcome to RuleWeaver                          │  │
│            │  │  Manage your AI coding assistant rules in one   │  │
│  > Rules   │  │  place. Sync to GEMINI.md, AGENTS.md, .clinerules│  │
│            │  │                              [Create First Rule] │  │
│  Settings  │  └─────────────────────────────────────────────────┘  │
│            │                                                        │
│            │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐   │
│            │  │ 3 Rules      │ │ 2 Global     │ │ 1 Local      │   │
│            │  │ Total        │ │ Rules        │ │ Rules        │   │
│            │  └──────────────┘ └──────────────┘ └──────────────┘   │
│            │                                                        │
│            │  Quick Start Templates                                │
│            │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐      │
│            │  │TypeScript│ │ React   │ │ Python  │ │ Git     │      │
│            │  │  Best   │ │Components│ │Standard │ │ Commits │      │
│            │  │Practices│ │         │ │         │ │         │      │
│            │  └─────────┘ └─────────┘ └─────────┘ └─────────┘      │
│            │                                                        │
│            │  Recent Activity                    [Sync All ▸]       │
│            │  • Synced 3 rules to 5 files - 2 mins ago             │
│            │  • Created "TypeScript Standards" - 1 hour ago         │
└────────────┴───────────────────────────────────────────────────────┘
```

**Rules List Page:**

```
┌────────────────────────────────────────────────────────────────────┐
│  RuleWeaver                                    🌙 Dark  ⚙ Settings │
├────────────┬───────────────────────────────────────────────────────┤
│            │  Rules                              [+ New Rule]       │
│  > Dashboard│                                                      │
│            │  🔍 Search rules...        Sort: Modified ▾  Filter:  │
│  Rules     │                                      [Global] [Local]  │
│            │                                                        │
│  Settings  │  ┌──────────────────────────────────────────────────┐ │
│            │  │ 🟢 TypeScript Best Practices     GLOBAL    ⋮     │ │
│            │  │     Always use TypeScript for new...  Modified 2h │ │
│            │  │     📁 Gemini  📁 OpenCode  📁 Cline              │ │
│            │  └──────────────────────────────────────────────────┘ │
│            │  ┌──────────────────────────────────────────────────┐ │
│            │  │ 🟢 React Component Guidelines    GLOBAL    ⋮     │ │
│            │  │     Use functional components with... Modified 1d │ │
│            │  │     📁 Gemini  📁 OpenCode                        │ │
│            │  └──────────────────────────────────────────────────┘ │
│            │  ┌──────────────────────────────────────────────────┐ │
│            │  │ ⚪ Monorepo Standards           LOCAL     ⋮     │ │
│            │  │     Use turborepo for caching...     Modified 3d  │ │
│            │  │     📁 OpenCode  📁 Cline  Path: /home/user/repo  │ │
│            │  └──────────────────────────────────────────────────┘ │
└────────────┴───────────────────────────────────────────────────────┘
```

**Rule Editor Page:**

```
┌────────────────────────────────────────────────────────────────────┐
│  RuleWeaver                                    🌙 Dark  ⚙ Settings │
├────────────┬───────────────────────────────────────────────────────┤
│            │  ← Back to Rules                   [Preview] [Save]   │
│  Dashboard │                                                        │
│            │  ┌─────────────────────────┬────────────────────────┐ │
│  Rules     │  │ TypeScript Best Pract...│ Settings               │ │
│            │  │                         │                        │ │
│  Settings  │  │ # TypeScript Standards  │ Scope: ○ Global ● Local│ │
│            │  │                         │                        │ │
│            │  │ Always use TypeScript   │ Target Paths:          │ │
│            │  │ for all new projects.   │ 📁 /home/user/project  │ │
│            │  │                         │   [+ Add Path]         │ │
│            │  │ - Prefer interfaces     │                        │ │
│            │  │   over types            │ Adapters:              │ │
│            │  │ - Use strict mode       │ ☑ Gemini  GEMINI.md    │ │
│            │  │ - Enable noImplicitAny  │ ☑ OpenCode AGENTS.md   │ │
│            │  │                         │ ☐ Cline    .clinerules │ │
│            │  │                         │                        │ │
│            │  ├─────────────────────────┴────────────────────────┤ │
│            │  │ Preview                           Gemini ▾        │ │
│            │  │ ┌─────────────────────────────────────────────┐  │ │
│            │  │ │ <!-- Generated by RuleWeaver -->            │  │ │
│            │  │ │ ## TypeScript Standards                     │  │ │
│            │  │ │ Always use TypeScript...                   │  │ │
│            │  │ └─────────────────────────────────────────────┘  │ │
│            │  │ Will write to: ~/.gemini/GEMINI.md  [📁 Open]    │ │
│            │  └──────────────────────────────────────────────────┘ │
└────────────┴───────────────────────────────────────────────────────┘
```

**Sync Preview Dialog:**

```
┌─────────────────────────────────────────────────────────────────┐
│  Sync Preview                                              [✕]  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  The following 5 files will be updated:                         │
│                                                                 │
│  ✓ ~/.gemini/GEMINI.md              [Gemini]      Modified      │
│  ✓ ~/.opencode/AGENTS.md            [OpenCode]    Modified      │
│  ⚠ /project/GEMINI.md               [Gemini]      CONFLICT      │
│  ✓ /project/.clinerules             [Cline]       New           │
│  ✓ /project/AGENTS.md               [OpenCode]    Unchanged     │
│                                                                 │
│  ─────────────────────────────────────────────────────────────  │
│                                                                 │
│  ⚠ 1 conflict detected - external changes found                │
│                                                                 │
│                                         [Cancel]  [Resolve & Sync]│
└─────────────────────────────────────────────────────────────────┘
```

---

### Phase 1.4: The File Sync Engine (Issue #4)

**Goal:** Build adapter-based sync system with excellent user feedback and transparency.

#### Backend Implementation

- [ ] Define `SyncAdapter` Rust trait with format and path methods
- [ ] Implement `GeminiAdapter` for GEMINI.md format
- [ ] Implement `OpenCodeAdapter` for AGENTS.md format
- [ ] Implement `ClineAdapter` for .clinerules format
- [ ] Create SyncEngine orchestrator that runs all active adapters
- [ ] Implement file hash tracking for conflict detection
- [ ] Create IPC command `sync_rules` to trigger sync
- [ ] Create IPC command `preview_sync` to get planned changes without writing
- [ ] Handle edge cases (missing directories, permission errors)

#### Sync UX Flow

```
User clicks "Sync"
    ↓
Show Sync Preview Dialog
    - List all files that will be written
    - Highlight conflicts in yellow/warning
    - Show new files with green checkmark
    ↓
User confirms or cancels
    ↓
If conflicts exist → Show Conflict Resolution Dialog
    - For each conflict: show diff, allow overwrite/keep/cancel
    ↓
Execute sync with progress indicator
    - Per-file status updates
    - Success/error per file
    ↓
Show Sync Results Summary
    - X files written successfully
    - Y files skipped (conflicts kept)
    - Z errors
    - Clickable file paths to open in explorer
    ↓
Dismiss or view details
```

#### Sync Preview Dialog

- [ ] List view of all files to be written
- [ ] Status indicators: New, Modified, Conflict, Unchanged
- [ ] File path display with "Open folder" icon
- [ ] Adapter badge for each file
- [ ] "Dry Run" checkbox (preview only, no writes)
- [ ] "Sync" button (enabled only if no unresolved conflicts)
- [ ] Summary counts at top

#### Conflict Resolution Dialog

- [ ] File path prominently displayed
- [ ] Side-by-side diff view (Current vs Incoming)
- [ ] Line numbers for context
- [ ] Resolution options:
  - "Overwrite external changes"
  - "Keep external changes" (skip this file)
  - "Cancel sync"
- [ ] "Apply to all conflicts" checkbox

#### Sync Progress Indicator

- [ ] Modal overlay during sync
- [ ] Progress bar: "Writing file X of Y"
- [ ] Current file name display
- [ ] Per-file status ticks (✓ success, ✗ error)
- [ ] Cancel button (stops after current file)

#### Sync Results Summary

- [ ] Success count with green checkmark
- [ ] Conflict count (if any kept) with yellow warning
- [ ] Error count (if any) with red X
- [ ] Expandable list of all files with status
- [ ] Clickable paths open file in explorer
- [ ] "View Sync History" link

#### Sync History (Settings/Activity Page)

- [ ] Table of past sync operations
- [ ] Columns: Timestamp, Files Written, Status, Triggered By
- [ ] Expandable to see per-file details
- [ ] Filter by date range
- [ ] Export history as JSON/CSV

#### Adapter-Specific Headers

Each synced file includes a header identifying RuleWeaver as the source:

```markdown
<!-- Generated by RuleWeaver - Do not edit manually -->
<!-- Last synced: 2026-02-21T15:30:00Z -->
<!-- Rules: general-tech-stack, monorepo-standards -->

# [Adapter-specific content follows]
```

This allows:

- Conflict detection (if header missing or modified)
- Attribution back to RuleWeaver
- List of source rules in generated file

#### SyncAdapter Trait

```rust
pub trait SyncAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn file_name(&self) -> &str;
    fn icon(&self) -> &'static str; // Icon identifier for UI
    fn description(&self) -> &str;
    fn format_content(&self, rules: &[Rule]) -> String;
    fn get_target_path(&self, scope: &Scope, local_path: Option<&str>) -> PathBuf;
    fn is_enabled(&self, db: &Database) -> bool;
}
```

#### Adapter Output Formats

GEMINI.md:

```markdown
<!-- Generated by RuleWeaver - Do not edit manually -->
<!-- Last synced: 2026-02-21T15:30:00Z -->

<!-- Rule: General Tech Stack -->

Always use TypeScript.

<!-- Rule: Monorepo Standards -->

Use turborepo caching.
```

AGENTS.md:

```markdown
<!-- Generated by RuleWeaver - Do not edit manually -->
<!-- Last synced: 2026-02-21T15:30:00Z -->

## General Tech Stack

Always use TypeScript.

## Monorepo Standards

Use turborepo caching.
```

.clinerules:

```markdown
# Generated by RuleWeaver - Do not edit manually

# Last synced: 2026-02-21T15:30:00Z

# Rule: General Tech Stack

Always use TypeScript.

# Rule: Monorepo Standards

Use turborepo caching.
```

#### IPC Commands

- `sync_rules` - Execute full sync, return SyncResult
- `preview_sync` - Return planned changes without writing
- `resolve_conflict` - Resolve a specific conflict
- `get_sync_history` - Get paginated sync history
- `get_file_preview` - Preview formatted output for specific adapter

#### Error Handling UX

| Error Type          | User Message                                        | Recovery Action           |
| ------------------- | --------------------------------------------------- | ------------------------- |
| Permission denied   | "Cannot write to {path}. Check folder permissions." | "Open folder" button      |
| Directory not found | "Target directory doesn't exist"                    | "Create directory" option |
| File locked         | "File is in use by another application"             | "Retry" button            |
| Disk full           | "Not enough disk space"                             | None, show alert          |
| Path too long       | "File path exceeds system limit"                    | Suggest shorter path      |

#### Tests

- [ ] Unit tests for each adapter's format_content
- [ ] Unit tests for path resolution (global/local)
- [ ] Unit tests for conflict detection (hash matching)
- [ ] Unit tests for header generation
- [ ] Integration tests for full sync flow
- [ ] Integration tests for conflict resolution flow
- [ ] Integration tests for error scenarios

---

## 3. Execution Checklist

### Setup & Foundation

- [ ] Create feature branch `feature/phase-1-foundation`
- [ ] Initialize Tauri project with React + TypeScript
- [ ] Configure TailwindCSS with design tokens
- [ ] Install and setup shadcn/ui components
- [ ] Configure ESLint, Prettier, TypeScript strict mode
- [ ] Setup Vitest for frontend testing
- [ ] Setup Rust testing infrastructure
- [ ] Create directory structure per architecture.md

### Phase 1.2: Data Storage

- [ ] Create Rust models for Rule, Scope, TargetDefinition
- [ ] Add rusqlite and serde dependencies to Cargo.toml
- [ ] Implement database manager with connection pooling
- [ ] Create migrations system with initial schema
- [ ] Implement CRUD operations for rules
- [ ] Create custom error types for database operations
- [ ] Implement all Tauri IPC commands for rules
- [ ] Get AppData path working cross-platform
- [ ] Write unit tests for database manager
- [ ] Write unit tests for CRUD operations
- [ ] Write unit tests for IPC command handlers

### Phase 1.3: GUI

#### Layout & Navigation

- [ ] MainLayout with collapsible sidebar
- [ ] Sidebar navigation (Dashboard, Rules, Settings)
- [ ] Header with theme toggle
- [ ] Keyboard navigation support

#### Dashboard

- [ ] Welcome card with app description
- [ ] "How it works" visual diagram
- [ ] Stats display (total rules, global, local, last sync)
- [ ] Empty state with "Create first rule" CTA
- [ ] Quick-start templates carousel
- [ ] Recent activity feed

#### Rules List

- [ ] Search bar with instant filtering
- [ ] Sort dropdown
- [ ] Filter chips (Global/Local)
- [ ] Adapter filter dropdown
- [ ] Rule cards with enable/disable toggle
- [ ] Overflow menu (Edit, Duplicate, Delete)
- [ ] Bulk selection and actions
- [ ] Empty state with templates option

#### Rule Editor

- [ ] Three-panel layout (list, editor, preview)
- [ ] Rule name input with validation
- [ ] Markdown editor with formatting toolbar
- [ ] Word/character count
- [ ] Autosave indicator
- [ ] Settings panel (scope, paths, adapters)
- [ ] Preview panel with adapter tabs
- [ ] File path display with "Open folder" link

#### Settings Page

- [ ] App data location display
- [ ] "Open in Explorer" button
- [ ] Default scope selection
- [ ] Default adapters selection
- [ ] Theme toggle (Light/Dark/System)
- [ ] Adapters enable/disable section
- [ ] About section with version

#### Dialogs

- [ ] Create Rule dialog with template picker
- [ ] Delete confirmation dialog
- [ ] Conflict resolution dialog with diff view
- [ ] Sync preview dialog

#### Feedback & Notifications

- [ ] Toast notification system
- [ ] Undo toast for deletions
- [ ] Loading skeletons
- [ ] Error boundaries
- [ ] Optimistic UI updates

#### Keyboard Shortcuts

- [ ] Ctrl+N (New rule)
- [ ] Ctrl+S (Save)
- [ ] Ctrl+Shift+S (Sync)
- [ ] Ctrl+F (Search)
- [ ] Ctrl+, (Settings)
- [ ] ? (Show shortcuts)

#### Accessibility

- [ ] Keyboard accessible all elements
- [ ] Focus visible outlines
- [ ] ARIA labels
- [ ] Screen reader announcements
- [ ] Color contrast (WCAG AA)
- [ ] Reduced motion respect

#### Tests

- [ ] Tests for MainLayout
- [ ] Tests for RulesList
- [ ] Tests for RuleEditor
- [ ] Tests for RuleSettings
- [ ] Tests for Settings page
- [ ] Tests for all dialogs
- [ ] Accessibility audit

### Phase 1.4: Sync Engine

#### Backend

- [ ] SyncAdapter trait definition
- [ ] GeminiAdapter implementation
- [ ] OpenCodeAdapter implementation
- [ ] ClineAdapter implementation
- [ ] SyncEngine orchestrator
- [ ] File hash tracking
- [ ] sync_rules IPC command
- [ ] preview_sync IPC command
- [ ] resolve_conflict IPC command
- [ ] get_sync_history IPC command

#### Sync UX

- [ ] Sync preview dialog
- [ ] Conflict resolution dialog with diff
- [ ] Sync progress indicator
- [ ] Sync results summary
- [ ] Sync history page/section
- [ ] Error handling with recovery actions

#### Tests

- [ ] Unit tests for adapter format_content
- [ ] Unit tests for path resolution
- [ ] Unit tests for conflict detection
- [ ] Unit tests for header generation
- [ ] Integration tests for sync flow
- [ ] Integration tests for conflict resolution
- [ ] Integration tests for error scenarios

### Polish & Verification

- [ ] Run all lint commands (ESLint, Clippy)
- [ ] Run all type checks (TypeScript, Rust)
- [ ] Achieve 80% test coverage on new code
- [ ] Manual end-to-end testing on Windows
- [ ] Verify database persistence across app restarts
- [ ] Verify sync writes files to correct locations
- [ ] Review UI for consistency and polish
- [ ] Ensure no hardcoded colors remain

---

## 4. Detailed Implementation Guides

### Frontend State Management (Zustand)

```typescript
// stores/rulesStore.ts
interface RulesState {
  rules: Rule[];
  selectedRule: Rule | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchRules: () => Promise<void>;
  createRule: (rule: CreateRuleInput) => Promise<void>;
  updateRule: (id: string, rule: UpdateRuleInput) => Promise<void>;
  deleteRule: (id: string) => Promise<void>;
  selectRule: (rule: Rule | null) => void;
}

// stores/syncStore.ts
interface SyncState {
  isSyncing: boolean;
  lastSyncTime: Date | null;
  syncErrors: SyncError[];
  conflicts: Conflict[];

  syncRules: () => Promise<void>;
  resolveConflict: (conflictId: string, resolution: "overwrite" | "skip") => void;
}
```

### TypeScript Types (mirroring Rust models)

```typescript
// types/rule.ts
export type Scope = "global" | "local";

export interface Rule {
  id: string;
  name: string;
  content: string;
  scope: Scope;
  targetPaths: string[] | null;
  enabledAdapters: AdapterType[];
  createdAt: number;
  updatedAt: number;
}

export type AdapterType = "gemini" | "opencode" | "cline";

export interface CreateRuleInput {
  name: string;
  content: string;
  scope: Scope;
  targetPaths?: string[];
  enabledAdapters: AdapterType[];
}

export interface SyncResult {
  success: boolean;
  filesWritten: string[];
  errors: SyncError[];
  conflicts: Conflict[];
}

export interface Conflict {
  filePath: string;
  adapterName: string;
  localHash: string;
  currentHash: string;
}
```

### Tauri IPC Integration Layer

```typescript
// lib/tauri.ts
import { invoke } from "@tauri-apps/api";

export const api = {
  rules: {
    getAll: () => invoke<Rule[]>("get_all_rules"),
    getById: (id: string) => invoke<Rule>("get_rule_by_id", { id }),
    create: (input: CreateRuleInput) => invoke<Rule>("create_rule", { input }),
    update: (id: string, input: UpdateRuleInput) => invoke<Rule>("update_rule", { id, input }),
    delete: (id: string) => invoke<void>("delete_rule", { id }),
  },
  sync: {
    syncRules: () => invoke<SyncResult>("sync_rules"),
    getAppDataPath: () => invoke<string>("get_app_data_path"),
    resolveConflict: (conflictId: string, resolution: string) =>
      invoke<void>("resolve_conflict", { conflictId, resolution }),
  },
};
```

### Markdown Editor Implementation

Use `@uiw/react-md-editor` or similar with:

- Split-pane live preview
- Syntax highlighting
- Toolbar with common markdown actions
- Controlled component pattern for form integration

```typescript
// components/RuleEditor.tsx
import MDEditor from '@uiw/react-md-editor';

export function RuleEditor({ value, onChange }: RuleEditorProps) {
  return (
    <div className="h-full" data-color-mode="system">
      <MDEditor
        value={value}
        onChange={onChange}
        preview="live"
        height="100%"
        visibleDragbar={false}
      />
    </div>
  );
}
```

### Adapter Selection UI

```typescript
// components/AdapterSelector.tsx
const ADAPTERS = [
  { id: "gemini", name: "Gemini CLI", file: "GEMINI.md", icon: GeminiIcon },
  { id: "opencode", name: "OpenCode", file: "AGENTS.md", icon: OpenCodeIcon },
  { id: "cline", name: "Cline", file: ".clinerules", icon: ClineIcon },
] as const;

// Checkbox group for selecting which adapters receive this rule
```

### Conflict Resolution Flow

1. User clicks "Sync Now"
2. Backend checks file hashes for all target files
3. If hash mismatch detected:
   - Return conflicts in SyncResult
   - Show modal with conflict details
   - User chooses: "Overwrite" | "Keep External Changes" | "Cancel"
4. On resolution, re-run sync with user's choice

### Error Handling Pattern (Rust)

```rust
// src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Rule not found: {id}")]
    RuleNotFound { id: String },

    #[error("Sync conflict detected: {file_path}")]
    SyncConflict { file_path: String },

    #[error("Invalid input: {message}")]
    InvalidInput { message: String },
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}
```

### Database Schema (SQLite)

```sql
-- migrations/001_initial.sql
CREATE TABLE rules (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    content TEXT NOT NULL,
    scope TEXT NOT NULL CHECK(scope IN ('global', 'local')),
    target_paths TEXT, -- JSON array
    enabled_adapters TEXT NOT NULL, -- JSON array
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE sync_history (
    id TEXT PRIMARY KEY NOT NULL,
    file_path TEXT NOT NULL UNIQUE,
    content_hash TEXT NOT NULL,
    last_sync_at INTEGER NOT NULL
);

CREATE INDEX idx_rules_scope ON rules(scope);
```

### Rust IPC Commands Structure

```rust
// src/commands/rules.rs
#[tauri::command]
pub async fn get_all_rules(db: State<'_, Database>) -> Result<Vec<Rule>, AppError> {
    db.get_all_rules().await
}

#[tauri::command]
pub async fn create_rule(input: CreateRuleInput, db: State<'_, Database>) -> Result<Rule, AppError> {
    // Validate input
    // Generate UUID
    // Set timestamps
    // Insert into database
    // Return created rule
}

// src/commands/sync.rs
#[tauri::command]
pub async fn sync_rules(
    db: State<'_, Database>,
    sync_engine: State<'_, SyncEngine>,
) -> Result<SyncResult, AppError> {
    let rules = db.get_all_rules().await?;
    sync_engine.sync_all(rules).await
}
```

### Testing File Structure

```
src-tauri/
├── src/
│   └── ...
└── tests/
    ├── database_tests.rs
    ├── sync_adapter_tests.rs
    └── integration_tests.rs

src/
├── __tests__/
│   ├── components/
│   │   ├── RulesList.test.tsx
│   │   ├── RuleEditor.test.tsx
│   │   └── AdapterSelector.test.tsx
│   ├── stores/
│   │   └── rulesStore.test.ts
│   └── lib/
│       └── tauri.test.ts
```

---

## 5. Technical Specifications

### Dependencies (Rust)

- `tauri` - Desktop framework
- `rusqlite` - SQLite database
- `serde` / `serde_json` - Serialization
- `thiserror` - Error handling
- `uuid` - ID generation
- `sha2` - File hashing

### Dependencies (Frontend)

- `react` / `react-dom` - UI framework
- `@tauri-apps/api` - Tauri frontend API
- `tailwindcss` - Styling
- `@radix-ui/*` - shadcn/ui primitives
- `react-hook-form` - Form handling
- `zod` - Validation
- `lucide-react` - Icons
- `zustand` - State management

### Testing Strategy

- Frontend: Vitest + Testing Library for component tests
- Backend: Rust built-in test framework for unit tests
- Integration: Test full sync flow with temp directories
- Coverage target: 80% (high-value tests only, no low-value tests)

### File Locations

- Database: `%APPDATA%/RuleWeaver/rules.db` (Windows)
- Global GEMINI.md: `~/.gemini/GEMINI.md`
- Global AGENTS.md: `~/.opencode/AGENTS.md`
- Local files: `{repo_path}/GEMINI.md`, `{repo_path}/AGENTS.md`, `{repo_path}/.clinerules`

---

## 6. Implementation Order & Dependencies

```
Issue #1 (Project Setup)
    │
    ├──► Issue #2 (Data Storage) ──► Issue #3 (GUI) ──► Issue #4 (Sync Engine)
    │                                      │                    │
    │                                      └── Uses IPC ────────┘
    │                                            │
    └── All issues depend on base project setup ──┘
```

### Recommended Build Sequence

1. **Day 1-2:** Issue #1 - Full project setup including all tooling
2. **Day 3-5:** Issue #2 - Data layer complete with all tests passing
3. **Day 6-10:** Issue #3 - GUI with basic read/write to database
4. **Day 11-14:** Issue #4 - Sync engine integration with GUI
5. **Day 15:** Polish, final testing, documentation

### Issue Dependencies

- **Issue #2 requires:** Issue #1 (project structure, Rust setup)
- **Issue #3 requires:** Issue #2 (IPC commands to fetch/save rules)
- **Issue #4 requires:** Issue #2 (rules from database), Issue #3 (sync button UI)

---

## 7. CI/CD Setup

### GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: "20" }
      - run: npm ci
      - run: npm run lint
      - run: npm run typecheck
      - run: npm run test

  backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: cargo clippy -- -D warnings
      - run: cargo test

  build:
    needs: [frontend, backend]
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - run: npm ci
      - uses: tauri-apps/tauri-action@v0
```

### Package Scripts

```json
{
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "lint": "eslint src --ext .ts,.tsx",
    "lint:fix": "eslint src --ext .ts,.tsx --fix",
    "typecheck": "tsc --noEmit",
    "test": "vitest run",
    "test:watch": "vitest",
    "test:coverage": "vitest run --coverage"
  }
}
```

---

## 8. Edge Cases & Error Scenarios

### File System Edge Cases

| Scenario                       | Handling                                     |
| ------------------------------ | -------------------------------------------- |
| Target directory doesn't exist | Create recursively with `fs::create_dir_all` |
| Permission denied              | Return error, show toast with path info      |
| File locked by another process | Retry with backoff, then fail with message   |
| Disk full                      | Return IO error, show user-friendly message  |
| Path too long (Windows)        | Validate paths before sync, show warning     |

### Data Edge Cases

| Scenario                        | Handling                                      |
| ------------------------------- | --------------------------------------------- |
| Rule with empty content         | Allow but warn in UI                          |
| Rule with no adapters selected  | Block save, show validation error             |
| Local rule with no target paths | Block save, require at least one path         |
| Duplicate rule name             | Allow (names not unique), use ID for identity |
| Very large rule content (>1MB)  | Warn user, suggest splitting                  |

### Sync Edge Cases

| Scenario                               | Handling                                        |
| -------------------------------------- | ----------------------------------------------- |
| External file modified since last sync | Detect via hash, prompt for conflict resolution |
| Multiple rules targeting same file     | Concatenate in defined order                    |
| Adapter format changes needed          | Track version, re-format on next sync           |
| Sync interrupted mid-way               | Each file is atomic, resume on next sync        |

---

## 9. Security Considerations

### File System Access

- Use Tauri's allowlist to restrict filesystem access
- Only allow writes to designated paths (user-selected, AppData, home)
- Never execute files from user-provided paths (Phase 1 - no command execution)

### Input Validation

- Sanitize rule names (no path traversal characters)
- Validate target paths are within allowed scopes
- Limit content size to prevent memory issues

### Database

- Store in user's AppData (OS-appropriate location)
- No sensitive data in Phase 1 (secrets come in Phase 5)

---

## 10. Definition of Done

Each issue is complete when:

- [ ] All code implemented per specifications
- [ ] All unit tests passing (80%+ coverage, high-value only)
- [ ] Integration tests passing
- [ ] Lint and typecheck passing with no warnings
- [ ] Manual testing completed on Windows
- [ ] Code reviewed
- [ ] No TODOs or placeholders in code
- [ ] Error handling comprehensive
- [ ] Loading states and error messages in UI
- [ ] Accessibility verified (keyboard navigation works)
