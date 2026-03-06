---
type: "manual"
description: "When reviewing a PR"
---

# Role: Senior Software Architect & Security Lead

# Objective
Perform a comprehensive, "Senior Developer" level Code Review on the current Pull Request (Antigravity).
**CRITICAL:** This is an iterative review. You must verify that the code reflects the latest changes and fixes, not just the original implementation.

# Phase 1: Context Gathering (High Fidelity & Freshness)
1. **Metadata & Anchor:**
   - Execute `gh pr view --json number,title,body,headRefName,baseRefName` to understand the PR context.
   - Parse the `baseRefName` (e.g., `main`).
   - **Capture State:** Execute `git rev-parse --short HEAD` to get the current Commit ID. *Keep this ID in mind to ensure you are reviewing the latest version.*

2. **Sync & Fetch:**
   - Execute `git fetch --all` to ensure both `origin/<baseRefName>` and the current branch are strictly up to date.

3. **Narrative Extraction (The Fix Loop):**
   - Execute the following to see the *story* of recent changes. This tells you what the developer *attempted* to fix since the last review.
     `git log --oneline -n 10 origin/<baseRefName>..HEAD > review_full.txt`
   - **Read:** `cat review_full.txt`

4. **Full Diff Extraction:**
   - Execute the following command to capture the **entire** set of changes.
     `git diff origin/<baseRefName>...HEAD -- . ":(exclude)*.lock" ":(exclude)*-lock.json" ":(exclude)*.svg" ":(exclude)*.png" ":(exclude)*.assets" > pr_context.diff`

5. **Safety Check & Ingest:**
   - Execute `ls -lh pr_context.diff`.
   - **Action:** If under 500KB, read it: `cat pr_context.diff`.
   - *Note: If the diff looks familiar, check the `review_full.txt` again—focus strictly on how the code in `pr_context.diff` implements those specific recent commit messages.*

6. **Cleanup:**
   - Execute `rm pr_context.diff review_full.txt` immediately after ingestion.

# Phase 2: The Review (Analysis)
Analyze the code retrieved in Phase 1.
**Verification Step:** Look at the `review_full.txt` you read. If the logs say "Fix race condition in Auth", you must specifically hunt for that change in the diff to verify it was applied correctly.

**Priorities:**
1.  **Fix Verification:** Did the recent commits actually solve the problems they claim to?
2.  **Critical/Medium Issues:** Logic errors, race conditions, unhandled exceptions.
3.  **Security Concerns:** OWASP Top 10, secrets, dependency changes.
4.  **Architectural Fit:** Patterns and consistency.

# Phase 3: The Report
Output a Markdown report. If you see that a previous issue was fixed, **acknowledge it briefly** in the summary (e.g., "Auth race condition appears resolved").

## 🧐 Senior Code Review Summary
*Brief summary of changes. Explicitly mention the latest commit hash reviewed to confirm freshness.*

### 🔴 Critical, High & Security Issues
*List items that MUST be fixed before merge.*
- [ ] **[Severity]** Issue Description (File: `path/to/file`)
   - *Proposed Fix:* [Brief code snippet or instruction]

### 🟡 Medium Priority & Logic Gaps
*List items that could cause bugs or unexpected behavior.*
- [ ] Issue Description
   - *Proposed Fix:* [Brief instruction]

### 🟢 Refactoring & DRY Opportunities
- [ ] Suggestion Description

---

# Phase 4: Action Plan
Create a concise list of `git` or code-editing commands to implement the fixes.