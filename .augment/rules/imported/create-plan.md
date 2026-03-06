---
type: "manual"
---

## Context
The user wants to start a complex task. You must generate a strategic document that serves as the "Source of Truth" for the project lifecycle.

**Input:** Any text provided after the slash command (e.g., `/create-plan to fix bug #123`) is injected into this prompt via the `$ARGUMENTS` placeholder. OpenCode will automatically replace `$ARGUMENTS` with the user's input.

## Steps

1.  **🔍 Deep Research (The "Context" Phase)**
    * **Analyze Input:** Thoroughly read the following input provided by the user:
      > $ARGUMENTS
    * **Github Issues:** If the input above or your research references a GitHub issue (e.g., "#330", "issue 330"), you **MUST** use the `gh` CLI to fetch its content:
        ```bash
        gh issue view <issue_number>
        ```
    * **Primary Source:** Read `architecture.md` FIRST to understand the global system design.
    * **Verification:** Scan the actual codebase to verify the `architecture.md` is up to date (and flag if it isn't).
    * **Pattern Matching:** Identify specific conventions for:
        * Directory structure (match against `architecture.md` descriptions)
        * Testing patterns
        * State management
    * *Constraint Check:* Does the user's goal conflict with the defined rules in `architecture.md`?

2.  **Analyze & Strategize**
    * Deeply analyze the user's request.
    * Identify the core objective, constraints, and necessary technologies.
    * Formulate a high-level strategy (the "Why" and "How").

3.  **Develop Implementation Plan**
    * Break the strategy down into specific Phases (e.g., Phase 1: Setup, Phase 2: Core Logic, Phase 3: UI/Integration).
    * For each Phase, list specific tasks.
    * Make sure to follow all rules, including the test coverage rule

4.  **Generate Checklist**
    * Create a granular, checkbox-style list (`- [ ]`) of every deliverable required.
    * Ensure the checklist covers code, tests, and documentation.
    * Ensure the work is planned to be done in a feature branch as the first step

5.  **Action: Write to File**
    * Create (or overwrite) a file named `PLAN.md` in the root directory.
    * **Format for PLAN.md:**
        ```markdown
        # Project Strategy: [Project Name]

        ## 1. High-Level Strategy
        [Summary]

        ## 2. Implementation Plan
        ### Phase 1: [Name]
        - Detail...
        ### Phase 2: [Name]
        - Detail...

        ## 3. Execution Checklist
        - [ ] Task 1
        - [ ] Task 2
        ...
        ```

6.  **Final Output**
    * Confirm to the user that `PLAN.md` has been created and summarize the first phase.
	
# rules

* Ensure these plans meet all of our rules
* Documentation should be considered when writing plans
* Ensure we fully implment features and harden/polish as we go to ensure a solid foundation and give users an excellent experience.