---
type: "manual"
---

I need a major refactor to improve maintainability, DRY compliance, and test coverage. Please follow this specific plan:

1.  **Identify Targets**
    * Run a terminal command to find the top 3 largest source code files in the `src` directory by line count.
    * *Exclude:* `node_modules`, `dist`, `build`, lock files, `json`, and test files.

2.  **Contextual Analysis**
    * For each of these 5 "God Files," read the file AND scan the other files located in the same directory.
    * Understand the primary responsibility of the God File vs. its neighbors.

3.  **Test Coverage Verification**
    * Check the existing tests for these files.
    * specific step: Compare the current coverage against **our test coverage rule**.
    * Identify gaps where the rule is not currently met.

4.  **DRY Strategy (De-duplication)**
    * Look for patterns, utility functions, or UI elements that are repeated across these sibling files.
    * Identify logic that should be lifted into a shared `utils`, `hooks`, or `components` file.

5.  **Refactor Plan**
    * Propose a plan to extract internal logic from the God Files into smaller, single-responsibility sub-modules.
    * Propose where to move the shared logic found in step 4.
    * Propose a plan to add/update unit tests to ensure we satisfy **our test coverage rule** for both the new and refactored files.
    * **STOP HERE.** Present the plan (including new file names and locations) and wait for my approval before writing code.

6.  **Execution (After Approval)**
    * Refactor the files one by one.
    * Update all import paths.
    * Write the necessary tests to meet the coverage rule.
    * Ensure the application compiles and all tests pass after each file is processed.