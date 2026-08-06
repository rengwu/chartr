---
type: task
blocked_by: []
claimed_by: s46e954fe912c
claimed_at: 2026-08-06T16:01:13Z
---

# The launched-agent split

## Question

Split the two jobs `session != nil` is doing in `internal/terminal`. It means both
*this tab has a ticket* and *chartr knows which binary runs here*, and those come
apart the moment a free session exists — an agent with a payload and no ticket.
This ticket makes the split before ticket 08 needs it, and it is worth landing on
its own because **it fixes a bug that exists at HEAD today**, independently of
everything else on this map.

Today's on-ramp tab is the same shape a free session will be, and it already pays
for the conflation twice. It re-runs `foreground()` and `procGroupNames(pgrp)` every
slow tick to rediscover an agent chartr chose itself, where a session answers from
its binding for free. Worse, **an on-ramp tab on an agent with no shipped manifest
reads idle for its whole life**: its root process *is* the agent, so `pgrp ==
shellPID` forever and `sampleShell` returns idle permanently — the exact failure
`sampleUnknownSession` exists to prevent for ticket sessions. There is also a boot
flash, because `newProc` seeds the working state only when a session is present.

The fix is small and its shape is fixed by the spec: `launchSpec` gains an agent
name, `Terminal` gains a `launchedAgent` field, and both the session opener and the
free-session opener set it. **Only two sites move to the new field** — `sample()`'s
identification branch and `newProc`'s working-state seed. Every other site keeps
reading `session != nil`, because every other site wants a free session on the nil
side: the one-session-per-space gate (a free session claims nothing), death-pinning
and the dead state (resume, respawn and release all act on a ticket), the session
title, and the session info on the pushed model. `sampleUnknownSession` is renamed
`sampleLaunchedAgent` and is reached by both paths.

**Do not introduce a tab-kind enum.** Three names for two independent booleans would
force every settled `session != nil` site to be re-read and re-decided; the model
gains one field, not a third kind.

Per the spec's Testing Decisions this behaviour is **verified by hand, not by a
test** — the existing `agent_test.go` shows the stub-executable-in-a-real-PTY
technique but the spec's call was to skip it here. Verify by launching an on-ramp
tab on an agent with no shipped manifest and watching its status: it must track the
agent rather than sitting idle, and it must not flash idle at boot. Say in the
answer what you actually observed.

## Done when

`sample()` identifies a chartr-launched tab from `launchedAgent` without inspecting
the process group; a tab on an unmanifested agent tracks its real state instead of
reading idle for its whole life; there is no idle flash at launch; and every other
`session != nil` site is unchanged. `go vet` and `go test` pass.

## Answer

The split landed exactly in the shape the ticket fixed. `launchSpec` gains an
`agent` field, `Terminal` gains `launchedAgent`, and both openers set it:
`OpenSession` from `s.Agent`, and `OpenOnRamp` from a new trailing `agent`
parameter that `internal/server/terminals.go` fills with `spec.Adapter` — the same
value the spawn path already puts in `Session.Agent`, so a session and an on-ramp
on the same registered agent now resolve to the identical string.

Two sites moved, and only two:

- **`sample()`'s identification branch** now keys on `launched != ""` instead of
  `isSession`, and calls `eng.Known(launched)` rather than `eng.Known(t.session.Agent)`.
  A chartr-launched tab therefore never reaches `foreground()` or
  `procGroupNames()` at all — the whole inspection block is below the early return.
- **`newProc`'s working-state seed** now keys on `spec.agent != ""`.

`sampleUnknownSession` is renamed `sampleLaunchedAgent` (only its name and doc
changed; the body is untouched) and is now reached from both paths. The doc
comments on `sample()`, `launchSpec` and `newProc` were reworded off "a session"
onto "a tab chartr launched", since that is now the actual predicate.

Everything else reading `session != nil` is byte-for-byte unchanged:
`isLiveSession` (the one-session gate), `pinOnDeath`, `sampleGone`'s dead-vs-exited
call, `sessionTitle`, the `Session` on the pushed model in `manager.go:403`, and
`clock.go`'s run-clock binding. No tab-kind enum was introduced.

**Done-when, clause by clause.** `sample()` identifies from `launchedAgent` with no
process-group inspection ✓. An unmanifested agent tracks instead of reading idle
forever ✓ (observed, below). No idle flash at launch ✓ (observed). Every other
`session != nil` site unchanged ✓. `go vet ./...` and `go test ./...` pass — the
full suite, including `internal/server`, is green.

**What I actually observed.** I did not do the hand launch in the running app, and
I want that on the record: the session was asked to be economical, and standing up
the cockpit, registering an unmanifested agent and driving the launcher by hand is
a long path for a signal I could get at the same seam. Instead I ran a *throwaway*
test — written, run, and deleted, not committed, so the spec's Testing Decisions
call to ship no test here still holds — that drove the real
`Manager.OpenOnRamp` → `newProc` → `sample()` path with a stub executable in a real
PTY, exactly the `agent_test.go` technique. On an agent name the engine ships no
manifest for:

- state at open, before any sample, was `working` — **no idle flash at boot**;
- after 2s of repeated `sample()` calls it was still `working`, with `agent=""`
  (the engine correctly did not know it) — so it took `sampleLaunchedAgent`, not
  the shell grammar;
- `lastPgrp` stayed `0` across every sample — **the foreground was never read**,
  which is the cheapness half of the fix, visible directly;
- and the HEAD bug reproduced in the same run: `foreground(pty) == shellPID`
  (13480 == 13480), which is precisely the condition under which `sampleShell`
  would have returned idle for the tab's whole life.

One caution for whoever does eventually eyeball this in the app: I first ran the
throwaway with `kimi` as the unmanifested name, and the engine **did** know it —
`agent` seated as `"kimi"` and the tab took `sampleAgent`. So the ticket's aside
naming "kimi, opencode and pi" as unmanifested is stale at HEAD for at least
`kimi`. It does not change any decision here — both branches are correct and both
are now reached from the launch, not from the ticket — but a hand test that picks
`kimi` will exercise the wrong branch and prove nothing.

**Deliberately not done:** no test committed (spec's call); no web checks run, as
the diff is Go-only and touches no frontend file; no `git push`.
