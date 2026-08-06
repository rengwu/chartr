---
type: task
blocked_by: []
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
