---
type: task
blocked_by: [02]
---

# Maps appear and derive

## Question

A registered space's maps enter the snapshot, live. Port wayfinder-maps' model layer with its test suite (ADR 0001) and extend its derived-status table with `proposed` (`## Proposed Answer` present, `## Answer` absent — ADR 0004) and the chartr's stricter frontier (a blocker must be resolved, which on disk means blessed). Discovery is by notice, not refresh: the filesystem watch surfaces a map created by a hosted shell, an external terminal, or a `git pull` — and it reads wherever wayfinder writes, handling the current `.plan/<slug>/` layout and tolerating `.plan/maps/<slug>/`, hard-coding neither. A malformed map renders as-is with the malformation surfaced, never refused. The sidebar nests spaces → maps, finished maps sorting last.

Done when: process-boundary tests show a fixture map dropped into a registered space from outside appearing in the snapshot without any refresh action, under both layouts; derived statuses — including `proposed` and the stricter frontier — are asserted against fixture tickets; the ported model-layer tests pass; the sidebar shows the space's maps.

## Answer

A registered space's maps now enter the pushed snapshot, discovered live and derived through the ported wayfinder model layer. Layout:

- **`internal/wayfinder`** — the wayfinder-maps model layer ported whole (ADR 0001): `ParseMap`, `ParseTicket`, `Load`, derived `Status`, `Frontier`, and `Lint`, with the upstream commit recorded in `doc.go` and the ported test suite travelling with it as the guard against drift. The one chartr-specific extension is the non-resolving `proposed` status (ADR 0004) — `## Proposed Answer` present, `## Answer` absent — a distinct heading the exact-match section scan never confuses with `## Answer`, so the frontier scan stays blind to it and a vanilla wayfinder tool reads the same map unchanged. Because `proposed` is not `resolved`, the ported `Frontier` already **is** the chartr's stricter frontier: a merely-proposed (committed-but-ungated) blocker never unblocks its dependents — the containment.
- **`internal/mapscan`** — the chartr-side policy over the ported layer: layout-agnostic discovery and tolerant derivation. A map is found by the presence of its `map.md` rather than a hard-coded path, so the current `.plan/<slug>/` layout and the eventual `.plan/maps/<slug>/` one are both found without either wired in (story 12); only `.plan/` itself — fixed by the tracker — is named. A malformed map renders as-is with its malformation surfaced, never refused (story 17): an unparseable ticket becomes a surfaced defect and the rest of the map still derives, and lint diagnostics (a dangling `blocked_by`, a drifted index) fold in as surfaced malformations on the map they bite. Maps are ordered for the sidebar — finished (every ticket closed) last, then by slug.
- **`internal/server`** — discovery is by notice, not refresh (story 11): an fsnotify watch over each space's repo root and its whole `.plan/` subtree fires a debounced rebuild, so a map created by a hosted shell, an external terminal, or a `git pull` enters the snapshot with no operator action. The watch set is reconciled to the registry on every rebuild (a registered space starts watched, a forgotten one stops) and a watch is added to each new directory as it appears; a watcher that cannot start degrades to action-driven discovery rather than failing the chartr. `internal/model` grows `Space.Maps`, plus `Map` (slug, name, dir, destination, tickets, finished, malformations) and `Ticket` (num, slug, title, type, derived status, blockers, stricter-frontier membership).
- **`web/`** — the sidebar nests spaces → maps in server order (finished last), each map showing its frontier-ticket count, a finished check, and a ⚠ carrying its surfaced malformations.

Against Done-when: `internal/server/maps_test.go` extends the process-boundary rig and covers a fixture map dropped from outside appearing with no refresh action under **both** layouts (dialling the control socket first, then waiting for the notice); the derived statuses — open, claimed, proposed, resolved, out_of_scope — and the stricter frontier asserted against fixture tickets (a proposed blocker holds its dependent open-but-not-frontier; a blessed one releases it); a malformed map rendering with its malformation surfaced; and finished maps sorting last. The ported model-layer tests pass unchanged, and `internal/wayfinder/proposed_test.go` adds the `proposed` derivation and stricter-frontier cases at the model layer. `go vet ./...`, `go test ./...`, `svelte-check` (0 errors), and the Vite build pass.

Scope notes for review: this slice discovers and derives maps read-only. Map *kind* and the session actions it gates are ticket 04's — a discovered map here offers no actions and carries no declared kind; the star-map renderer and the ticket pane are tickets 06–07. The `.plan/` watch established here is also the config-change notice ticket 02 deferred, though bindings still resolve on each action rather than on the watch.

Review payload should carry this Done-when and the spec by assembly (spec, Prompts and payload).
