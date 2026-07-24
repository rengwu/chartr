---
type: task
blocked_by: [04]
---

# Delete the binding layer, the committed config file, and the config surface

## Question

Take it all out. Nothing has reached the binding path since ticket 04, so this is
subtraction: roughly 870 lines of Go, two test files, a settings section, a
committed file format, and two ADRs' standing. It is the largest ticket on the
map and the lowest risk per line — but it must land as **one** ticket, because
deleting `config.Resolved` breaks the model the settings surface renders, and
splitting it would mean parking a deliberately-dead `bindings: []` in the pushed
model for a whole ticket.

**Delete from `internal/config`:**

- `binding.go` — the `builtins` table, `Binding`, `Resolved`, `Layer` and its
  three constants, `Resolution`'s bindings, `Input`, `rawBinding`, `Resolve`,
  `apply`, `parseWorkspace`, `parseUser`, `findAgent`, `assignmentWarnings`,
  `retiredModelWarnings`, `unknownRoleWarnings`, and `WorkspaceConfigName`.
  **Keep** `Role`, `Roles`, `IsRole`, `RoleForTicketType`, `RoleIsAFK` and
  `LookPath` — roles still choose a skill, derive a default from a ticket's type
  and drive the AFK/HITL quiet hint; they simply no longer resolve to an agent.
- `userbinding.go` — the whole file. `BindingEdit`, `SetUserBinding`, the field
  constants and the comment-preserving TOML line surgery exist to edit one field
  of one role table; there are no role tables.
- `agents.go` — keep it, and drop the retired `Model` field from `rawAgent`
  along with the warning it fed. There are no users, so an old key is simply an
  unknown key: **no migration, no warnings, no shims.** The library's *live*
  validation stays — an agent with no adapter, a bad name, an unreadable prompt
  delivery are current problems, not dead config.

**Delete the committed workspace config.** `.chartr/config.toml` is read in
exactly three places (`spaces.go`, `configsurface.go`, `spawn.go`), all for role
bindings; the skill library resolves from a *directory*, not that file. Remove
the constant, the reads and the concept. A space that still has the file on disk
is simply not read from — not warned about, not deleted, not migrated.

**Delete the provenance trailers.** `AdapterFrom` and `ArgsFrom` in
`internal/server/claim.go` name config layers that no longer exist. `Agent:`,
`Adapter:` and `Args:` from ticket 01 are the whole record.

**Collapse the settings surface.** In `web/src/lib/Settings.svelte`, remove the
role-bindings section, the per-field editors, the layer badges, `setBinding` in
`actions.ts`, and the `RoleBinding` type in `model.ts`. What remains is
`AgentLibrary.svelte` plus the file paths behind it and their open-in-editor
hatches — which is small enough to need no ADR of its own.

**Delete `internal/config/binding_test.go` and
`internal/config/userbinding_test.go` outright** rather than rewriting them: they
test layer merging, per-field provenance and TOML surgery, all of which cease to
exist. `internal/config/agents_test.go` stays and grows.
`internal/server/configsurface_test.go` shrinks to the library and its paths.

**Update the documents.** Supersede **ADR 0014** — it is built on per-field
provenance across layers, which is gone. Supersede **ADR 0009's execution half**:
there is no committed execution layer and so no layering question for bindings;
its content-versus-execution rule survives on the content side (skills), and its
safety property *strengthens* — with no committed execution config at all,
nothing about how an agent runs can arrive by `git pull`. **ADR 0002 is upheld**,
not amended. In `CONTEXT.md`: **Role** drops *"Resolves through config to a
concrete agent command"*, and **Agent** graduates from a term listed only under
*Avoid* to a first-class entry — a registered, named, complete way to run a
harness, chosen per spawn.

Done when: `go vet ./...` / `go test ./...` and the frontend `check` / `build` /
`vitest` are green with no amber in the built CSS;
`git grep -n "Resolved\|builtins\|SetUserBinding\|BindingEdit\|WorkspaceConfigName\|AdapterFrom\|ArgsFrom\|RoleBinding\|setBinding\|LayerWorkspace"`
returns nothing outside `.plan/` and `docs/adr/`; a space containing a leftover
`.chartr/config.toml` resolves clean at the process boundary with no warning and
no error; the cockpit still spawns, ideates and respawns end to end; the settings
route shows the agent library and its file paths and nothing else; and ADR 0014
and ADR 0009 carry their supersessions with `CONTEXT.md` updated.

