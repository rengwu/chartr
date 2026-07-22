---
type: task
blocked_by: [02]
---

# Ideate, respawn, Next up and the payload preview carry the agent

## Question

Bring the four remaining paths onto explicit selection, so that when ticket 04
makes the parameter required nothing is left reaching for a binding. Three of
these are surfaces that must *name* the agent; two are server paths that must
*carry* one.

**Ideate stops borrowing `grill`.** `handleIdeate` in
`internal/server/terminals.go` currently resolves `config.RoleGrill`'s binding to
pick an agent — an indirection that exists only in a Go comment and appears on no
surface. It takes an explicit `agent` in its request body instead, resolved
against the library the same way ticket 01 resolved spawn's, with the same
refusals in the same order. Everything else about ideate is unchanged: it is
still a ticketless tab with a starter prompt, still carries no
`terminal.Session`, still writes no claim and still never enters the death halt.
Say that in the interface — the ideate control should explain what it opens
rather than leaving it to source comments.

**Respawn reuses the dead session's agent.** `handleRespawn` in
`internal/server/halt.go` re-resolves a binding from the dead session's role.
It should instead launch the agent that session used, read from the
`terminal.Session` the halted tab carries (extend it with the registered name
alongside the adapter it already holds). "Start over cleanly" composes a fresh
payload and writes a fresh claim; it does not change what executes. A respawn
whose agent has since been deregistered or fallen off PATH is refused with the
same message any other absent agent gets — surfaced, never silently substituted.

**The "Next up" drawer names its agent and never bypasses the first choice.**
`web/src/lib/ActionStation.svelte` spawns straight from a row today. Each row now
shows the agent it would use, taken from `agentchoice.ts`. In the **unchosen**
state a row must *not* spawn: it selects its ticket and leaves the operator on
the deliberate control, so there is exactly one picker implementation in the
codebase and no one-click path that skips the initial choice. The one-click ethos
survives for every subsequent spawn, which is the case that matters.

**The payload preview names the agent.** `web/src/lib/PayloadPreview.svelte` is
the pre-spawn transparency surface and currently answers only *what will this
session read*. It should also answer *what will run it*: the resolved agent and
the command line it produces, beside the composed context. The library already
builds that command server-side through the same seam that builds the real argv
(see `AgentLibrary.svelte`), so reuse it rather than assembling a second one in
the frontend — a preview that can drift from the launch is worse than none.

Done when: `go vet ./...` / `go test ./...` and the frontend `check` / `build` /
`vitest` are green with no amber in the built CSS; `internal/server/ideate_test.go`
shows ideate launching an explicitly named agent's flags verbatim and refusing an
unregistered or PATH-absent one, with the tab still carrying no session;
`internal/server/halt_test.go` shows a respawn launching the dead session's agent
rather than re-resolving a role, and refusing when that agent is gone; and in the
running cockpit a drawer row names its agent and spawns in one click once a space
has one, a drawer row in a space with nothing remembered selects its ticket
instead of spawning, and the payload preview shows the agent and command that
would run.

