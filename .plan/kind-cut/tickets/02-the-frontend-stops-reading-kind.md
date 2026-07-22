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

## Answer

**The frontend no longer knows what a map's kind is.** `Kind`, `Map.kind`,
`Map.kindGuess`, `rolesForKind` and `classifyMap` are gone; the picker's
plan/impl confirm, its unclassified section, the kind pill and Settings' "Map
kinds" section are gone with them. A map arriving over the socket is openable
and its frontier tickets are spawnable, with nothing in between.

Landed as three green commits, in the order the surfaces stop depending on each
other: the picker's confirm (`d04aa3b`), the settings section (`f3efa77`), then
the model type and its last two readers (`0cc8fe6`).

**Three deletions the ticket did not list, each forced by one it did.**
`MapPickerCard`'s `spaceId` prop existed only to address `classifyMap`, so it
went with it and `MapCard` stopped threading it to the tiles. `attention.ts`
carried the *same* inert-map gate as the detail pane — `if (map.kind === '')
return []` at the top of `mapActionItems` — which the ticket named only through
its test; deleting `Map.kind` forced it, so an unclassified map's frontier
tickets now reach the action station like any other. And `actions.ts`'s
`spawnSession` doc listed "an unclassified map" among the refusals it surfaces;
it now names none of chartr's specific reasons, since the frontend renders
whatever message comes back and ticket 03 deletes that particular one.

**`mapsHash` stays.** `onOpenMaps` had exactly one caller — the deleted
"classify on the star-map →" button — so the prop, its wiring, and `App.openMaps`
went. That left `route.ts`'s `mapsHash` with no in-app caller, but `#s=…&maps=1`
is still a live route SpacePane parses and a human can type, and route.ts owns
both halves of the hash grammar. Deleting one half of a live route is a
different decision than this cut; flagged rather than taken.

**The attention test's unclassified case was rewritten, not dropped.** "offers
nothing on an unclassified map, even with a frontier ticket" asserted exactly
the behaviour this cut removes. It became the assertion that still means
something — a map whose only tickets are blocked offers no action items — so the
"empty is possible" case stays covered.

**The done-when's spawn clause cannot be met by this ticket, by design.**
`internal/server/spawn.go:96` still 409s a spawn on an unclassified map (pinned
by `TestSpawnRespectsKind`), and ticket 01 deliberately left it for 03. So a map
forced to `kind: ""` now *opens* directly and *offers* all four spawn buttons —
but clicking one gets chartr's "this map is unclassified" message surfaced
inline in the detail pane. This is the window ticket 01 named when it kept the
gate, now moved to where the map says it belongs: the frontend is ready, the
backend has one ticket left to catch up, and the reverse order is the one the
map forbids. Ticket 03 closes it. **Nothing here needs revisiting for that** —
the affordance and the refusal are already wired to each other.

**Runtime verification: partial, and here is exactly what was checked.** The
Chrome extension was not connected, so no human-eyes pass in a real cockpit
happened. Instead, against a build of this branch served on `:8811` (the
operator's own cockpit on `:8787` was left running and untouched), with
`[maps."reskin"]` temporarily removed from `.chartr/config.toml` and restored
after: the control socket was read directly and confirmed still sending
`kind: "implementation"` on the declared maps and `kind: "", kindGuess:
"implementation"` on `reskin`; those verbatim bytes were then mounted through
the *real* `MapCard` in jsdom, which rendered all seven maps — `reskin` among
them — as `Open <name>` targets with no "Unclassified" heading, no classify
group, and no error. The extra wire fields are the non-event the ticket
predicted. That harness was temporary and is deleted; it needed a `browser`
resolve condition the committed vitest config does not set, which is why it did
not stay. **The eyes-on pass in a real cockpit is still outstanding.**

**Gates:** frontend `check` (0 errors, 0 warnings), `build`, and `vitest` (85
passed, 9 files) green; `go vet ./...` and `go test ./...` green; no amber in
the built CSS (0 occurrences in `dist/assets/*.css`). The done-when grep returns
nothing.
