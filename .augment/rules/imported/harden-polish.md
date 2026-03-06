---
type: "manual"
---

# Workflow: Phase Quality Review & Polish
# Trigger: /harden-polish [Phase Number]


## Context
The user wants to conduct a deep-dive review of a specific project phase provided in `$ARGUMENTS` (e.g., "Phase 5"). This is not just a checkbox exercise; it is a request for critical analysis regarding UI/UX excellence and code hardening.


## Steps


1.  **Retrieve Requirements**
    * Read `PLAN.md` and extract the specific deliverables and tasks listed under the requested Phase (e.g., "Phase 5").
    * Identify the core code files created or modified during this phase.

2.  **Completeness Audit**
    * verify that every specific requirement listed in `PLAN.md` for this phase has a corresponding implementation in the code.
    * Flag any missing functionality immediately.

3.  **UI/UX Polish Analysis (World-Class Standard)**
    * Review frontend components associated with this phase.
    * Critique the UI/UX with a high bar:
        * **Interactivity:** Are there loading states, hover effects, and transitions?
        * **Feedback:** Does the user know when actions succeed or fail?
        * **Aesthetics:** Is spacing consistent? Is the design accessible?
    * *Generate specific suggestions to elevate the UI to "World Class" status.*

4.  **Hardening & Resilience Analysis**
    * Review logic and backend handlers.
    * Look for:
        * **Edge Cases:** What happens with empty data or network failures?
        * **Error Handling:** Are errors caught and displayed gracefully?
        * **Security:** Are inputs validated?
    * *Generate specific suggestions to "harden" the code.*

5.  **Report & Update**
    * Output a structured review:
        * **Status:** [Complete / Incomplete]
        * **Missing Items:** [List if any]
        * **UI/UX Polish Suggestions:** [Actionable improvements]
        * **Hardening Suggestions:** [Actionable improvements]
    * Ask: "Would you like me to implement these polish/hardening items now?"