---
type: grilling
blocked_by: [01]
claimed_by: se451ad1e7690
claimed_at: 2026-08-05T18:59:50Z
---

# Space path and shared Git state

## Question

Define the boundary between the Space path and the Git root after
[01 — Git root discovery and fallback errors](01-git-root-discovery-and-fallback-errors.md)
settles root selection.

The selected directory must remain the Space path and the Git root must remain
internal. Decide how this boundary applies to the registry entry, Space name and
identity, map and skill discovery, branch display, dirty status, and multiple
Spaces that share one Git root. The current direction is that branch and dirty
state describe the full Git root, while file and map actions stay at the Space
path.

## Done when

The answer states which path each affected part of the Space uses. It defines the
behavior for two Spaces in one Git root. It states whether any model, API, or UI
field changes are needed.

## Answer

### Decision

Use a two-path boundary:

- The **Space path** is the selected, normalized absolute directory. It remains
  the Space's identity and the boundary for Space files.
- The **Git root** is an internal derived value. Registration discovers it by the
  rule in Ticket 01. Later Git actions use that root. The root is not stored in
  the registry and is not exposed to the operator.

The answer from Ticket 01 is a sound blocker. It selects the innermost worktree
root, including a linked worktree root, and does not confuse the common Git
directory with that root. This supports the boundary below.

### Path assignment

| Space part | Path | Contract |
| --- | --- | --- |
| Registry entry | Space path | Persist the selected absolute path. Do not persist the Git root. |
| Space identity | Space path | Keep the existing ID from the normalized absolute Space path. Two different Space paths have different IDs, even when they have the same name. |
| Space name | Space path | Derive the name from the selected directory. Do not derive it from the Git root. |
| Maps and map files | Space path | Discover `.plan/maps/` and read or write map files below the Space path. |
| Skills | Space path | Resolve the Space skill layer below `<Space path>/.chartr/skills`. Global and user skill layers do not change. |
| Other Space files and file actions | Space path | Use the selected directory. This includes the Space's working directory and file watchers. |
| Branch display | Git root | Read the current branch or detached HEAD of the worktree at the Git root. |
| Dirty status | Git root | Read the Git status of the full Git root. Changes outside the Space path can make the Space dirty. |
| Git actions | Git root | Use the root for repository setup, branch and dirty checks, and claim or release commits. Ticket 03 defines the commit path details. |

The internal root can be passed between these Git operations or cached in
memory. It must remain derived from the Space path under Ticket 01 and must not
become a registry, API, model, or UI field.

### Two Spaces in one Git root

For two selected directories such as `/repo/one` and `/repo/two`, with
`/repo` as their Git root:

- They remain two registry entries and two Spaces. Their IDs use their own
  Space paths. Their names use `one` and `two`. A name collision in other paths
  does not merge their IDs.
- Their map discovery, map files, skill layers, file actions, and working
  directories remain independent because each uses its own Space path.
- They show the same branch. They also show the same dirty value because both
  describe `/repo`'s Git state.
- A Git change or commit made for one Space can change the branch or dirty
  value shown for the other Space. A rebuild reads the current root state for
  both Spaces. This shared state is intentional.

Two linked worktrees that use one common Git administrative directory do not
share a Git root for this rule. Ticket 01 selects each linked worktree root.

No public field changes are needed. Keep the existing registry path, model
`Space.path`, `branch`, and `dirty` fields, and the existing registration
response with `id`, `path`, and `gitInited`. Do not add `gitRoot` to the registry
file, API response, server model, client model, or UI.

### Rejected alternatives

- **Use the Space path for every operation.** This keeps a local-looking
  boundary, but it relies on Git's implicit discovery. Direct `.git` readers
  fail for a repository subdirectory and a linked worktree. Later Git actions
  can also calculate paths from the wrong base. It does not give every Git
  action the root contract from Ticket 01.
- **Use the Git root for every operation.** This makes branch and dirty checks
  simple, but it makes root-level maps, files, and skills appear to belong to
  every Space. It breaks the selected directory as the Space path and can
  collapse the identity of Spaces that share a root.
- **Persist the Git root as a hidden registry field.** This avoids a later
  discovery call, but the value can become stale when a nested repository or
  worktree changes. It also adds migration and leakage risk for a value that
  the operator does not need to see. The Space path is the stable registry
  value; the Git root is derived state.
- **Show Git state only for files below each Space path.** This would isolate
  the badges, but branch state belongs to the worktree and the map direction
  requires root-wide Git state. It would also hide changes that can affect a
  claim or release commit.

### Revisit trigger

Reopen this decision if an accepted Git action cannot use the root selected by
Ticket 01, if an operator needs branch or dirty state limited to one Space path,
if shared-root state causes unsafe cross-Space work, or if root discovery cannot
remain reliable without a persisted public value. A need for the superproject
root instead of the innermost worktree root reopens Ticket 01 first.
