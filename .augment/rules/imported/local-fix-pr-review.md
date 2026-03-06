---
type: "manual"
---

# Role: Senior Engineer & Review Implementer

# Goal
Receive a manually pasted "Senior Code Review," analyze the validity of the claims, apply necessary fixes to the codebase, and handle git operations.

# Input Format
The user will paste a review in this specific Markdown format:
- `## 🧐 Senior Code Review Summary`
- `### 🔴 Critical...`
- `### 🟡 Medium...`
- `### 🟢 Refactoring...`

# Procedure

## Step 1: Analyze & Triage
For every checkbox item in the pasted review:
1. Identify the target file (e.g., `src/utils.ts`).
2. Analyze the current code in that file against the Review Comment.
3. **CRITICAL STEP:** Determine if the review is Valid or Invalid.
   - **Valid:** The code has the bug/issue described.
   - **Invalid:** The reviewer is missing context, the code is actually correct, or the suggestion introduces regressions.
  **Verify Claims:**
    * **Hallucination Check:** Does the code actually contain the logic the reviewer is citing?
    * **Context Check:** Is the reviewer missing context (e.g., legacy constraints, external dependencies, or middleware that handles the issue)?
    * **Loop Prevention:** Does the review suggest reverting a change we plainly made on purpose? If so, STOP. usage `notify_user` to ask: "The reviewer wants to revert [X]. We did this because [Y]. Should I revert or maintain?"
    * **Value Check:** For `🟢 Refactoring`, does the suggestion objectively improve code health, or is it purely subjective/risky?
3.  **Triage Decision:**

## Step 2: Implementation
- Apply fixes for all **Valid** items.
- Prioritize 🔴 Critical and 🟡 Medium issues.
- Apply 🟢 Refactoring only if it significantly improves code health without breaking changes.
- **Do not** change code for items deemed **Invalid**.

## Step 3: Reporting
Output a summary to the user:
- ✅ **Implemented:** [List of changes made]
- ❌ **Skipped/Disagreed:** [List of review items you rejected and why (e.g., "Reviewer was incorrect about variable scope")]

## Step 4: Git
After implementation and reporting:
- `git add .`
- `git commit -m "fix: address manual code review feedback"`
- `git push`