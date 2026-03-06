---
type: "manual"
---

# Workflow: GitHub PR Review & Remediation

## Usage
Use the slash command with a GitHub review URL:

`/fix-pr-review <GITHUB_REVIEW_URL>`

Example:
`/fix-pr-review https://github.com/ChrisUFO/GateZero/pull/334#pullrequestreview-3667347907`

## 1. Extract Review Data
The PR review URL is provided in the variable: `$ARGUMENTS`.

### Action Items for the Agent:
1.  **Parse the URL**: Extract the `owner`, `repo`, `pull_number`, and `review_id` from the URL provided in `$ARGUMENTS`.
2.  **Fetch Review Details**:
    *   Use `mcp_github-mcp-server_pull_request_read` with `method="get_reviews"` to find the review matching the `review_id`. Capture the `body` (the review summary).
    *   Use `mcp_github-mcp-server_pull_request_read` with `method="get_review_comments"` to fetch all inline comments for this PR, then filter them by the `pull_request_review_id` matching our `review_id`.
3.  **Prepare for Triage**: Consolidate the review summary and all relevant inline comments for analysis.

## Phase 1: Validation Audit (The "Brain")
**Constraint:** Do not apply fixes yet. For every comment found, perform a Reality Check loop:

*   **Locate Context**: Open the specific file path mentioned in the comment.
*   **Verify Claims**: Critically assess the reviewer's feedback against the actual code behavior.
*   **Hallucination Check**: Does the code actually function the way the reviewer claims?
*   **Context Check**: Is the reviewer missing context (e.g., is this logic handled by middleware, a parent class, or legacy constraints)?
*   **Loop Prevention**: Does the review recommend reverting a change we specifically made intentionally (e.g. "put this back to how it was")? If so, STOP and call `notify_user` to ask: "The reviewer suggested reverting [X]. We changed this intentionally because [Y]. Should I revert it or explain why it's needed?"
*   **Correctness Check**: Is the proposed solution factually correct, or will it break other logic?

### Triage Decision:
*   Mark valid, actionable items as **ACCEPTED**.
*   Mark incorrect, dangerous, or out-of-context items as **REJECTED**.

## Phase 2: Execution (The "Hands")
Iterate through the **ACCEPTED** items only.

### Prioritize Fixes:
1.  **Priority 1**: Security issues & Critical bugs.
2.  **Priority 2**: Logic gaps & Functional errors.
3.  **Priority 3**: Refactoring & Style (DRY, naming conventions).

**Implementation:** Apply the code changes to the local files.
**Skip:** Do not modify code for items marked **REJECTED**.

## Phase 3: Finalization & Git
### Report: Output a final summary to the user:
*   ✅ **Fixed**: [List of items implemented]
*   ❌ **Skipped/Disagreed**: [List of items rejected and the reason why]

### Commit and Push
1.  Stage all changes: `git add .`
2.  Commit with a message referencing the PR: `git commit -m "fix: resolve PR reviews for #<PR_NUMBER>"`
3.  Push the changes: `git push`