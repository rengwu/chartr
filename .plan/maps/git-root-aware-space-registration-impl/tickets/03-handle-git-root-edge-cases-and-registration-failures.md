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

## Answer

Completed the registration edge cases and failure results.

- Registration now validates the Space path before it runs Git. Missing paths,
  inaccessible directories, and regular files return client errors. They do not
  run Git and do not add a registry entry.
- Registration uses the shared Git-root resolver. It accepts linked worktrees,
  nested repositories, and submodules without running `git init` or changing
  the outer repository. A plain directory still receives `.git` only in the
  selected Space path and reports `gitInited: true`.
- Missing Git, broken Git metadata, bare repositories, and invalid worktrees
  return server errors. They do not run `git init`, change existing `.git`
  entries, or add a registry entry.
- The success response is unchanged. It reports the selected absolute Space
  path and `gitInited`. It does not expose the Git root. Registration persistence
  errors are server errors, and a successful `git init` is not removed when a
  later registry save fails.
- Added process-boundary coverage for linked worktrees, nested repositories,
  submodules, invalid paths, Git failures, error JSON, and registry save
  failures.

Validation:

- `go vet ./...` passed.
- `go test ./... -run '^$' -count=1` passed and compiled all packages.
- `go test ./internal/registry -count=1` passed.
- The focused registration process tests passed:
  `go test ./internal/server -run '^TestRegister(SubdirectoryUsesRootGitStateAndSpaceFiles|LinkedWorktreeUsesLinkedRoot|NestedRepositoryAndSubmoduleUseInnermostRoots|FailuresDoNotInitOrPersist|RegistryFailureIsServerErrorAndKeepsInit|InitialisesNonRepoAnnounced)$' -count=1`.
- A broader `^TestRegister` run also selects an existing ticket 02 spawn test.
  That test still fails in ticket 02 claim-path code. It is outside this ticket.

Claim and release Git actions remain in ticket 02. Public Git-root display and
legacy nested repository cleanup remain out of scope.
