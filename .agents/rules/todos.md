---
trigger: always_on
---

# AI Coding Standards & Behavioral Rules

## 1. Test Coverage ("Antigravity" Protocol)

**Priority:** High-Value Verification > 80% Coverage Metric

- **Target:** Aim for 80% test coverage on all new or modified code.
- **Strict Constraint:** You are **prohibited** from generating "low-value" tests solely to meet this numeric target.
  - _Definition of Low Value:_ Tests that only verify simple getters/setters, constants, or trivial pass-through functions without logic.
- **Resolution:** If the only way to achieve 80% coverage is to add low-value tests, **stop**. Submit the code with the lower coverage percentage. Do not inflate the metric with brittle or redundant code.

## 2. Code Completeness (No "TODOs")

**Directive:** Zero Tolerance for Placeholders.

- **Strict Prohibition:** You must **NOT** output comments such as `// TODO`, `// FIXME`, or `// IMPLEMENT THIS` in the final code block.
- **Requirement:** If you identify a necessary logical step, edge case, or requirement, you must generate the working implementation code for it immediately.
- **Stop Sequence:** If a specific piece of logic is impossible to implement because you lack necessary context (e.g., an API key, a user file, or a specific library), you must **STOP** generating code and ask the user for that specific context. Do not hallucinate a solution or leave a placeholder.
