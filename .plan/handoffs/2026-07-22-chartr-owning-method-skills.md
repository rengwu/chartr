# Handoff — should chartr own the wayfinder method skills?

**Date:** 2026-07-22 · **Repo:** `chartr` (dir still named `wayfinder-harness`)

Two threads: one shipped change (uncommitted), one open decision (grilled to a
verdict, not yet written up).

---

## 1. Shipped, uncommitted — global skills in the settings surface

**Bug:** `Settings` → Global → *Your config* listed only the three shared config
layer paths. `Skills` was derived per space only, so you had to register a space
to see what your library resolved to. The open-in-editor action also routed
through `POST /api/spaces/{id}/config/open`, so it had no space id to use when
nothing was registered.

**Fix:** `Model.Skills` (library resolved with no repo in play) + a space-less
`POST /api/config/open`. The skill row is now a shared snippet used by both
scopes.

Touched: `internal/model/model.go`, `internal/server/{spaces,configsurface,server}.go`,
`web/src/{App.svelte,lib/Settings.svelte,lib/actions.ts,lib/model.ts}`,
`docs/adr/0014-the-effective-config-surface.md` (new paragraph: "The global scope
stands on its own").

New test: `TestGlobalSkillsResolveWithoutASpace` in
`internal/server/configsurface_test.go`.

**State:** all green — `go vet` / `go test ./...`, frontend `check` / `build` /
`vitest` (61), no amber in built CSS. Verified live against a dev server.
**Everything is uncommitted** (`git status` shows 10 modified files). This repo
commits straight to `main`; do not branch.

**Known gap, deliberately not fixed:** nothing watches the skills roots, so a
fork added while the app is running doesn't appear until the next rebuild (a
registry action or a space filesystem event). Predates this change.

---

## 2. Open decision — which method skills should chartr own, and how?

**The question.** rengwu (author of the skills at
`/Users/rengwu/Desktop/Projects/skills/pocock`) wants chartr to own
`wayfinder`, `to-spec`, `to-tickets`, `domain-modeling`,
`improve-codebase-architecture` — currently external, invisible to chartr, and
unreachable by any non-Claude adapter.

**Why it's a real defect, not a preference.** ADR 0002 forbids leaning on Claude
Code's skill loader, but `.plan/chartr-design/map.md:13` declares that every
session should consult `domain-modeling` — which chartr cannot resolve or
inject. It works today only because all four default bindings are `claude`
(`internal/config/binding.go:114-117`).

**Where the grilling landed.** The final proposal is to **wrap, not fork**:
inject the upstream body verbatim (hash-pinned) plus a chartr adaptation as two
tagged segments of one part. `prompt.Segment` was built for exactly this and has
been degenerate since whole-skill shadowing; `PayloadPreview.svelte:162` already
renders per-segment provenance.

**The rule that decides each skill:** a wrapper can *add* but cannot *retract* a
sentence the model also reads.

| skill | verdict | reason |
|---|---|---|
| `domain-modeling` | wrap | no interview; the only one of the five that is model-invocable |
| `wayfinder` | wrap | `SKILL.md:25` already says "consult the adapter for this repo" |
| `to-tickets` | wrap | accepts "a plan, spec, **or** the current conversation" — select the branch |
| `to-spec` | out | "synthesize the current conversation" would have to be negated; chartr has no spec artifact |
| `improve-codebase-architecture` | out | emits an HTML report chartr has no surface for |

Stated once: **wrap** when there's an extension point → **re-author** when you'd
have to retract → **restate** when you need a format, not a method. Chartr
already does the latter two (`grill`/`research`/`prototype` are re-authored;
`tracker-convention` restates the map format).

**Three problems to solve before implementing:**

1. Upstream bodies still ship as a copy inside the binary (`//go:embed
   assets/skills`) — the win is hash-pinned rather than hand-edited. Reading from
   a local skills dir at runtime is ruled out: `Payload.Skills` hashes become the
   claim commit's provenance trailers, so two machines would commit different
   provenance for the same ticket.
2. Whole-skill shadowing (ADR 0009) wins the *entire* directory, so a user fork
   of `wayfinder` would eat the chartr adaptation — and a forked wayfinder that
   loses the adapter stops declaring map kind, which is the ADR 0007 failure.
   The adaptation must compose independently of which layer won the skill.
3. Wrapping `wayfinder` would inject `TRACKER-MARKDOWN.md` alongside
   `tracker-convention` — the map format twice, free to diverge. Inject the
   upstream `SKILL.md` only.

**Revisit trigger:** the first wrapper containing the word "instead" or "ignore."
That skill needed re-authoring, not wrapping.

**Key evidence gathered (don't re-derive):** `wayfinder`, `to-spec`,
`to-tickets`, `improve-codebase-architecture`, `grill-me`, `grill-with-docs` all
carry `disable-model-invocation: true`; `domain-modeling` does not. Chartr's
`grill` / `research` / `prototype` are **not** ports of the pocock skills of the
same name — they are original chartr skills built around an injected payload and
a `## Answer` section. That precedent (3 for 3) is why "maintain my own branch"
was rejected in favour of wrap-or-re-author.

**Next step, unanswered:** the user was asked whether to chart this as a planning
map or draft the ADR directly, and asked for a handoff instead. Both are still
open. The decision is ADR-shaped — it adds a third content strategy and changes
the layering rule so an adaptation survives shadowing.

---

## Suggested skills for the next session

*Apply only if available in that agent's environment.*

- **`wayfinder`** — chart decision 2 as a planning map if it needs more than one
  session; the wrap/re-author rule, the layering change, and the composition
  change are separable tickets.
- **`domain-modeling`** — the decision adds vocabulary ("wrap", "adaptation",
  "restate") that belongs in `CONTEXT.md`, and it is ADR-shaped for `docs/adr/`.
- **`grill-me`** or **`grill-with-docs`** — if the verdict needs more pressure
  before it is written down.
- **`to-tickets`** — only once the planning map is finished.

## Files worth opening first

- `internal/prompt/compose.go` — where a wrap would land (`Compose`, `Segment`).
- `internal/prompt/prompt.go:353` — `Library` iterates the closed `Names()` set;
  an open namespace would change here.
- `docs/adr/0007-map-kind-declared-not-inferred.md` — "chartr stays a cockpit
  *over* wayfinder"; owning the authoring path strikes that consequence.
- `docs/wayfinder-adapter.md` — the existing, working extension point a wrap
  would inline.
