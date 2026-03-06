---
type: "agent_requested"
description: "When completing a plan"
---

# Workflow: Check Plan & Verify Progress
# Trigger: /check-plan

## Context
The user wants to verify the current state of the project against the established `PLAN.md`.

## Steps
1.  **Read Context**
    * Read the content of `PLAN.md`.
    * Scan the current codebase (file structure, implemented functions, recent changes).

2.  **Gap Analysis**
    * Compare the *actual* code implementation against the *planned* steps in `PLAN.md`.
    * Identify items that appear complete but are not marked as checked.
    * Identify items marked as checked that appear incomplete or broken.

3.  **Action: Update Plan**
    * Edit `PLAN.md` to reflect the current reality.
    * Mark completed items with `- [x]`.
    * Add notes next to items that are "In Progress" or "Blocked".

4.  **Report**
    * Output a summary to the user:
        * **Completed:** [List of newly completed items]
        * **Pending:** [Immediate next steps]
        * **Discrepancies:** [Any items that were missed or implemented differently than planned]

5.  **Prompt for Next Step**
    * Ask the user if they want to proceed with the next unchecked item in the checklist.