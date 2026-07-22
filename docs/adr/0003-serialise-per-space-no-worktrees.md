# Serialise per space; no worktrees; linear history

One session runs against a space at a time. Parallelism comes from driving several spaces at once, not several tickets of one map. There are no per-ticket worktrees or branches, and history stays linear.

The wayfinder markdown adapter forbids concurrent sessions outright — two collide on ticket numbers and on `map.md`, and git merges the collision silently. The obvious fix, inherited from iudex, is a worktree and branch per ticket. We rejected it: worktrees and their conflicts are a standing tax, and a linear history is worth more than intra-map parallelism. Because a space is a git repository and owns exactly one working tree, the **space** — not the map — is the unit of serialisation.

## Consequences

- Two maps inside one repository still cannot be driven at once.
- With nothing racing, an agent may write to `.plan/` directly; the chartr need not mediate map writes.
- The human gate cannot be a *merge* gate — there is no branch to merge. See ADR 0004.
