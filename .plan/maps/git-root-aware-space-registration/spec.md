<!-- triage: ready-for-agent -->

# Git-root-aware Space registration — spec

## Problem Statement

When an operator registers a Space from a subdirectory of a Git repository,
chartr checks only the selected directory for `.git`. It does not see the Git
root above that directory. It then runs `git init` in the subdirectory.

This creates a new nested repository inside the company's repository. Later Git
actions also use the subdirectory. Branch display, dirty status, and chartr claim
or release commits can therefore use the wrong repository.

This blocks operators who work in monorepos. Their Space path is often one or two
levels below the Git root.

## Solution

Keep the selected directory as the Space path. From that path, find the nearest
Git repository root by using Git's own repository discovery.

Use the Git root for all current Git actions:

- repository detection and setup;
- branch display;
- dirty status;
- claim commits; and
- release commits.

Use the Space path for Space files, maps, skills, and other non-Git actions.

If Git reports that the Space path is not inside a repository, initialize Git in
the Space path. Use that path as the Git root after initialization. If Git reports
another error, fail registration and do not run `git init`.

The Git root is internal. The Space remains identified and displayed by its
selected path. Two Spaces may use different paths inside one Git root. They remain
separate Spaces and share the Git root's branch and dirty state.

Claim and release commits remain limited to one ticket file. Compute the ticket
path relative to the Git root so unrelated changes in the shared repository are
not committed.

## User Stories

1. As an operator, I want to register a directory that is already a Git root, so that chartr uses the existing repository.
2. As an operator, I want to register a directory inside a Git repository, so that chartr does not create a nested repository.
3. As an operator, I want the selected directory to remain the Space path, so that chartr reads maps, skills, and files from the directory I selected.
4. As an operator, I want chartr to find the nearest Git root, so that nested repositories and submodules use the repository that Git assigns to the selected path.
5. As an operator, I want linked worktrees to work, so that a `.git` file does not cause false repository detection.
6. As an operator, I want registration to report whether chartr ran `git init`, so that I know when chartr created a repository.
7. As an operator, I want a non-repository directory to become a Git repository, so that I can register a new Space without preparing Git first.
8. As an operator, I want `git init` to run in the selected directory when no repository exists, so that the new repository belongs to the Space path.
9. As an operator, I want Git failures other than “no repository” to stop registration, so that chartr does not hide a broken Git installation or another operational error.
10. As an operator, I want the registration response to keep the selected path, so that the Space identity does not change when its Git root is above it.
11. As an operator, I want two subdirectories in one monorepo to remain separate Spaces, so that I can switch between them independently.
12. As an operator, I want the branch shown for a subdirectory Space to come from its Git root, so that the cockpit shows the real repository branch.
13. As an operator, I want the dirty state shown for a subdirectory Space to come from the full Git root, so that the state reflects the repository that Git actions change.
14. As an operator, I want file and map discovery to remain scoped to the Space path, so that one Space does not display another Space's maps or skills.
15. As an operator, I want a claim commit from a subdirectory Space to stage only its ticket file, so that chartr does not include unrelated monorepo changes.
16. As an operator, I want a release commit from a subdirectory Space to use the same ticket path rule, so that claim and release have consistent Git behavior.
17. As an operator, I want shared-root Spaces to keep their own Space names and identities, so that Git root sharing does not merge their registry entries.
18. As an operator, I want the Git root to remain an internal detail, so that the existing Space API and UI do not gain a second path that can confuse me.
19. As an operator, I want existing nested repositories to remain untouched, so that this change does not delete or rewrite data created by an older chartr version.
20. As a maintainer, I want one high-level test seam for registration and Git actions, so that the tests prove operator-visible behavior without coupling to private helper functions.

## Implementation Decisions

- The canonical domain terms are Space path, Git root, and Git action. The Space
  path is the selected directory. The Git root is the nearest repository root
  found from that path.
- Git-native root discovery is the source of truth. It must support normal
  worktrees, linked worktrees, and nested repositories as Git supports them.
- Registration must distinguish “no repository” from another Git error. Only the
  explicit no-repository result permits initialization. A missing Git command,
  permission failure, malformed or unusable Git metadata, bare repository, or
  invalid worktree is an error and does not initialize a new repository.
