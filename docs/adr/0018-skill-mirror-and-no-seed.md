# Sessions read a repo-local skill mirror; chartr ships no seed

Supersedes two parts of [0017](0017-skills-come-from-registered-sources.md): the
seeded default source it kept for offline first-run, and its consequence that
`<space>/.chartr/skills/` "goes inert". Both are named in 0017's banner. The core
of 0017 — skills come from an ordered list of registered sources, resolved by
chartr, bound to roles by source-qualified references — is untouched and now
holds without exception.

chartr ships no skills at all, and every session reads the skills it is told
about from a copy inside its own working tree.

## Ship none, for real

0017 kept one exception to "chartr ships none": a `chartr-skills` source seeded
from the binary, sitting last, so a first run could spawn offline. That exception
is deleted — the vendored skills, the embed, `make vendor-skills`, the synthetic
default row, and the four role bindings that were seeded pointing into it.

- **chartr ships no skills inside the binary.** grill, implement, prototype,
  research, wayfinder and the rest live in a normal repo — `rengwu/chartr-skills`
  — the operator registers like any other. chartr's skills are no longer
  privileged over anyone else's: nothing is embedded, nothing is materialized from
  the binary. That is the plainest possible reading of "hackable".
- **A fresh install pre-registers that repo as an ordinary `git` source.** The one
  run that first writes `sources.toml` clones `rengwu/chartr-skills` into a row the
  operator can remove, refresh or reorder like any other — not a synthetic seed
  privileged in resolution, just a URL chartr fills in for them so the first run
  has something to spawn from. It sits alongside whatever the layer-model migration
  carried forward. Removing the row is final: the pre-registration fires only on
  that same once-per-machine signal, so it is never re-cloned.

**Amends this ADR's original "a fresh install registers nothing."** That first cut
made the source list start genuinely empty and left every role unbound until the
operator registered a source by hand — the price it named for owning nothing. In
practice that price was paid on every single first run, for a repo every operator
wanted anyway, so we now pre-register it rather than pre-fill a form and wait. The
principle the cut was defending is intact: **nothing is embedded, and the row is
the operator's** — a fresh install with no network or no `git` still comes up (the
clone is best-effort), just with an empty list to fill in from Settings.

## The mirror

A role skill's body is still concatenated into the payload chartr writes (0017,
reaffirming [0002](0002-agent-agnostic-adapters.md) and
[0005](0005-assembled-context-no-agent-memory.md) — unchanged). But a role skill
routinely sends the agent on to *other* skills — a `grill-me` whose whole body is
"run `/grilling`" — and those were reachable only through an absolute path outside
the repository, which an agent sandboxed to its own working tree cannot read. The
sources block named skills an agent then could not open.

So chartr materializes a **mirror**: every enabled source's discovered skill
directories, copied into `<space>/.chartr/skills/<source>/<skill>/`, and the
payload's sources block points at those paths instead of the external ones.

- **Reconciled in place, before every session.** The `ensureSkillsCurrent`
  barrier brings the mirror level with the enabled sources before a spawn or a
  free session composes its payload, so a session always starts against current
  skills. In place, not replaced wholesale: a live session reads these files on
  demand, and the directory it is reading is never renamed out from under it.
- **Regular files only; symlinks skipped.** The minimal, safe handling: a symlink
  inside a skill can point outside the space and reopen the very sandbox hole the
  mirror closes. Source mode bits are preserved, so a bundled script stays
  executable.
- **Gitignored, per-machine, never committed.** A `*` marker at the mirror root
  keeps the whole tree out of git — the same device `.chartr/run` uses, the same
  footing as `CHARTR.md`. Nothing here reaches a teammate through a `git pull`, so
  0017's rule that execution content is never committed still holds.
- **Repo-relative paths.** The block prints `.chartr/skills/<source>`, not a path
  on the operator's disk. Relative on purpose: it is what a sandboxed agent can
  read, and it is identical in every space, which is what keeps the standing
  `CHARTR.md` composable once for all of them.

## Consequences

- **Reverses 0017's "the committed workspace layer goes inert".** chartr writes
  into `<space>/.chartr/skills/` again — the exact path 0017 declared dead — but
  gitignored and per-machine, which is the footing 0017's own "chartr stops
  writing into the operator's repository" already carved out for `CHARTR.md` and
  the per-session payload. "This project's skills" is still not expressible: every
  enabled source is mirrored into every space (global trust), unchanged from 0017.
- **The mirror owns `.chartr/skills/` outright.** An operator's leftover content
  under that path — a hand-made file, a relic of the retired workspace layer — is
  pruned by the reconcile, because the mirror removes what no enabled source
  accounts for. The path is chartr's now; "I want my own skills here" is answered
  by registering a `dir` source, not by writing into the mirror.
- **Freshness beyond a spawn is a separate, still-open concern.** The barrier
  keeps the mirror current only for sessions chartr launches. An ad-hoc agent that
  reads `CHARTR.md` without chartr spawning it needs the mirror kept fresh some
  other way; the intended mechanism is a focus-triggered background reconcile of
  the focused space's local sources, which is not yet built.
- **No change to reproducibility beyond 0017.** The mirror is a copy of what a
  source already held; the claim trailer still records `Skill: name=source` with
  `@<commit>` where the source is pinned, and `Payload-SHA256` still fixes the
  exact bytes a session was told.

## Considered options

- **Point the payload at the external source paths and require agents to read
  outside the repo** — rejected: it assumes every harness can, and every sandbox
  will allow, reads outside the working tree, which is exactly the assumption that
  made cross-skill references unreachable in the first place.
- **Symlink the sources into the repo instead of copying** — rejected: a sandbox
  authorizes the resolved target, not the path inside the repo, so a symlink to an
  external directory is the same escape it appears to prevent; and it would expose
  mutable external files to a session as if they were the repo's.
- **Keep the seed for offline first-run** — rejected: it is the one exception that
  kept "chartr ships none" from being true, and the whole of this change is making
  it true. The cost — no spawn until a source is registered — is accepted.
