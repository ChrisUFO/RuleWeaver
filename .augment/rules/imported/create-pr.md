---
type: "manual"
---

# 1. Create Changeset
Use @add-changeset.md to create a customer focused changeset if not done already.

# 2. Commit/push changeset

# 3. Stage changes
git add .

# 4. Commit with conventional commit message
git commit -m "type: description"

# 5. Push to feature branch 
git push

# 6. Create a PR to merge to main
When creting github PRs, link to the issue that it addresses (if there is one).

[!TIP] To avoid formatting issues with multi-line descriptions, always write the PR body to a temporary pr_body.md file and use gh pr create --body-file pr_body.md instead of passing the body as a string." 

Delete the temporary file after creating the PR. (rm not)

# 7. Wait for PR to be reviewed