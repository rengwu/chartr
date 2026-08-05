---
type: task
blocked_by: [01]
claimed_by: s969a2483e0a1
claimed_at: 2026-08-05T19:29:21Z
---
<!-- triage: ready-for-agent -->

# Handle Git root edge cases and registration failures

## Question

Complete the registration contract for repository forms and failure results that
are not covered by the normal monorepo path.

Registration must use the nearest usable Git root for linked worktrees, nested
repositories, and submodules. A valid directory outside every repository must
initialize Git only in the Space path. A missing Git command, broken Git metadata,
bare repository, invalid worktree, inaccessible path, missing path, or regular
file must fail without creating a new repository or registry entry.

Keep the existing success response. Keep the selected Space path in the response,
report `gitInited` only when chartr ran `git init`, and use the specified client or
server error result for each failure class. Do not delete or migrate an existing
nested `.git` directory.

## Done when

- A linked worktree root and child use the linked worktree root without a new
  `git init`.
- A nested repository and a submodule use the innermost repository root without
  changing the outer repository.
- A plain directory receives `.git` only in the selected directory and reports
  `gitInited` as true.
- Missing Git, broken Git metadata, a bare repository, and an invalid worktree
  fail without initialization or a registry update.
- A missing path, inaccessible path, and regular file fail before any Git action.
- The response and error results match the settled registration contract.
- Existing nested `.git` directories remain untouched.
- Tests prove all supported cases through the existing chartr process boundary.
