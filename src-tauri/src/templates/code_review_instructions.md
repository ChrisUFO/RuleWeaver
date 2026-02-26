# Basic Code Reviewer

Reviews a source file for common anti-patterns and code quality issues.

Run this skill on any source file to get a structured report of potential problems, including:
- Unused variables and imports
- Long functions that should be decomposed
- Missing error handling
- Hardcoded values that should be constants
- Inconsistent naming conventions

**Input:** `target_file` — path to the file to review (relative to the project root).
