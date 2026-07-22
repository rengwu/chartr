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

## Answer

The cockpit has its first real route, and the three config layers have a face.
`#/settings` renders every value they resolve — with the layer it came from and
the file that layer lives in — and edits exactly one thing.

**The route.** `web/src/lib/route.ts` is a parser and its inverse, ~40 lines, no
routing library: `App.svelte` holds `hash` in `$state`, keeps it fresh from a
`hashchange` listener, and derives `parseRoute(hash)`. The settings scheme is
prefixed (`#/settings`, `#/settings/user`, `#/settings/s=<id>`) and the star
deep-link never is (`#s=…`), so the two are **disjoint by construction** —
`parseRoute('#s=settings')` is the cockpit, and the pane's own hash handling was
untouched. A ⚙ in the sidebar header and `,` enter; Esc, the ⚙ again, or
selecting a space leave. `SpacePane`'s bindings Sheet is deleted; its button
navigates to `#/settings/s=<thisSpace>`, so config has one home.

**One deviation from the ticket's letter, for the islands.** The ticket says the
route renders *in place of* the space cockpit. It renders **over** it instead:
`<main>` is `relative` and the surface is an `absolute inset-0` layer. Replacing
`SpacePane` in the tree would unmount the xterm terminal and the canvas star-map
— imperative islands (ADR 0010) — costing a socket re-attach and the map's open
state every time the operator glances at config. The pane stays mounted and takes
an `active` prop that makes it **inert**: `onKey` returns early, and the effect
that reflects its selection into the URL stands down while the route owns the
hash, restoring its own link when the pane is live again. Same appearance, no
island churn, and the two Esc handlers no longer race.

**The read path** rides the existing per-space push, as specified.
`deriveSpace` gained `Space.Skills` (name → winning layer, `forked_from`, stale)
from ticket 04's resolver, computed next to the bindings loop. Layer *paths* are
derived too, split by scope: a space's own two on `Space.Layers`, the three
shared by every space on a new **`Model.Config`** — the ticket asked for "each
participating layer's file path", and the global user file is not a space's, so
repeating it under every space would have been a lie about where it lives. That
also gives `#/settings/user` something to render on its own.

**The split from ticket 04 is rendered, not papered over.** Bindings resolve from
`<dataDir>/user.toml`; the user *skill* layer is `<configDir>/skills/`. The
surface shows both paths under one `user` badge and says so in a caption. One
layer in ADR 0009's sense, two files — flagged forward by ticket 04, discharged
here.

**The write** is `config.SetUserBinding` (`internal/config/userbinding.go`): a
key-level TOML **line editor**, not a decode-and-re-encode. It parses just enough
TOML to find one table header and one key inside it — quoted keys, comments,
values that run past their first line — and sets, replaces, or deletes exactly
that key, keeping the file's own line endings and indentation. Comments, key
ordering, spacing and unrelated tables come back byte-identical (a test asserts
line-for-line that *only* the edited line changed). An absent table is appended
in `DeclareMapKind`'s style. It **refuses** rather than guesses: a role bound
through an inline or dotted table is left alone with "edit it by hand", because
writing the canonical table beside it would produce a duplicate key the decoder
rejects wholesale. `PUT …/config/binding` writes only `<dataDir>/user.toml` and
rebuilds; clearing reveals the layer beneath. Workspace config is never written —
asserted byte-for-byte after an edit.

**The hatch.** `POST …/config/open` walks `$VISUAL` → `$EDITOR` → the OS opener →
surfacing the absolute path, and resolves a **name** the server knows
(`workspace-config`, `user-config`, `builtin-skills`, `user-skills`,
`workspace-skills`, `skill:<name>`) — never a path from the client. A test drives
a stub `$VISUAL` and asserts it received the *server-resolved* path, and that a
path, a traversal, and an unknown skill are each refused. A layer with nothing on
disk is reported with its path and nothing is created.

**Kind stays classify-only.** The surface lists kinds read-only and links to the
star-map picker, where the confirm lives (ADR 0007). Reaching it needed one small
addition to the existing deep-link vocabulary — `&maps=1`, "open the picker" —
rather than a second classify path. There is no autopilot toggle and no
preferences table: with autopilot gone (ticket 03) there is no ad-hoc preference
left to invent one for.

**Explanation in place**: provenance badges on the same built-in/workspace/user
weight scale the payload preview uses, one line of layering, a link that opens
the `core` skill, and the ADR 0009 path. No diagram. A per-role "preview →"
opens the **existing** `PayloadPreview` on the space's first frontier ticket
(named in the tooltip) rather than rebuilding the assembly.

**Tested.** `internal/config/userbinding_test.go` covers the writer against a
hand-written file: set / insert / append / clear, byte preservation, multi-line
arrays edited whole, space scoping, and every refusal — asserting meaning through
`config.Resolve`, not just text. `internal/server/configsurface_test.go` asserts
the pushed surface (provenance, `Space.Skills` winning layer and stale fork,
kinds, every layer path), that an edit writes only the user layer and re-derives,
that clearing reveals the layer beneath, and the open handler's name resolution
and refusals. `route.test.ts` pins the two hash schemes apart. `go vet` /
`go test`, `check` / `build` / `vitest`, and no amber in the built CSS: green.

**Also swept, because it bit:** `.gitignore` never learned ticket 04's rename, so
running the harness from the repo root left an untracked `skills/` in the tree.
One line, ignore-only — ticket 06's character, but it dirties every run until
it lands.

**Not verified by eye.** The browser extension was unavailable this session, so
the rendering is covered by `svelte-check`, the build, and the route unit tests —
not by a screenshot. Worth one look at the live screen.

**ADR 0014** is written. **ADR 0009** is amended with the edit-boundary
consequence (a UI writes only the user layer, and only bindings — what
user-over-workspace *means*, not a policy on top) and the two-files note.
`CONTEXT.md` gains **Effective config surface**.
