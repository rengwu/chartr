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
