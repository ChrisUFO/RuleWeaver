---
type: "manual"
---

# 0. Check for uncommitted changes

# 1. Commit/push uncomitted changes

# 2. Merge (Squash) 
USE Github MCP or use gh cli as a fallback (gh pr merge --squash)

# 3. Switch to target and sync
git checkout [Target]
git pull origin [Target]

# 4. Clean up feature branch
git branch -d feature-branch-name
git push origin --delete feature-branch-name

If not target branch is specified, you can assume it's "main"