---
type: "manual"
---

# Workflow: Operation "Investor Grade" (Cleanup Swarm)
# Description: Autonomous refactoring of technical debt, database health, and infrastructure stability.
# Strategy: Execute strictly in order. Do not ask for confirmation unless a destructive database action is required.

---

## Phase 1: The Database Architect (Schema & Performance)
**Goal:** Optimize query performance and clean up schema debt.

1.  **Audit Schema:**
    - Scan `schema.prisma` (or equivalent `models` file).
    - Identify tables with multiple boolean flags (e.g., `is_done`, `is_finished`) and refactor them into a single Enum (e.g., `STATUS`).
    - **Action:** Generate the migration file immediately.
2.  **Index Check:**
    - Analyze all `FindMany` or `Where` clauses in the repository.
    - specific-check: If a foreign key is used in a `Where` clause but lacks an index in the schema, add the index.
    - **Action:** Add `@index` to the schema and generate the migration.
3.  **Verification:**
    - Run `pg_stat_statements` (or mock check) to ensure no full table scans remain on core tables.

## Phase 2: The Decoupler (Async & Docker)
**Goal:** Remove blocking operations from the main thread and standardizing dev environments.

1.  **Background Jobs:**
    - Scan controllers for file uploads, CSV processing, or heavy loops (>200ms).
    - **Action:** Move logic to a background worker file (using BullMQ, Celery, or language equivalent).
    - **Action:** Replace the original controller logic to return `202 Accepted` immediately.
2.  **Docker Standardization:**
    - Check for `docker-compose.yml`. If missing or incomplete, generate one that includes:
        - The Web App
        - The Database (seeded)
        - Redis (for queues)
    - **Constraint:** Ensure `npm install` (or equivalent) runs inside the container, not on the host.

## Phase 3: The Accountant (Cost & Observability)
**Goal:** Map money to code.

1.  **Cost Logging:**
    - Identify all external API calls (OpenAI, Stripe, Twilio).
    - **Action:** Create a wrapper function `track_cost(provider, input_size, output_size)` and wrap every external call with it.
    - **Action:** Ensure logs output a structured JSON object: `{"type": "cost", "provider": "openai", "cents": 0.04, "user_id": "..."}`.

## Phase 4: The Safety Net (Resilience)
**Goal:** Prevent cascading failures.

1.  **Circuit Breakers:**
    - Identify all `axios`, `fetch`, or `request` calls to external services.
    - **Action:** Wrap them in a standard Retry mechanism (3 retries, exponential backoff).
    - **Action:** Add a `catch` block that returns a "Degraded UI" state (e.g., empty list or toast notification) rather than crashing the page.

---

## Final Output Requirements
Once all phases are complete, generate a single artifact: `TECHNICAL_DEBT_REPORT.md` containing:
1. List of Indexes added.
2. List of Endpoints converted to Async.
3. The new `docker-compose` setup instructions.
4. A diff summary of the schema changes.