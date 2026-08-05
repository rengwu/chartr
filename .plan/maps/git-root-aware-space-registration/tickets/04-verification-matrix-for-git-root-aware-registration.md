---
type: grilling
blocked_by: [01, 02, 03]
claimed_by: s4529e7818655
claimed_at: 2026-08-05T19:08:12Z
---

# Verification matrix for Git-root-aware registration

## Question

Turn the decisions in:

- [01 — Git root discovery and fallback errors](01-git-root-discovery-and-fallback-errors.md);
- [02 — Space path and shared Git state](02-space-path-and-shared-git-state.md); and
- [03 — Claim and release commits from a subdirectory](03-claim-and-release-commits-from-a-subdirectory.md)

into a complete acceptance matrix. Include registration from a repository root,
from a subdirectory, from a non-repository directory, and from the supported
special repository forms. Include branch and dirty state, two Spaces sharing one
Git root, claim and release commits, the `gitInited` response, and the unchanged
non-repository behavior.

## Done when

The answer lists the required test scenarios and their observable results. It
identifies the smallest test seams that prove the contract without testing Git's
own behavior. It also states which old tests must remain unchanged.

## Answer

### Decision

Use one acceptance matrix at the HTTP boundary, with two narrow test seams:

1. Use real temporary Git fixtures for successful registration, branch and
   dirty state, shared roots, linked worktrees, nested repositories, submodules,
   and claim or release commits.
2. Use a test Git command runner, or a Git command shim, for missing commands,
   controlled Git failures, command working directories, arguments, and the
   no-repository versus other-error decision. Use a registry-save failure seam
   for the persistence case.

The tests must check chartr's inputs, outputs, and side effects. They must not
test Git's internal implementation. The public registration route remains the
main seam. It proves the HTTP status, JSON body, registry state, Space model,
and files together.

The three blocker answers are load-bearing and agree. The no-repository result
must be different from every other Git error. The selected directory remains
the Space path. The derived Git root is used only for Git actions. A current
implementation gap is visible: registration still checks `Space path/.git`,
`deriveSpace` still passes the Space path to branch and dirty checks, and claim
and release still pass the Space path to their commit writers. The matrix must
fail for these cases until the later implementation fixes those paths. This
ticket does not change that code.

### Registration matrix

| Case | Setup and action | Required observable result |
| --- | --- | --- |
| Repository root | Register the root of a normal worktree. | HTTP 200. The response has only `id`, absolute Space `path`, and `gitInited: false`. No `git init` runs. The registry and model use the root as the Space path. |
| Repository subdirectory | Register `/repo/one` while `/repo` is the worktree root. | HTTP 200 and `gitInited: false`. The response path and registry path are `/repo/one`. The Space name and ID come from `/repo/one`. No `.git` is created in `/repo/one`. A Git action uses `/repo` as its Git root. |
| Linked worktree root | Register a linked worktree root whose `.git` is a file. | HTTP 200 and `gitInited: false`. The `.git` file stays valid. The Git root is the linked worktree root, not the common Git directory. Branch and dirty state work. |
| Linked worktree subdirectory | Register a directory below that linked worktree. | HTTP 200 and `gitInited: false`. No nested `.git` is created. A Git action uses the linked worktree root. |
| Nested repository | Register the inner repository in an outer repository. | HTTP 200 and `gitInited: false`. The inner repository is the Git root. The outer repository is not initialized or changed by registration. |
| Submodule | Register the submodule worktree, and then a directory below it. | HTTP 200 and `gitInited: false`. Git uses the submodule worktree as the root. The superproject is not used as the Git root and no new `.git` is created in the selected path. |
| Directory outside a repository | Register a plain temporary directory with no repository in its ancestors. | HTTP 200 and `gitInited: true`. `git init` runs in the selected Space path. `.git` is created there. The response path is the selected absolute path. |
| Explicit no-repository result | At the Git command seam, return the typed no-repository result for a valid directory. | `git init` runs once with the Space path as its working directory. Registration succeeds with `gitInited: true`. |
| Missing Git command | Make the Git command unavailable. Register a valid directory. | HTTP 500. The error JSON has no success fields and identifies Git discovery. `git init` does not run. No registry entry is created or changed. |
| Broken or unusable `.git` | Use a broken `.git` entry or an invalid worktree. | HTTP 500. The error preserves useful Git output. `git init` does not run. No registry entry is created or changed. |
| Bare repository | Select a bare repository as the Space path. | HTTP 500. Chartr does not treat the bare repository as a worktree and does not run `git init`. No registration is saved. |
| Init failure | Return the explicit no-repository result, then fail `git init`. | HTTP 500 with the init operation and Git output. No registry entry is created or changed. |
| Registry save failure after init | Let `git init` succeed, then fail registry persistence. | HTTP 500. The new `.git` remains in the Space path. Chartr does not delete it. No successful registration response is returned. |
| Invalid path | Register a missing path, a regular file, and an inaccessible path. | HTTP 400. Validation stops before any Git command. No `.git` entry and no new or updated registry entry exists. |
| Response privacy | Inspect every successful response, error response, registry entry, and model JSON. | No response, registry field, server model field, client model field, or UI value exposes `gitRoot`. |

For all success cases, assert the existing JSON shape. For all failure cases,
assert an error JSON body with no `id`, `path`, or `gitInited`. The HTTP status
must be 400 only for invalid input. Git discovery, init, and persistence errors
must be 500.

### Branch, dirty state, and shared-root matrix

