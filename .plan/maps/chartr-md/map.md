# CHARTR.md in the space

## Destination

An agent chartr did not launch can learn what chartr is by reading one file at the
root of the repository it is already sitting in. chartr generates `CHARTR.md` into
every registered space, carrying the same text a free session is told, and keeps it
out of git through `.git/info/exclude` rather than the space's tracked `.gitignore`.
Done when a `/new` agent in a registered space finds `CHARTR.md` at the root, the
file names chartr's conventions and the operator's registered sources, registering a
source rewrites it in every registered space, and `git status` in that space is
unchanged.

## Notes

**The mechanism already exists; this moves it.** chartr composes a free payload
(`prompt.ComposeFree`, `internal/prompt/compose.go:140`) and writes it to
`<space>/.chartr/run/<sid>/payload.md` under a `.gitignore` of `*`, then types
`adapter.Opener`'s read-this-file line into the launched TUI
(`internal/server/spawn.go:387`). That path stays exactly as it is. What this map
adds is a **second, standing copy at a well-known root path**, for the agents the
opener never reaches: one spawned with `/new`, one the operator started in their own
terminal, or one that compacted away its brief and can re-read it.

**Discovery is hopeful, not guaranteed.** Nothing auto-reads `CHARTR.md` — CLAUDE.md
is Claude Code's convention and AGENTS.md is the cross-harness one. This file works
on an agent's curiosity, and failing that on the operator typing *"read CHARTR.md"*.
That is the accepted ceiling. chartr does not edit CLAUDE.md or AGENTS.md; they are
the operator's files, not chartr's.

**Per-machine, so it may carry everything.** Because the file is excluded locally
and never committed, it can hold the sources block with its absolute
`<config>/sources/…` paths and `preferences.md` verbatim — the same bytes the free
payload carries. Nothing here reaches a teammate through `git pull`, so the standing
rule that execution config is never committed is not in tension with it.

**The ignore goes in `.git/info/exclude`, not `.gitignore`.** It is git's local,
uncommitted ignore file, which is exactly what a per-machine generated file needs:
no phantom diff in the operator's tracked `.gitignore`, no clone inheriting an
ignore rule for a file it will never have, and chartr never writes a file the
repository owns.

**Two reconciliation triggers, and only two: startup and a sources mutation.** Not
`rebuild()` — that fires on terminal churn and space registration as well, and this
is deliberately narrower. A `preferences.md` edit therefore does not reach
`CHARTR.md` until the next restart or the next sources change; that staleness window
is accepted for now.

**Per-ticket checks, before every commit:** `go vet ./...` and `go test ./...`; in
`web/`, the `check` and `build` scripts plus `vitest`. No UI is in scope, so
`docs/design-system.md` does not bind here.

**Develop against a throwaway config root and a throwaway space.**
`XDG_CONFIG_HOME=$(mktemp -d)` — `server.ConfigRoot` honours it — and register a
scratch `git init` directory rather than pointing a test run at this repository,
which is the one whose `git status` the destination clause is about.

## Decisions so far

<!-- one line per resolved ticket: gist + link. Empty until the first ticket ships. -->

- **The standing `CHARTR.md`.** The free brief minus its launch clause, at every
  registered non-scratch space's root, ignored through `.git/info/exclude`,
  reconciled at startup, on each sources mutation, and — by amendment — on space
  registration — [01](tickets/01-the-standing-chartr-md.md).

## Not yet specified

- **How a `preferences.md` edit propagates.** chartr never sees the write — the
  operator edits it in their own editor — so nothing short of a hash check at a
  read point or a filewatcher catches it. Out of scope here by decision; the
  restart-or-sources-change window stands until it is charted.
- **Whether the standing file should also point at the live map or frontier.** The
  free payload deliberately carries no live fact about the space, and a standing
  file has a strictly worse staleness profile than a per-spawn one, so the answer
  is probably no — but it has not been asked.

## Out of scope

- **Editing CLAUDE.md, AGENTS.md, or the space's tracked `.gitignore`.** Files the
  repository or the operator owns. Named here so no session reaches for them as a
  way to improve discovery.
- **Committing anything.** No variant of `CHARTR.md` is tracked, and no
  teammate-facing copy of the conventions is in scope on this map. That is a
  different document with a different content rule and it needs its own charting.
- **Removing or changing the per-session payload path.**
  `.chartr/run/<sid>/payload.md`, its `*` ignore and `adapter.Opener` are untouched.
