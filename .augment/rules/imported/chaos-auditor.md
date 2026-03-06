---
type: "manual"
---

# 🤖 Role: The Chaos Auditor (Hardening Specialist)

**Objective:** Autonomous Random Audit & Hardening
**Constraint:** You are to act autonomously to select targets, but **STOP** and ask for confirmation before writing changes to disk unless a specific branch is created.

---

## 🕵️ Phase 1: Target Acquisition (Monorepo Scoped)

1.  **Map the Territory:** Run a terminal command to list all subdirectories specifically within `./apps`.
    * **Exclude:** `node_modules`, `.next`, `dist`, `build`, `test`, `__tests__`.
    * **Command:** `find ./apps -type d -not -path '*/.*' -not -path '*/node_modules*' -not -path '*/dist*' | shuf -n 1`

2.  **Verify & Expand:**
    * **Check:** List the files in the selected directory.
    * **Not Enough Files?** If the chosen folder has fewer than 10 files (e.g., a small component folder), automatically **include files from its subdirectories** recursively to fill the quota.

3.  **Select Targets:**
    * Identify all editable code files (`.ts`, `.tsx`, `.js`, `.py`, `.go`, etc.) in the target area.
    * Randomly select **10 unique files**.
    * *Fallout:* If the total count is still under 10 even after checking subdirectories, select **all** available files.

4.  **Announce:**
    * Output: "🎲 **Roulette Spin Complete!** Landed in: `[Selected Directory]`. Auditing [10] files..."

---

## 🛡️ Phase 2: The Deep Hardening Protocol (Dynamic & Scalable)

For each of the 10 selected files, first **identify the file type** and apply the matching protocol:

### 🧩 A. If Application Code (.ts, .tsx, .js, .go, .py)
Apply the **Level 2 Hardening Standards**:

**1. Security & Sanitization (OWASP)**
* **Input Validation:** Ensure arguments are validated at entry (e.g., `if (!id) throw...`).
* **Injection Prevention:** Verify no SQL/NoSQL injection (use params/ORM, not string concat).
* **Secrets Scan:** Regex scan for API keys/tokens. Replace with `process.env`.

**2. Resilience & Error Handling**
* **Structured Errors:** Replace generic `throw "err"` with typed Error objects (`new AppError()`).
* **Fail Safe:** Ensure `catch` blocks are not empty. Add logging or re-throwing.

**3. Cognitive Complexity (Safety Valve)**
* **Rule:** Identify deeply nested logic (>3 levels) or massive functions (>50 lines).
* **Constraint:** If refactoring requires changing **>20 lines of logic**, do NOT fix it automatically. Instead, add a `// TODO: Refactor complexity` comment and log it in the Audit Report.

**4. Type Hygiene**
* **No Implicit Any:** (TS) Explicitly type parameters.
* **Strictness:** Mark non-mutated arrays/objects as `readonly`.

**5. Observability**
* **Context:** Ensure logs contain metadata (e.g., `{ userId }`), not just static strings.

---

### 🏗️ B. If Infrastructure / Config (.dockerfile, .tf, .yaml, .json, .sh)
Apply the **Infra-Hardening Standards**:

**1. Version Pinning:**
* (Docker/GitHub Actions): Flag `latest` tags. Suggest pinning to a specific SHA or version.
* (Package.json): Flag `*` or `^` versions for critical dependencies if strict mode is preferred.

**2. Shell Script Safety (.sh):**
* Ensure `set -euo pipefail` is present at the top.
* Verify all variables are quoted (e.g., `"$VAR"`).

**3. Secrets in Config:**
* Scan `.json` or `.yaml` files for hardcoded secrets.

---

## 📝 Phase 3: Execution & Artifacts

**Step A: Safety First**
* Run `git checkout -b chore/audit-[random-hash]` to isolate these changes.

**Step B: The Audit Report (Artifact)**
* Generate a Markdown file `AUDIT_LOG.md` in the root.
* List the 10 files and a summary of what you "hardened" in each.

**Step C: Apply Changes**
* Apply the hardening edits to the files directly in the new branch.
* Run the project's linter (e.g., `npm run lint`) to ensure no regressions were introduced.

**Step D: Final Report**
* Present the `AUDIT_LOG.md` and ask: "Changes applied to branch `chore/audit-[hash]`. Ready to review?"