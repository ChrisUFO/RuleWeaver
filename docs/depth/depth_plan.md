# RuleWeaver Depth Plan

This plan prioritizes **depth-first** improvements that make current RuleWeaver capabilities more reliable, trustworthy, and useful without expanding into unrelated new feature areas.

## Prioritized Execution Order

### Group 1 - Single Source of Truth and Validation Baseline

1. **Canonical Tool Capability Registry** (`#31`)  
   Establish the authoritative model for capability/path support.
2. **Frontend Metadata Replacement** (`#42`)  
   Remove duplicated frontend adapter metadata and consume shared registry data.
3. **Capability/Path Consistency Checks** (`#38`)  
   Enforce runtime/test validation to catch unsupported combinations and path drift early.

### Group 2 - Path Correctness and Deterministic Lifecycle

4. **Path/Scope Resolution Invariants Across Artifacts** (`#41`)  
   Standardize resolver behavior for preview/sync/write/cleanup across all artifacts.
5. **Cross-Artifact Reconciliation Engine** (`#39`)  
   Make rename/delete/deselect operations converge cleanly without stale files.

### Group 3 - Import Completeness and Post-Import Convergence

6. **Command/Workflow Import Pipeline** (`#37`)  
   Add scan/preview/execute parity for commands.
7. **Skills Import Pipeline** (`#33`)  
   Add scan/preview/execute parity for skills.
8. **Full Post-Import Reconciliation** (`#40`)  
   Ensure imports trigger cleanup/sync across all generated artifacts.
9. **Full Import Parity (Unified Delivery)** (`#59`)  
   Consolidate command+skill import lifecycle parity into one operator-grade workflow.

### Group 4 - Slash and Conflict Hardening

10. **Slash Lifecycle Completeness** (`#43`)  
    Harden cleanup/remove/autosync/status behavior for slash files.
11. **Deterministic Conflict Handling + Race Mitigation** (`#58`)  
    Improve confidence in preview conflict detection and resolution outcomes.

### Group 5 - Operator Experience and Artifact Distribution Depth

12. **Unified Artifact Status + One-Click Repair UX** (`#47`)  
    Provide one health surface with actionable remediation.
13. **Skills Native Distribution + Capability-Aware Targeting** (`#45`)  
    Complete native skills lifecycle delivery where supported.
14. **Command Execution Reliability, Safety, and Diagnostics** (`#57`)  
    Improve runtime trust with stronger diagnostics and safety controls.

### Group 6 - Quality Gate and Docs Truthfulness

15. **Lifecycle Integration Tests and Coverage Gates** (`#44`)  
    Lock behavior with integration coverage across the lifecycle matrix.
16. **Docs/Runtime Truthfulness Automation** (`#46`)  
    Keep README/guide/reference aligned with shipped behavior and registry output.

## Depth Milestone Issues

- `#31` canonical tool capability registry
- `#42` replace duplicated frontend adapter metadata
- `#38` capability/path consistency checks
- `#41` adapter-specific local path resolver / path invariants
- `#39` artifact reconciliation engine
- `#37` command/workflow import pipeline
- `#33` skills import pipeline
- `#40` full post-import reconciliation
- `#59` full import parity for commands and skills
- `#43` slash lifecycle hardening
- `#58` deterministic conflict handling and race-safe preview
- `#47` unified artifact status and repair actions
- `#45` skills sync adapters with capability-aware targeting
- `#57` command execution reliability, safety, and diagnostics
- `#44` integration test matrix and coverage gates
- `#46` align architecture/user docs with shipped behavior

## Dependency Notes

- `#31` -> `#42`, `#38`, `#41`, `#39`, `#46`
- `#41` -> `#39`, `#43`, `#58`, `#47`
- `#37` + `#33` + `#40` -> `#59`
- `#39` + `#43` + `#58` -> `#47`
- `#44` is a milestone-wide quality gate and should run continuously while Groups progress.
