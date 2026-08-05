# Git-root-aware Space registration — implementation

## Destination

Implement the contract in [Git-root-aware Space registration](../git-root-aware-space-registration/spec.md).
An operator can register a Space below a Git root, use the Git root for every
current Git action, and keep the selected directory as the Space path.

## Notes

This is an implementation map. Its tickets are tracer-bullet slices. Each ticket
must leave a user-visible path working and must use the existing chartr process
test seam described in the spec.

Use the terms Space path, Git root, and Git action from `CONTEXT.md`. Keep the Git
root internal. Do not add a public Git-root field or change Space file discovery.

The local tracker has no separate label field. The `ready-for-agent` marker is
recorded in each ticket's metadata comment. Status remains derived from the
ticket file.

## Decisions so far

<!-- one line per resolved implementation ticket: gist + link. Empty until work lands. -->

- **01 — subdirectory registration.** Registration and the branch and dirty Git actions use one internal Git-root resolver. The selected Space path remains the registry, response, map, and skill path; shared-root Spaces keep separate identities and show shared Git state. [01](tickets/01-register-a-subdirectory-space-with-root-wide-git-state.md)
- **02 — child Space claim and release commits.** Claim, ticket release, and dead-session release resolve the Git root, calculate root-relative ticket paths, and keep path-limited commits; process tests cover nested paths and unrelated root-file isolation. [02](tickets/02-commit-claims-and-releases-from-a-subdirectory-space.md)
- **03 — registration edge cases and failure results.** Path validation runs before Git; linked worktrees, nested repositories, submodules, plain directories, and failure classes keep the settled response and error contract. [03](tickets/03-handle-git-root-edge-cases-and-registration-failures.md)

## Not yet specified

<!-- no additional implementation fog is known beyond the tickets below -->

## Out of scope

- The planning decisions and product boundaries in
  [Git-root-aware Space registration](../git-root-aware-space-registration/map.md).
- Legacy nested repository cleanup, public Git-root display, and new Git actions;
  these are excluded by the source spec.