**If the cockpit cannot spawn this ticket, no agent was ever registered.** Ticket
04 made one mandatory, so a data root with an empty library cannot start the very
session that would work this ticket — and the fix is not reachable from a cockpit
that will not spawn. Either register an agent and rebuild, or finish this ticket
the vanilla-wayfinder way: a terminal, `git`, and a `## Answer` committed by hand
(see the map's self-hosting note).

## Answer

The binding layer is gone; execution is the agent library and nothing else.

**`internal/config`.** `binding.go` is reduced to what the ticket names keeping —
`Role`, `Roles`, `IsRole`, `RoleForTicketType`, `RoleIsAFK`, `LookPath` — and
everything else (`builtins`, `Binding`, `Resolved`, `Layer` + its three constants,
`Resolution`'s bindings, `Input`, `rawBinding`, `Resolve`, `apply`, `parseWorkspace`,
`parseUser`, `findAgent`, `assignmentWarnings`, `retiredModelWarnings`,
`unknownRoleWarnings`, `WorkspaceConfigName`) is deleted. `userbinding.go` is gone;
`agents.go` drops the retired `Model` field and its warning (an old key is now just
an unknown key — no migration, no shim). `Resolution` survives, trimmed to
`{Agents, Warnings}`, as the thing every spawn surface consults. `AssignedRoles` is
deleted (no role tables to scan).

**Deviation, raised not quiet — the TOML line surgery.** The ticket says delete
`userbinding.go` "the whole file … the comment-preserving TOML line surgery." But
`useragent.go` (the agent-library *writer*, explicitly kept) shares that surgery to
edit `[agents.<name>]` tables — it is generic table editing, not binding-specific.
Deleting it outright would not compile. So the generic helpers moved to a new
`internal/config/tomlsurgery.go` and only the binding-specific top half of
`userbinding.go` was deleted. `userbinding.go` is gone as instructed; the shared
code that outlives it was relocated, not resurrected.

**Server.** `.chartr/config.toml` is no longer read anywhere: `s.resolve` resolves
the agent library from the operator's config alone, `deriveSpace` drops the
workspace read and the bindings loop (surfacing only live agent-library and
skill-library warnings per space), and `spaceLayers` no longer lists a
`workspace-config` layer. `handleSetBinding` and its route are deleted; the claim
struct and `claimMessage` drop `AdapterFrom`/`ArgsFrom` and the `*-From:` trailers —
`Agent:`, `Adapter:`, `Args:` are the whole execution record. `handleDeleteAgent`
no longer reports stranded assignments.

**Frontend.** `model.RoleBinding`/`Space.Bindings` and the TS `RoleBinding` +
`Space.bindings` are gone, `needsAgents()` with them (it read `space.bindings`; the
[[agent-badge-removed-from-sidebar]] note said don't re-add it, and it is now
removed, not revived). `setBinding` is deleted from `actions.ts`. Per the operator's
call, `Settings.svelte` collapses to **the agent library plus the file paths behind
it, and nothing else**: the role-bindings section, the per-field editors, the layer
badges, the "three layers resolve" explanation, *and* the skills-resolution rows are
removed — leaving `AgentLibrary.svelte`, a "Files on disk" list with open-in-editor
hatches, and per-space warnings. Skills stay resolved on the wire (content half,
untouched) and stale forks still surface as warnings; the settings route simply
stops rendering the layer-provenance UI. `ConfigLayer.holds` is now `agents | skills`.

**Docs.** ADR 0014 carries a full supersession banner; ADR 0009 supersedes its
execution half only (the content/skills half stands, and its safety property
strengthens — with no committed execution config, nothing about how an agent runs
arrives by `git pull`). ADR 0002 is upheld. `CONTEXT.md`: **Role** drops "resolves
to a concrete agent command"; **Agent** graduates from an *Avoid* term to a
first-class entry; the Configuration section's `Role binding` /
`Effective config surface` entries are replaced by `Agent library`,
`Committed skills`, and `Settings surface`, and `User config` now names the library.

**Gates.** `go vet ./...` / `go test ./...` and the frontend `check` / `build` /
`vitest` are green; no amber in the built CSS. The Done-when `git grep` returns
nothing outside `.plan/` and `docs/adr/` (the surviving `ResolvedSkill`,
`ResolveAgents`, wayfinder `StatusResolved`, and prompt's own skill-layer
`LayerWorkspace` are unrelated substrings). Spawn / ideate / respawn end-to-end and
the empty-library and leftover-`config.toml`-is-inert facts are exercised at the
process boundary with real stub-agent PTY launches.

