---
type: task
blocked_by: [01]
---

# The frontend stops reading kind

## Question

Delete every place the cockpit renders, confirms, or branches on a map's kind,
so that a discovered map is simply a map: openable, spawnable, with no
classification step between it and the operator. The backend still *serves*
`kind` after this ticket — the frontend just stops caring — which is what keeps
every commit here shippable on a self-hosting repo.

**`web/src/lib/MapPickerCard.svelte` — the bulk of it.** Delete `doClassify`,
the `p` / `i` keydown branches (~:53), the `classified` derived (~:41), the kind
pill (~:103), and the entire unclassified branch (~:109–133: the "No sessions
run until you set this map's kind" line and both plan/impl buttons). Every tile
becomes what a classified tile is today — name, resolution meter, a live open
target. Prune the now-unused `typeLabel` map (~:28), the `Badge` import if
nothing else in the file uses it, and the header comment's ADR 0007 paragraph
(~:9–13, ~:24).

**`web/src/lib/MapCard.svelte`.** Drop the `classified` / `unclassified` split
(~:76–77) and the pinned-to-the-bottom unclassified section (~:437) — one flat
list of maps. The comment at ~:73 explaining why unclassified tiles are taller,
and the ADR 0007 note at ~:24–26, go with them.

**`web/src/lib/Settings.svelte`.** Delete the whole `Map kinds` section
(~:480–507: the comment, heading, the read-only list, and the "classify on the
star-map →" button). Check whether `onOpenMaps` still has another caller — if
that button was its only one, remove the prop and its wiring in the parent
rather than leaving a dead prop. In `layerRow` (~:662) the label
`'bindings & map kinds'` becomes `'bindings'`.

**`web/src/lib/model.ts`.** Delete the `Kind` type (~:76), `Map.kind` /
`Map.kindGuess` (~:93–94), and `rolesForKind` (~:246). Fix the comments at ~:80
(`kind` gates…) and ~:180 (`bindings (and, committed, map kinds)`).

**`web/src/lib/actions.ts`.** Delete `classifyMap` (~:196).

**`web/src/App.svelte`.** The comment at ~:274 and the settings-gear `title` at
~:638 both name kinds; so does `SpacePane.svelte`'s title at ~:340 and its
comments at ~:53 and ~:139. Reword rather than delete the titles — they still
describe a real surface, just one without kinds in it.

**`web/src/lib/attention.test.ts`.** The `map()` helper takes a kind argument
(~:109); drop it and update the call sites. The attention logic itself never
depended on kind and must not change behaviour here.

**Extra `kind` fields on the wire are ignored, not an error.** Confirm the model
types tolerate the server still sending `kind` / `kindGuess` — TypeScript's
structural typing should make this a non-event, but verify it at runtime in a
real cockpit rather than by reading the diff, because this is precisely the
window ticket 03 depends on being safe.

Done when: `git grep -in "kindGuess\|rolesForKind\|classifyMap\|unclassified" web/src`
returns nothing; the frontend `check` / `build` / `vitest` are green; and in a
running cockpit against the *unmodified* backend, every map in the picker opens
directly and spawns, including one whose `.chartr/config.toml` declaration has
been temporarily removed to force `kind: ""`.
