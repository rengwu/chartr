---
type: task
blocked_by: [03, 04]
---

# The transparency surface

## Question

Ship the cockpit's first real route — a global settings screen that renders every
space's resolved config with its provenance layer and file location, edits the one
high-churn thing (role bindings) inline against the correct layer, and opens any
other layer file in the operator's editor. Legibility first: it shows what the
three documented layers resolve and never becomes a second config store. Blocked
by the vanilla-lifecycle revert (03, so there is no autopilot to toggle) and the
skills ticket (04, whose resolver provenance it renders). This writes **ADR 0014**
and amends **ADR 0009** with an edit-boundary consequence.

- **The route.** `App.svelte` reads a `#/settings` hash prefix
  (`#/settings/s=<spaceId>`, `#/settings/user`) in one `$derived` route and renders
  `<Settings.svelte/>` in place of the space cockpit — ~15 lines, no routing
  library; the bare star deep-link (`#s=…`, no `/` prefix) is untouched so the
  schemes never collide. A ⚙ button in the sidebar header and the `,` key enter;
  Esc or selecting a space exits. `SpacePane.svelte`'s bindings button rewires to
  **navigate** to `#/settings/s=<thisSpace>` rather than opening a local Sheet.
- **The read path** rides the existing per-space model push. `Server.deriveSpace`
  already folds `Bindings` (per-field provenance + PATH probe), `Kinds`, and
  `Warnings` into `model.Space`; add one derived field **`Space.Skills
  []ResolvedSkill`** (name → winning `Layer`, plus `forked_from` / stale) computed
  next to the bindings loop from ticket 04's resolver. `Settings.svelte` renders
  bindings (provenance + probe), resolved skills (winning layer + stale-fork),
  kinds (read-only, linking the existing classify action), each participating
  layer's file path, warnings, and a link to the existing payload preview.
- **The edit boundary.** Only role bindings are inline-editable, and only into the
  **user layer** (`[spaces."<path>".roles.<role>]`) — bindings resolve
  user-over-workspace (ADR 0009), so that *is* their home. Add a user-layer binding
  writer in `internal/config` behind `PUT …/config/binding`: a **key-level,
  comment-preserving** edit (harder than `DeclareMapKind`, which only appends to an
  absent slug) that sets or clears the specific `adapter` / `model` / `args` key,
  creating the table if absent in `DeclareMapKind`'s style, and leaves every
  surrounding byte — comments, ordering, unrelated tables — intact; clearing an
  override reveals the layer beneath (reversible); the handler rebuilds so the new
  provenance reflects straight back. The UI never writes committed workspace
  config, never writes kind (stays classify-only, ADR 0007), and has no autopilot
  toggle (deleted by ticket 03).
- **Open-in-editor.** `POST …/config/open` launches `$VISUAL` / `$EDITOR`, falling
  back to the OS opener (`open` / `xdg-open`) and finally to surfacing the absolute
  path; it opens a **named layer file resolved server-side** (workspace / user / a
  named skill dir), never a client-supplied path.
- **Explanation in place.** Provenance badges + a one-line layering caption + a
  "how resolution works →" link into the `core` / `tracker-convention` skills and
  `docs/adr/0009` — no in-app diagram or tutorial. Add a **CONTEXT.md** term
  *Effective config surface*.

Done when: the `#/settings` route renders every resolved value with its provenance
layer and file path from the pushed model; a `deriveSpace` test asserts
`Space.Skills` with winning layer + stale-fork; editing a binding writes only the
user layer with comments / ordering / unrelated tables byte-preserved, clearing an
override reveals the layer beneath and re-derives, and workspace config is never
written; the open handler resolves only server-named layer files (a
client-supplied path is refused); `vitest` covers the route and `config` unit
tests cover the writer; `go vet` / `go test` and the frontend gates are green; and
ADR 0014 is written with ADR 0009 amended.
