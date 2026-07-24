---
type: task
blocked_by: [02]
---

# The backend stops serving kind

## Question

Remove kind from the model, the config resolver, the map scanner and the HTTP
surface. Nothing reads it any more after ticket 02, so this is deletion, not
migration.

**Delete `internal/config/kinds.go` outright** (`git rm`) — `DeclareMapKind` and
the kind validation it wraps.

**`internal/config/binding.go`:** delete `RolesForKind` (~:52–63),
`KindOffersRole` (~:82–91), `resolveKinds` (~:352–380) and its call site
(~:251), the `Kinds` field on `Resolution` (~:170) and its use in the return
(~:321), `rawMap` (~:220–223), and `Maps` on `rawWorkspace` (~:217). `Role`,
`Roles`, `RoleIsAFK` and `RoleForTicketType` all stay.

**`internal/mapscan/mapscan.go`:** delete `GuessKind` (~:181–202). Nothing
proposes a kind any more.

**`internal/model/model.go`:** delete `KindUnclassified` / `KindPlanning` /
`KindImplementation` (~:139–141), `ValidKind` (~:147), `Map.Kind` (~:182) and
`Map.KindGuess` (~:187), plus the doc comments at ~:132 and ~:155–156 that
explain the inert-until-classified rule.

**`internal/server/`:** delete `handleClassify` and the `Kind` request body in
`spaces.go` (~:98–127) and the `res.Kinds` fill at ~:204–206; the classify route
registration in `server.go` (~:110) and the stale mention in its comment at
~:105; the `Kinds[slug]` fill in `halt.go` (~:237–238); and in `spawn.go` the
unclassified refusal at ~:95–96 that ticket 01 deliberately left standing. Delete
`classify_test.go`; prune the kind cases from `configsurface_test.go` (~:214),
`spawn_test.go` and `halt_test.go`.

**Stale comments naming a deleted function.** `internal/config/userbinding.go`
(~:49, ~:54, ~:134) and `userbinding_test.go` (~:112) all explain themselves by
reference to "`DeclareMapKind`'s style". That style — append a blank line, then
the table — is still what those functions do; restate it in its own terms rather
than pointing at a function that no longer exists.

**Old configs must stay readable.** A teammate's checkout, and every space
already registered, still has `[maps."<slug>"]` tables on disk. With
`rawWorkspace.Maps` gone, confirm the TOML decode **ignores** them rather than
erroring or warning — write a test that decodes a config carrying a `[maps.*]`
table and asserts a clean resolution with no warnings. If the decoder turns out
to be strict, keep the field as an ignored placeholder and say so in the answer;
do not make a stale config a hard failure.

**Do not touch** `internal/prompt` or `cmd/webview`: their `Kind` symbols are
`PayloadPart.Kind` and `reflect.Kind`, unrelated to any of this.

Done when: `git grep -n "KindGuess\|RolesForKind\|KindOffersRole\|DeclareMapKind\|GuessKind\|ValidKind\|resolveKinds"`
returns nothing; `go vet ./...` / `go test ./...` and the frontend gates are
green; a space whose committed config still carries `[maps.*]` tables resolves
with no warnings; and `POST /api/spaces/{id}/maps/{slug}/classify` returns 404.

## Answer

**Kind is gone from chartr.** `internal/config/kinds.go` is deleted;
`RolesForKind`, `KindOffersRole`, `resolveKinds`, `rawMap`, `Resolution.Kinds`,
`GuessKind`, `ValidKind`, the three `Kind*` constants, `Map.Kind`,
`Map.KindGuess`, `handleClassify` and its route are all gone. The spawn path's
unclassified refusal — the one gate ticket 01 deliberately left standing — went
with them, so a discovered map is live: it opens and spawns the moment it is
found, on no config at all.