- A successful initialization uses the Space path as the Git root. The existing
  `gitInited` response value remains true only when chartr actually ran
  initialization. Registration of an existing repository returns false.
- Path validation happens before Git discovery. A missing path, inaccessible path,
  or regular file fails without a Git action.
- The existing registration response remains the public contract. Success keeps
  the selected absolute path and reports `gitInited`; invalid paths remain client
  errors, while Git, initialization, and registry persistence failures remain
  server errors. A successful `git init` is not removed automatically if registry
  persistence fails.
- The registry keeps the selected path as the Space path and keeps the existing
  Space identity rules. The Git root is derived internal state. It is not a new
  persisted registry field, API field, or UI field.
- All current Git actions receive the Git root as their repository context. This
  includes setup, branch display, dirty status, claim commits, and release
  commits.
- Non-Git file actions continue to use the Space path. Map discovery, skill
  discovery, ticket reads and writes, and other Space file operations must not
  widen to the Git root.
- Dirty status is repository-wide for the Git root. If two Spaces share a Git
  root, a change anywhere in that root can make both Spaces dirty.
- Branch display is repository-wide for the Git root. If two Spaces share a Git
  root, they show the same branch state.
- Claim and release commits keep their current single-ticket behavior. The ticket
  path is made relative to the Git root before staging and committing. The commit
  must remain path-limited and must not include unrelated changes.
- Existing nested `.git` directories are not deleted, migrated, or repaired by
  this feature. Legacy cleanup is a separate effort.
- Prefer one shared internal Git-root resolution seam for registration and all Git
  actions. The exact function or type shape is an implementation detail, but the
  behavior must not be reimplemented differently by each caller.

## Testing Decisions

- Tests must observe external behavior. They must not assert private fields,
  helper calls, command construction, or the number of root-resolution calls.
- The primary seam is the real chartr process boundary through the existing test
  harness. Tests register temporary Spaces through the operator-facing HTTP API.
- The test harness must assert the registration response, the Space snapshot, the
  files in the temporary Space, and Git history or status as appropriate.
- Use real temporary Git repositories. Create a repository root and child Space
  directories to prove that registration from a subdirectory does not create a
  child `.git` directory.
- Cover registration from a repository root, from a child directory, from a
  normal non-repository directory, from a linked worktree, and from a nested
  repository or submodule where the platform test environment supports it.
- Cover a Git failure that is not “no repository”. The observable result must be a
  failed registration with no new `.git` directory. Include missing Git, unusable
  metadata, and bare repository cases where the test environment supports them.
- Cover two registered Spaces in one Git root. Assert that their paths and
  identities remain separate while their branch and dirty values reflect the
  shared Git root.
- Cover a claim from a subdirectory Space. Assert that the ticket file is
  committed and an unrelated changed file is not committed.
- Cover release from a subdirectory Space with the same path and isolation
  assertions.
- Keep the existing non-repository registration test as the prior-art pattern for
  the `gitInited` response and the created `.git` directory.
- Extend the existing Space snapshot tests for branch and dirty state, and extend
  the existing claim and release tests for Git history. Reuse the existing
  temporary Space and Git fixture helpers.
- The test set must verify the operator-visible contract. It must not test Git's
  own root discovery implementation beyond the cases needed to prove chartr uses
  the returned root correctly.

## Out of Scope

- Deleting, migrating, or identifying `.git` directories created by older chartr
  versions.
- Adding the Git root to the registry file, API response, Space model, or UI.
- Changing the Space identity, Space name, map discovery, skill discovery, or file
  path rules to use the Git root.
- Filtering dirty status to only the Space path. Dirty status remains full-root
  Git state.
- Adding Git actions that do not exist today. A later Git action must receive its
  own root policy.
- Creating Git worktrees, branches, or other isolation for Spaces that share a
  Git root.
- Changing Git behavior outside the commands chartr runs for registration, branch
  display, dirty status, claim, and release.

## Further Notes

This spec is based on the planning discussion in
[Git-root-aware Space registration](map.md). The answer in
[01 — Git root discovery and fallback errors](tickets/01-git-root-discovery-and-fallback-errors.md)
settles root discovery and fallback errors. The remaining decision tickets can
refine the shared-root and commit contracts before an implementation map is
created.

The local chartr tracker has no separate label field. The `ready-for-agent` label
is recorded in the metadata comment at the top of this spec.
