# RuleWeaver Depth Plan

This plan prioritizes **depth-first** improvements that make current RuleWeaver capabilities more reliable, trustworthy, and useful without expanding into unrelated new feature areas.

## Prioritized Execution Order

1. **Canonical Tool Capability Registry**  
   Why first: every downstream depth initiative depends on one source of truth for capabilities and paths.

2. **Path/Scope Resolution Invariants Across Artifacts**  
   Why second: resolves correctness baseline for preview/sync/cleanup/reconcile across rules/commands/skills/slash.

3. **Cross-Artifact Reconciliation Engine**  
   Why third: enforces deterministic desired-state convergence and stale artifact cleanup.

4. **Full Import Parity for Commands + Skills**  
   Why fourth: unlocks migration value and lifecycle parity with rules.

5. **Slash Command Lifecycle Completeness**  
   Why fifth: ensures generated slash assets remain correct through save/rename/delete/re-target operations.

6. **Deterministic Conflict Handling + Race Mitigation**  
   Why sixth: increases operator confidence in preview and conflict outcomes.

7. **Unified Artifact Status + One-Click Repair UX**  
   Why seventh: exposes drift and health in one place once underlying lifecycle guarantees are in place.

8. **Skills Native Distribution + Capability-Aware Targeting**  
   Why eighth: deepens existing skills value by completing cross-tool lifecycle behavior.

9. **Command Execution Reliability, Safety, and Diagnostics**  
   Why ninth: improves trust and debuggability of existing command execution workflows.

10. **Docs/Runtime Truthfulness Automation**  
    Why tenth: locks in long-term consistency after core lifecycle hardening is complete.

## Issue Mapping (Depth Milestone)

1. Canonical registry -> **#31**
2. Path/scope invariants -> **#41**
3. Reconciliation engine -> **#39**
4. Import parity commands + skills -> **#59**
5. Slash lifecycle completeness -> **#43**
6. Conflict determinism + race mitigation -> **#58**
7. Unified status + repair UX -> **#47**
8. Skills distribution parity -> **#45**
9. Command execution reliability/safety -> **#57**
10. Docs truthfulness automation -> **#46**

## Notes on Existing Overlap

- Existing lifecycle issues under Milestone #6 overlap heavily with this depth strategy.
- This Depth plan consolidates those overlaps into a **clear execution sequence** and fills uncovered areas with new issues where necessary.
- Related existing issues that remain complementary:
  - `#33` skills import pipeline
  - `#37` command/workflow import pipeline
  - `#38` capability/path consistency checks
  - `#40` post-import reconciliation
  - `#42` frontend metadata replacement
  - `#44` lifecycle integration tests
