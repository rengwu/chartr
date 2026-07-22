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

