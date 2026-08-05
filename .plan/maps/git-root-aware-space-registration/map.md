# Git-root-aware Space registration

## Destination

A settled implementation contract for Git-root-aware Space registration. When an
operator registers a Space from inside a Git repository, chartr keeps the selected
directory as the Space path, finds the Git root, uses that root for every Git action,
and does not run `git init` in the subdirectory. If no Git repository exists, chartr
initializes Git in the Space path.

## Notes

This is a planning map. Resolve its tickets with the `grilling` and
`domain-modeling` skills. Read the `tracker-convention` skill before changing map
files.

Use the terms **Space path**, **Git root**, and **Git action** from `CONTEXT.md`.

The standing boundaries agreed during charting are:

- The selected directory remains the Space path. The Git root stays internal.
- Repository setup, branch display, dirty status, and claim or release commits use
  the Git root. Maps and other Space files use the Space path.
- Spaces that share a Git root remain separate Spaces and share the Git root's Git
  state.
- Existing nested `.git` directories are not deleted or migrated by this effort.

## Decisions so far

<!-- one line per resolved ticket: gist + link. -->

- [01 — Git root discovery and fallback errors](tickets/01-git-root-discovery-and-fallback-errors.md) — Git-native discovery selects the nearest usable repository root; only an explicit no-repository result allows initialization, and other Git failures do not initialize or update the registry.

- **01 — Git discovers the innermost worktree root from the Space path; only a confirmed no-repository result allows `git init` in the Space path. Linked worktrees and submodules stay valid, Git failures do not fall back, and the response reports the Space path plus `gitInited` without exposing the Git root.** [ticket](tickets/01-git-root-discovery-and-fallback-errors.md)

## Not yet specified

- **Git actions added later** — future Git actions may need the same root policy, but
  no sharper question exists until such an action enters the codebase.

## Out of scope

- **Legacy nested repository cleanup** — deleting or migrating `.git` directories
  that older chartr versions created inside a Space path.
- **Showing the Git root** — adding the internal Git root to the registry, API, or
  UI as a user-visible field.
- **Implementation work** — this planning map produces the contract for a later
  implementation map.
