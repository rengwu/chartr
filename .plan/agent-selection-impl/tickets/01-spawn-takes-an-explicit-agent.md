---
type: task
blocked_by: []
---

# Spawn takes an explicit agent, and the space remembers it

## Question

Make it possible to say *which registered agent runs this session*, and have the
space remember the answer — without removing anything. This is the expand step:
the binding path stays exactly as it is underneath, so the cockpit behaves
identically for anyone who does not send the new field.

**The spawn request gains an optional agent name.** `handleSpawn` in
`internal/server/spawn.go` decodes `agent` alongside `role`. When it is present,
resolve it against the operator's library through `config.ResolveAgents` — the
same call `Resolve` already makes — and launch that agent's `{adapter, args,
prompt}` instead of the role's binding. A name that is not registered is a `400`
naming it; a registered agent whose binary is not on PATH is a `409` carrying the
library's own `Missing` message, refused on the same doorstep and in the same
order as the binding case (before the claim, before any write). When `agent` is
absent, nothing changes: `bindingFor` resolves the role exactly as today.

The cleanest shape is to stop threading `config.Resolved` through
`sessionLaunch` and thread the *launch spec* instead — adapter, args, prompt
delivery, plus the registered name when there was one. Both paths then converge
on one struct before `launchSession`, which is what makes ticket 04's deletion a
subtraction rather than a rewrite.

**The claim trailer records the choice and the mechanism.** In
`internal/server/claim.go`, `Agent:` becomes the *registered agent's name* and a
new `Adapter:` line carries the binary; `Args:` is unchanged. A local name means
nothing on a teammate's machine, so the trailer must carry both — the name for
what the operator chose, the adapter and args for what the line actually means
anywhere else. On the fallback path there is no name, so `Agent:` is omitted
rather than blank. The provenance trailers (`AdapterFrom`, `ArgsFrom`) stay for
now; ticket 05 removes them with the layers they name.

**The space remembers what it last spawned with.** Add `LastAgent string` to
`registry.Entry` beside the `LastActive` it already carries — `spaces.toml` under
the data root is chartr-written per-space state with exactly the right lifecycle,
so this needs no new file, no new store and no new seam. Write it after a
*successful* launch only, so a refused spawn changes nothing. Surface it on the
space in the pushed model (`internal/model`, and `Space` in `web/src/lib/model.ts`)
so the frontend can render it in ticket 02. Nothing consumes it yet.

**A remembered name that no longer resolves reads as nothing remembered.** Do
not substitute, do not fall back to another agent — the value is reported as-is
and the frontend treats an unresolvable name as unchosen. Keep that rule on the
read side so deleting an agent needs no registry rewrite.

Done when: `go vet ./...` / `go test ./...` and the frontend `check` / `build` /
`vitest` are green with no amber in the built CSS; process-boundary tests in
`internal/server/spawn_test.go` (following its existing stub-agent-on-PATH prior
art) show that spawning with `agent` launches that agent's flags **verbatim and
in order**, that an unregistered name is refused `400` and a PATH-absent one
`409` with **no claim commit written**, that the claim carries `Agent:`,
`Adapter:` and `Args:`, and that spawning without `agent` still resolves through
the role binding unchanged; a further test shows the space's `LastAgent` appears
in the pushed model after a successful spawn, is untouched by a refused one, and
survives restarting the server against the same data root.


## Answer

A spawn can now say *which registered agent runs this session*, and the space
remembers the answer — with the binding path still intact underneath, so anything
that does not send the new field behaves exactly as before. The expand step, in
four parts:

- **`internal/server/spawn.go` — the launch spec is the new seam.** `handleSpawn`
  decodes an optional `agent` beside `role`, and a new `launchSpecFor` settles
  what will actually run: a named agent is resolved against the operator's library
  (through the `Agents` that `config.Resolve` already carries on its `Resolution`,
  so no second read of the user config), and with no name the role's binding
  decides exactly as it did. Both paths converge on one `launchSpec`
  (`{Name, Adapter, Args, Prompt}` plus the two provenance layers) *before*
  `launchSession`, and `sessionLaunch` now threads that instead of
  `config.Resolved` — which is what makes ticket 05's deletion a subtraction: the
  launch mechanics no longer know role bindings exist. `specOf` keeps a binding's
  registered name when it resolved cleanly (a role assigned to an agent is still a
  name out of the library) and drops it when the binding names nothing registered.
  Refusals sit on the same doorstep and in the same order as the binding case —
  an unregistered name is `400`, a registered agent off PATH is `409` carrying the
  library's own `Missing` — before the frontier's claim and before any write.
- **`internal/server/claim.go` — the trailer records the choice *and* the
  mechanism.** `Agent:` is now the registered agent's name and is **omitted**
  rather than blank when there was none; a new `Adapter:` line carries the binary;
  `Args:` is unchanged. A local name means nothing on a teammate's machine, so the
  trailer carries both — the name for what the operator chose, the adapter and
  args for what the line means anywhere else (stories 30, 31). The subject falls
  back to the adapter when no name was chosen, so it is never an empty bracket.
  `Adapter-From:` / `Args-From:` are likewise written only when a binding decided:
  an explicit agent consulted no layers, and claiming it resolved from `built-in`
  would be a lie. They go with the layers they name in ticket 05.
- **`internal/registry` — the memory has a home, not a new one.** `Entry` gains
  `LastAgent` beside the `LastActive` it already carries, and `SetLastAgent`
  persists it through the existing atomic whole-file write. `spaces.toml` under
  the data root is already chartr-written, per-space, local and rebuildable —
  exactly this value's lifecycle — so this needed no new file, no new store and no
  new seam. `handleSpawn` writes it only after a *successful* launch, past every
  refusal, so a blocked spawn changes nothing; a failed persist costs one re-pick
  and never the running session, so it does not fail the request.
- **The pushed model carries it, unresolved.** `model.Space.LastAgent` and
  `Space.lastAgent` in `web/src/lib/model.ts`, reported exactly as the registry
  holds it. A name that no longer resolves is deliberately **not** rewritten or
  substituted server-side — the frontend reads an unresolvable name as nothing
  remembered (story 19), which is what lets deleting an agent need no registry
  surgery. Nothing consumes it yet; ticket 02 renders it.

Against Done-when: `go vet ./...` and `go test ./...` pass, as do the frontend
`check` (0 errors), `build` and `vitest` (85 tests), with no amber in the built
CSS. Four process-boundary tests in `internal/server/spawn_test.go` follow the
existing stub-agent-on-PATH prior art:
`TestSpawnWithAnExplicitAgentLaunchesThatAgent` puts *both* binaries on PATH — so
what is proven is the choice deciding, not the binding's adapter happening to be
absent — and asserts the chosen agent's flags reach the process verbatim and in
order with the opener last, that the bound `claude` was never launched, and that
the claim carries `Agent:`, `Adapter:` and `Args:`;
`TestSpawnWithoutAnAgentStillResolvesTheBinding` registers an agent, pointedly
does not name it, and shows the binding still runs with nothing remembered;
`TestSpawnRefusesAnUnknownOrAbsentAgentWithoutClaiming` covers the `400` and the
`409` with HEAD still unborn and the ticket still on the frontier; and
`TestSpaceRemembersTheAgentItSpawnedWith` shows `LastAgent` in the pushed model
after a spawn, untouched by a refused one, and read back by a second server
started against the same data root. `TestSpawnWiresTheWholeChain` and
`TestRegisteredAgentDrivesTheSpawn` were amended for the new trailer shape (the
former now also asserts the *absence* of `Agent:`), and `chartrtest` gained
`SpawnWithAgent` — a separate helper from `Spawn` so a test says which shape of
request it is making and the role-only path keeps a caller.

No ADR is touched. ADR 0002 holds: nothing added here knows what any flag means —
the library's list stays opaque and is passed through in order. ADR 0008 holds:
the claim is still chartr's one write, still pathspec-limited, and every new
refusal lands before it.

Scope notes for review: `terminal.Session.Agent` still carries the *adapter*, not
the registered name — ticket 03 extends it when respawn needs to reuse the dead
session's agent. Ideate still borrows the `grill` binding (ticket 03), respawn
still re-resolves a binding (ticket 03), and nothing in the frontend picks an
agent yet (ticket 02). The `agent` parameter stays optional until ticket 04.