| Case | Setup and action | Required observable result |
| --- | --- | --- |
| Branch from the Git root | Register `/repo/one` with `/repo` on a named branch. Rebuild the model. | The Space path remains `/repo/one`. The branch field shows the branch for `/repo`. A linked worktree shows its own branch. |
| Dirty state outside the Space | Keep `/repo/one` clean. Create or modify `/repo/two/outside.txt`. Rebuild. | The Space for `/repo/one` is dirty because the full Git root is dirty. A file below another Space can cause this result. |
| Two Spaces, one root | Register `/repo/one` and `/repo/two`. Give each directory different maps, skills, and files. | Two registry entries and two Spaces remain. IDs, names, maps, skills, working directories, and file actions stay separate. IDs use the two Space paths, not the Git root. |
| Shared Git state | With the two Spaces registered, change the branch or dirty state of `/repo`. Rebuild. | Both Spaces show the same branch and dirty value for that Git root. A Git commit for one Space can change the state shown for the other. |
| No root leakage | Inspect the two registry entries, model snapshots, registration responses, and client payloads. | The internal Git root is absent. The existing `path`, `branch`, and `dirty` fields keep their meanings. |

### Claim and release matrix

Use a Space at `/repo/one` and a ticket at
`/repo/one/.plan/maps/demo/tickets/03-commit.md`. Use `/repo/two/work.go` as
an unrelated file. Keep that file staged before the lifecycle action when the
test checks `--only`.

| Case | Setup and action | Required observable result |
| --- | --- | --- |
| Claim from a subdirectory Space | Register `/repo/one` and spawn a supported test agent for the ticket. | The ticket is stamped. The session starts only after the claim commit succeeds. The claim commit names `one/.plan/maps/demo/tickets/03-commit.md`, uses the Git root as its working directory, and contains only that ticket. The unrelated staged file is not in the commit and remains staged or dirty. Existing claim trailers remain. |
| Release from a subdirectory Space | Start with an orphaned claim, then call the ticket release route. | HTTP 200. The response keeps the existing release fields. The ticket has no claim and is open/frontier. The release commit contains only the same root-relative ticket path, names that path in the message, and keeps the session and `Chartr-Write` values. |
| Claim root discovery failure | Make discovery return no repository or another Git failure during claim. | HTTP 500. Claim and init do not run. The ticket is not read or changed, no payload is written, no index path is staged, and no session starts. |
| Release root discovery failure | Make discovery fail during release. | HTTP 500. No init or fallback occurs. The ticket is not changed, no success is reported, no dead tab is discarded, and no model rebuild occurs. |
| Lifecycle path outside the root | Call the claim and release path-validation seam with the root itself, an outside path, and a path beginning with `..`. | The action rejects each path before ticket read or write, staging, or commit. The check uses a relative-path result, not a string prefix. |
| Claim Git failure | Fail `git add` or `git commit` after the ticket write. | HTTP 500. No payload or session starts. The visible ticket or index change already made before the failure is not silently reset. Git output is included in the error. |
| Release Git failure | Fail `git add` or `git commit` after the claim removal write. | HTTP 500. No success response, dead-tab discard, or rebuild occurs. Any visible ticket or index change left by the failed operation remains for the operator. |

For successful claim and release, inspect the commit file list and message. Do
not infer correctness only from the ticket model. The file-list check proves
that an unrelated file in a shared Git root was not added to the lifecycle
commit.

### Smallest test seams

- Use the existing process-boundary test rig and `POST /api/spaces` for
  registration. Add fixture builders for a normal root, a subdirectory, a
  linked worktree, a nested repository, a submodule, and a plain directory.
- Use the model snapshot after registration or rebuild for Space path, name,
  ID, map discovery, branch, and dirty state. This checks the public model and
  does not test Git internals.
- Use the existing claim/spawn and ticket-release HTTP routes for lifecycle
  tests. Inspect the ticket file, the session or tab state, and the root Git
  commit after each action.
- Use one narrow Git runner seam, or a test Git shim, only for command absence,
  controlled error classes, working directories, and exact arguments. It must
  record calls and return configured output. It must not reimplement Git root
  discovery.
- Use one registry persistence seam for the post-init save failure. Assert the
  accepted partial result instead of adding automatic cleanup.

### Tests that remain unchanged

Keep `TestRegisterInitialisesNonRepoAnnounced` unchanged. It is the regression
for the existing non-repository behavior: a plain directory gets `.git`, the
response says `gitInited: true`, the selected path and name remain visible, and
an existing repository is not initialized again.

Keep the existing root-based registration and lifecycle tests unchanged,
including `TestForgetNotDestroy`, `TestRegistryLossIsRebuildable`,
`TestReleaseTicketClearsAnOrphanedClaim`,
`TestReleaseTicketDropsThePinnedDeadTab`,
`TestReleaseTicketRefusedWhileSessionLive`, and
`TestReleaseTicketRefusedWhenNotClaimed`. Add subdirectory cases beside these
tests. Do not weaken their checks for registry persistence, unchanged
repositories, claim state, release state, or commit count.

### Rejected alternatives and trade-offs

Testing only `Registry.Register` would miss HTTP status, response privacy,
snapshot state, and registry side effects. Testing only real Git fixtures would
not reliably prove missing Git, error classification, or exact command paths.
Testing only helper functions would repeat Git behavior and could pass while
the public route uses the wrong root. These are useful small checks, but none
is a complete acceptance test.

The chosen matrix costs fixture setup and one controlled command seam. That
cost is accepted because it proves the operator-visible contract and the
shared-root safety rule. It does not make Git's rules part of chartr's tests.

Reopen this decision if a new repository form cannot use Git root discovery, if
bare repositories become supported, if Git state must be limited to one Space,
if a lifecycle action must commit more than one file, or if failed `git init`
or lifecycle commits must be rolled back automatically.
