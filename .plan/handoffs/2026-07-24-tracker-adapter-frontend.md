# Handoff — the tracker-adapter offer: backend done, frontend prompt to build

**Date:** 2026-07-24 · **Repo:** `chartr` (dir still named `wayfinder-harness`)

The backend for "chartr offers to install its tracker adapter into a repo" is
**shipped and committed**. What remains is the **frontend prompt** that renders the
offer and wires its two actions. This repo commits straight to `main` — **do not
branch**.

---

## What this feature is (one paragraph)

A vanilla wayfinder-family skill (Matt Pocock's, the common case) reads
`docs/agents/issue-tracker.md` to learn a repo's tracker, defaulting to
`.scratch/`. chartr watches `.plan/maps/`. So chartr offers to write its own
`docs/agents/issue-tracker.md` into a registered space, redirecting those skills to
`.plan/maps/` in chartr's format. The write is **consented, never silent**, and a
foreign file is **never clobbered** without an explicit replace. Full rationale is
in the git log of the three commits below and in the auto-memory
`one-spec-three-copies-drift.md`.

---

## Shipped & committed (backend — do not redo)

Three commits on `main`, tip `2f9c938`:

1. `cf44b62` — moved all maps to `.plan/maps/<slug>/`.
2. `2c74896` — bundled the adapter template + `internal/tracker` classify/install core.
3. `2f9c938` — wired the offer through the model + registry + two endpoints, with tests.

All green: `go vet ./...`, `go test ./...`. **Frontend is untouched** — that's your job.

---

## The API contract you consume

**The offer rides every control-socket snapshot** on each space. Server type
(`internal/model/model.go`), already on the wire as JSON:

```go
// model.Space gains:
TrackerAdapter *TrackerAdapterOffer `json:"trackerAdapter,omitempty"`

type TrackerAdapterOffer struct {
    State      string `json:"state"`                // "absent" | "stale" | "foreign"
    Path       string `json:"path"`                 // absolute docs/agents/issue-tracker.md
    RemoteHint string `json:"remoteHint,omitempty"` // "gh"|"glab"|"linear"|"" — foreign only, phrasing only
}
```

- `trackerAdapter` is **present only when there is something to act on** and the
  offer hasn't been dismissed. When the adapter is already current or the operator
  dismissed it, the field is **absent (undefined)** — so "show the prompt iff
  `space.trackerAdapter` exists" is the whole gating rule. `up-to-date` never hits
  the wire.
- The three `state` values map to three actions:
  - `absent` → **Install** (clean first write)
  - `stale` → **Refresh** (chartr's own copy drifted — a version bump or an operator edit)
  - `foreign` → **Replace or Leave** (a non-chartr file is in the way; `remoteHint`
    lets you say e.g. "looks like a GitHub tracker" — cosmetic only)

**Two endpoints** (`internal/server/server.go`):

- `POST /api/spaces/{id}/tracker-adapter` — install / refresh / replace (one write;
  the operator acting on the offer **is** the consent). Returns `200 {"path": "..."}`.
  After it, the next snapshot drops the offer (adapter now up-to-date).
- `POST /api/spaces/{id}/tracker-adapter/dismiss` — silences the prompt for good
  (persisted per-space), writes nothing to the repo. Returns `204`.

State flow you'll observe: act → server writes/records → `rebuild()` pushes a fresh
whole snapshot → the offer disappears. You never mutate model state client-side;
the snapshot is authoritative (ADR 0010).

---

## The frontend task — concrete

### 1. `web/src/lib/model.ts` — mirror the type

Add after the `Space` interface (the `warnings?` field is the last one today,
`model.ts:136`):

```ts
export interface TrackerAdapterOffer {
  state: 'absent' | 'stale' | 'foreign'
  path: string
  remoteHint?: string
}
```

and add `trackerAdapter?: TrackerAdapterOffer` to `interface Space`.

### 2. `web/src/lib/actions.ts` — two actions

Mirror the existing `send(...)` helpers (`actions.ts:8`, and e.g. `pinSpace` at
`:60` for shape). A `204` returns `null`; a non-2xx throws `ActionError` with the
server message — surface that to the operator, don't swallow it.

```ts
export function installTrackerAdapter(id: string): Promise<{ path: string }> {
  return send('POST', `/api/spaces/${encodeURIComponent(id)}/tracker-adapter`) as Promise<{ path: string }>
}
export function dismissTrackerAdapter(id: string): Promise<void> {
  return send('POST', `/api/spaces/${encodeURIComponent(id)}/tracker-adapter/dismiss`) as Promise<void>
}
```

### 3. A component that renders the offer

**This is chrome, not an island** — normal Svelte + tokens + primitives (ADR 0010).

**Placement decision (yours to make):** the offer is space-scoped. Two candidates:
- `web/src/lib/SpacePane.svelte` — the cockpit; a dismissible banner/card at the top
  is the most discoverable ("you just registered this space; want the adapter?").
  **Recommended.**
- `web/src/lib/Settings.svelte` — already renders per-space `warnings` (`:205`); a
  quieter home, but easy to miss on first registration.

I lean **SpacePane banner**, because the whole point is catching the user right
after they register a space with the wrong tracker. Confirm against how SpacePane
is laid out before committing to it.

**UI shape per state:**
- `absent`: one line ("Let chartr's skills write maps here") + **Install** button +
  a **Dismiss** (×). Show `path`.
- `stale`: "chartr's tracker adapter has an update" + **Refresh** + Dismiss.
- `foreign`: "An existing tracker config is here{ — looks like `remoteHint`}." Default
  action **Leave** (= call `dismissTrackerAdapter`); **Replace** is the secondary,
  and since it overwrites a foreign file it should get a confirm step (the `dialog`
  primitive) — don't one-click-overwrite someone's file.

### 4. Design-system rules (hard — read first)

- **Read `docs/design-system.md`** before writing any UI (ADR 0012; chrome/island
  split is ADR 0010). Also re-read the frontend section of `CLAUDE.md`.
- **A token for every colour** — never a raw hex/rgb/named colour. The chrome is
  monochrome (hue ~107); **no amber**; `--destructive` (red) is the only chroma, and
  it's the natural fit for the Replace-a-foreign-file confirm.
- **A primitive for every component** — reach into `web/src/lib/components/ui/`.
  Available and relevant: `button`, `card`, `badge`, `dialog`, `label`. **Never**
  hand-roll a `.btn`/`.card`/banner CSS; a genuinely new shared pattern earns *one*
  token-driven `@layer components` class, not a pile of one-offs.
- **Icons are Phosphor** (`phosphor-svelte`); text is IBM Plex. No CDN/runtime fetch.

### 5. The gate before committing (all must pass)

From `CLAUDE.md` → *Before committing frontend changes*:
- `cd web && npm run check && npm run build && npx vitest run`
- `go vet ./...` and `go test ./...` (the embed test compiles against `dist/`, so
  build the frontend before the Go test).
- **No amber in the built CSS.**
- Verify live against a dev server if you can (register a plain repo → see the offer
  → Install → offer clears → check `docs/agents/issue-tracker.md` landed).

Then commit straight to `main`.

---

## Open decisions for you

1. **Placement** — SpacePane banner (my lean) vs Settings row. Look at SpacePane's
   layout and pick.
2. **Foreign Replace confirmation** — I recommend a `dialog` confirm before replacing
   a foreign file (it's the one destructive path). Default/primary action is Leave.
3. **Copy** — keep it short; the user has ADHD and dislikes walls of text. One line +
   buttons.

## Deliberate non-goals (don't scope-creep)

- No re-offer UI after dismissal (a future Settings "install anyway" can clear the
  flag; not now).
- No per-version re-prompt logic — `TrackerDismissed` is a plain bool; a dismissed
  space stays quiet even across a template bump. Acceptable for v1.
- Backend doesn't commit the file — it only writes it. Don't add a commit step.

## Key files

- Contract: `internal/model/model.go` (`TrackerAdapterOffer`), `internal/server/server.go` (routes), `internal/server/spaces.go` (`deriveSpace`, handlers).
- Core (for reference, don't change): `internal/tracker/adapter.go`, template at `internal/prompt/assets/issue-tracker.md` served by `prompt.TrackerAdapter()`.
- Backend tests (the behaviour you're rendering): `internal/server/trackeradapter_test.go`.
- You edit: `web/src/lib/model.ts`, `web/src/lib/actions.ts`, `web/src/lib/SpacePane.svelte` (or `Settings.svelte`), plus a new component if you extract one.
