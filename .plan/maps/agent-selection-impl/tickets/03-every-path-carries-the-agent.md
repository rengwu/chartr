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

## Answer

Every remaining path now carries an explicit agent, so nothing but ticket 04's
own refusal is left reaching for a binding. Five parts — the four the question
names, plus resume, which the question does not (see the deviation below).

- **`agentSpec` is the one explicit-selection resolver.** Ticket 01's
  `launchSpecFor` grew a sibling in `internal/server/spawn.go`:
  `agentSpec(res, name)` is the whole of the named-agent path — unregistered is
  `400`, registered-but-off-PATH is `409` carrying the library's own `Missing`,
  and an empty name is `400` — and `launchSpecFor` now delegates to it. Every
  surface that names an agent refuses the same three ways in the same order,
  in one place, because they all call the same function.

- **Ideate stops borrowing `grill`.** `handleIdeate` decodes `agent` and resolves
  it through `agentSpec`; the `bindingFor(…, RoleGrill)` indirection and the Go
  comment that was its only documentation are gone. Everything else about ideate
  is untouched: still ticketless, still no `terminal.Session`, still no claim,
  still no death halt — asserted, not assumed, in the tests below. The interface
  now says so: the ideate control's menu carries *"A live, ticketless agent tab
  opened on a starter prompt for thinking an idea through. Nothing is claimed,
  nothing is committed, and it ends when you end it."*

- **Respawn reuses the dead session's agent.** `terminal.Session` gained
  `AgentName` beside the `Agent` that already held the adapter — the two answer
  different questions (what a tab renders and what means something anywhere,
  versus what to relaunch), so both are kept. `handleRespawn` resolves that name
  rather than re-resolving the role, so "start over cleanly" composes a fresh
  payload and writes a fresh claim without changing what executes; an agent since
  deregistered or off PATH is refused with the message any other absent agent
  gets. `resolveBinding` had no callers left and went with them.

- **The "Next up" drawer names its agent and never bypasses the first choice.**
  `ActionStation.svelte` takes `agents` and `lastAgent` (threaded through
  `MapCard`) and reads `chooseAgent`. A **ready** row shows the agent's name and
  spawns with it in one click, as before. An **unchosen** row deliberately does
  not spawn: it selects its ticket and closes the drawer, leaving the operator on
  the deliberate control, and the drawer's own description says why. There is
  therefore no one-click path past the initial choice, and no second picker.

- **The payload preview names the agent and the command.** `PayloadPreview.svelte`
  takes the same two inputs and renders the resolved agent's name beside
  `agent.command` — which the server already builds through the seam that builds
  the real argv (`agentLibrary` in `spaces.go`, with its `‹opener›` placeholder),
  so the preview cannot drift from the launch. Nothing assembles a second command
  line in the frontend. Unchosen and empty each say what they are instead.

**One picker, actually one.** Ticket 02's split control lived inline in
`DetailPane.svelte`; ideate needed the same thing in two more places. Rather than
copy it, it was lifted into `web/src/lib/AgentSplitButton.svelte` — primary action,
caret trigger, and the agent list with absent binaries disabled and their reason on
the row. `DetailPane`, the sidebar's *Idea* button and `SpacePane`'s *New Idea*
are all now that one component, so "exactly one picker implementation" is a fact
about the codebase and not just about the drawer. No new primitive was vendored;
it composes the `button` and `dropdown-menu` already there.

**One deviation, raised rather than made quietly.** The question names four paths;
**resume is a fifth**, and it was re-resolving a role binding too. Leaving it would
have handed ticket 05 a live binding caller after ticket 04 had closed every other
door, and it is wrong on its own terms — resume is crash recovery for *this*
session, so re-deciding its agent from config is exactly what it must not do. It
now takes the same `launchSpecFor(…, sess.AgentName)` line respawn does. Ticket 04
should treat resume as a fourth path alongside spawn, ideate and respawn.

Against Done-when: `go vet ./...` and `go test ./...` pass, as do the frontend
`check` (0 errors), `build` and `vitest` (91 tests), with no amber in the built
CSS. `internal/server/ideate_test.go` gained `TestIdeateLaunchesTheNamedAgent`
(both `claude` and the named agent on PATH, so what is proven is the *name*
deciding; flags verbatim and in order, `claude` never launched, tab carries no
Session) and `TestIdeateRefusesAnUnknownOrAbsentAgent` (`400` and `409`, no tab
and no prompt written); its four existing tests now register and name an agent,
and `chartrtest.Ideate` takes one (with `IdeateRaw` for refusals).
`internal/server/halt_test.go` gained `TestHaltRespawnReusesTheDeadSessionsAgent`
(a present, recording `claude` proves the binding was *not* consulted; the
re-claim carries `Agent:`/`Adapter:`/`Args:`) and
`TestHaltRespawnRefusesWhenTheAgentIsGone` (`400`, no commit, dead tab still
pinned to retry from).

The runtime facts were driven against a real cockpit — a throwaway data root,
config root and space, a stub agent on PATH — since they are only real at runtime:
ideate refused `400`/`400`/`409` for no name, an unknown name and a PATH-absent
one and launched the named agent's flags verbatim; respawn reused `demo-fast` and
refused once it was deregistered without writing a commit; a drawer row read
*"unblocks 0 · demo-fast"* and spawned on one click (claim commit written), while
the same row with nothing resolvable read *"choose an agent"* and opened the
detail pane instead, writing nothing; and the payload preview showed
**demo-fast** over `demo-harness -m fast --sandbox`.

No ADR is touched. ADR 0002 holds — `agentSpec` asserts only that a registered
name exists and its binary is present, never what any flag means. ADR 0008 holds —
the claim is still chartr's one write and every new refusal lands before it. ADR
0012 is followed: tokens only (the disabled row's reason uses `--destructive`, the
palette's one chromatic token), vendored primitives, Phosphor icons, no new CSS.

Scope notes for review: `agent` stays **optional** on spawn, respawn and resume —
a request with no name still falls through to the binding, which is what keeps
this ticket independently green and makes ticket 04 the single place that closes
it. Ideate is the exception and has no fallback, because its binding was the thing
being deleted. The empty-library state still falls through to the server's refusal
on every surface (ticket 04 owns it). `terminal.Session.AgentName` is deliberately
**not** on the wire: nothing in the frontend needs it, and the session tab keeps
rendering the adapter as it did.

