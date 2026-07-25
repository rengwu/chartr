# Serialise per space; no worktrees; linear history

One session runs against a space at a time. Parallelism comes from driving several spaces at once, not several tickets of one map. There are no per-ticket worktrees or branches, and history stays linear.

The wayfinder markdown adapter forbids concurrent sessions outright — two collide on ticket numbers and on `map.md`, and git merges the collision silently. The obvious fix, inherited from iudex, is a worktree and branch per ticket. We rejected it: worktrees and their conflicts are a standing tax, and a linear history is worth more than intra-map parallelism. Because a space is a git repository and owns exactly one working tree, the **space** — not the map — is the unit of serialisation.

## Consequences

- Two maps inside one repository still cannot be driven at once.
- With nothing racing, an agent may write to `.plan/` directly; chartr need not mediate map writes.
- The human gate cannot be a *merge* gate — there is no branch to merge. See ADR 0004.

## Amendment: serialisation is the default, not a hard refusal

Spawning, resuming, or respawning into a space that already has a live session no longer refuses outright. It warns — naming the cost, that both agents will share one working tree with no branch or worktree between them — and the operator can confirm it.

The rejection of worktrees stands, and so does the reason: two agents in one tree *can* clobber each other's uncommitted edits, and git will not announce it. What changed is who decides. Whether a given pair collides depends on which files the two tickets touch, and chartr cannot know that — only the operator can. Refusing categorically made chartr the arbiter of a question it has no information about, and the cost fell on the safe cases: two tickets in unrelated corners of a repository were blocked for a hazard that was never going to happen.

So this joins the other gates that trust the operator at the point of decision rather than pre-empting them (the spawn preview's choice of role, ADR 0008's un-rolled-back claim). The warning is the design; the override is the escape hatch, and it is deliberately per-spawn — there is no "don't ask again", because the answer is a property of the two tickets, not of the space.

- Concurrency is opt-in per spawn, never remembered and never inferred.
- The refusal is machine-readable (`code: "live_session_exists"`) so the surface can tell the one overridable conflict from the refusals of fact — a held ticket, a missing agent — which no confirmation can turn into a spawn.
- A forced session is an ordinary session. Nothing downstream of the gate knows or behaves differently; the claim, payload, tab, and death halt are unchanged.
- Release is ungated, as it always was: it clears a claim and seats nothing.
