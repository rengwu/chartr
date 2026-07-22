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