**Landed as one commit, not the map's usual several.** The split the map asks
for was not available here. Kind's producer (`mapscan`), its resolver
(`config`), its carrier (`model.Map`) and its four consumers (`deriveSpace`,
`spawn`, `frozenTicket`, the tests) are one type and its transitive users:
removing the field forces every reader in the same edit, and every partition I
could find left either a non-compiling tree or a commit whose only content was
dead code waiting for the next one. The build/test tripwire the discipline
exists to protect was run against the whole thing instead.

**Old configs cost a checkout nothing — the decoder was already lenient.**
`toml.Decode` is non-strict, so with `rawWorkspace.Maps` gone the leftover
`[maps."<slug>"]` tables are simply not decoded: no error, no warning, no
placeholder field needed. `TestStaleMapTablesAreIgnored` pins it at the
resolution level — a config carrying three stale tables (including one with a
kind that was never valid) resolves byte-for-byte identically to the same file
with the tables cut out — and `configsurface_test.go`'s fixture keeps its stale
table and now asserts the space warns about nothing, so the same fact is held at
the process boundary too.

**The done-when's 404 clause is not met, and the reason is pre-existing.**
Deleting the route unregisters it, but `spaHandler` is mounted at `/` and serves
`index.html` for any unmatched path — including an unmatched `/api/…` one — so
the classify POST now returns 200 with the SPA shell rather than 404. That is
true of every unregistered API path today, not something this cut introduced,
and making unmatched `/api/` 404 as JSON is a decision about the whole HTTP
surface rather than about kind. Flagged, not taken. The test asserts what the
cut actually guarantees instead: nothing answers the route, and a POST to it
writes no committed config.

**Test pruning went further than the three named files.** `implConfig` and
`planningConfig` existed only to make a map spawnable, so both helpers and all
sixteen of their call sites went — across `agents_test.go` and `ideate_test.go`
as well as `spawn_test.go` and `halt_test.go`. Every one of those tests now runs
against a space with no `.chartr/config.toml` at all, which is a stronger
statement of the cut than a config that declares nothing. `TestSpawnRespectsKind`
became `TestDiscoveredMapSpawnsWithNoConfig`, and it asks for `grill` on a bare
map — a role no config declared and, before ticket 01, a role a map had to be
declared `planning` to offer. `TestSurfaceNeverWritesKind` is deleted outright:
its subject was the binding editor refusing `field: "kind"`, which is now just
one unknown field among many, already covered by the unknown-field case in
`TestBindingEditArgsAndRefusals` (whose `"kind"` example became
`"colour"`, since naming a field that no longer exists teaches nothing).

**Stale comments restated rather than repointed.** `userbinding.go` (×2) and
`userbinding_test.go` explained themselves by reference to `DeclareMapKind`'s
style; they now state the style itself — *a blank line off whatever precedes it,
then the table*. Five more comments that named kind in passing were swept while
their files were open: the package doc on `WorkspaceConfigName`, the prompt-
delivery warning's "in the same spirit as an unrecognised map kind" aside,
`Space.Warnings`, `configsurface.go`'s "the same rebuild the classify action
triggers", and the spawn route's registration comment.

**`docs/adr/0014` still names `DeclareMapKind` twice** (as the easier write
`SetUserBinding` is contrasted against). Left alone: ADRs are decision records,
and ticket 04 owns the doc sweep — though 04's brief names ADR 0007, CLAUDE.md
and the adapter, not 0014, so it is worth a look there.

**The map's own Decisions index was lying, and now is not.** Linting this map
through `mapscan.Discover` turned up `resolved but absent from the map's
Decisions-so-far` against **all three** resolved tickets — `reDecision` wants
`(./tickets/NN-…)` and every entry since ticket 01 has written `(tickets/NN-…)`.
Pre-existing drift, invisible because the malformation surfaces in the cockpit
rather than in a gate. All three links are now `./tickets/…` and the map lints
clean; writing mine in the working form while leaving 01 and 02 broken would
have been the worse half-measure.

Verified: `go vet ./...` and `go test ./...` green, `svelte-check` 0 errors,
`vitest` 85 passing, `npm run build` clean, no amber in the built CSS. The
done-when grep returns nothing outside `.plan/` and `docs/`.
