# cut map kind — maps are maps

## Destination

A map's `kind` is gone from the chartr entirely, and the thing it approximated —
which role a session spawns as — is read off the ticket's own `type:`, where it
was always stated exactly. Done looks like `internal/config/kinds.go` deleted;
`Map.Kind` / `Map.KindGuess`, `Resolution.Kinds`, `resolveKinds`, `GuessKind`,
`RolesForKind`, `KindOffersRole`, `ValidKind` and the `POST …/classify` route all
gone; the map picker's plan/impl confirm and Settings' "Map kinds" section gone;
every map in every space openable and spawnable the moment it is discovered; and
`CLAUDE.md`, `docs/wayfinder-adapter.md` and ADR 0007 no longer asking a
map-creating session to record anything.

## Notes

**This is a cut, not a redesign.** Kind does exactly one thing today:
`config.RolesForKind` picks which of the four roles a map's tickets offer. That
set maps one-to-one onto wayfinder's own four ticket types — `grilling`→`grill`,
`prototype`→`prototype`, `research`→`research`, `task`→`implement` — and
`model.ts`'s `defaultRole(type, offered)` already derives the role from the type
and lets kind merely *clamp* it. A per-map uniform value was standing in for
per-ticket data the tickets already carry precisely.

**ADR 0007's own amendment struck the premise.** The ADR was decided on "the two
kinds have different lifecycles… getting it wrong lets code resolve unreviewed."
The `simplify` effort deleted review, and the ADR's amendment says so outright:
planning and implementation tickets now resolve identically. What the amendment
kept — kind selects the role set — is the redundancy above. The remaining ground
is the *inert-until-classified* gate, and that gate was bought with the
unreviewed-code failure mode. With review gone, the worst an ungated spawn does
is open a session on a ticket, which is the cockpit's entire purpose.

**The one real behaviour change is the clamp, and it is a fix.** Wayfinder
explicitly permits a `task` ticket on a planning map. Today that ticket's natural
role, `implement`, is clamped away to `grill` because the *map* said planning.
After this cut it offers `implement`, which is what the person who typed
`type: task` meant. Ticket 01 isolates this change so it lands on its own,
independently green, ahead of the deletions.

**Order matters, because this repo drives itself.** Every intermediate commit
must build, derive ticket status, and spawn — including on *this* map. So the
frontend stops *reading* kind (02) before the backend stops *serving* it (03).
The reverse order has a window where every map arrives with `kind: ""`, which the
current frontend renders as inert — a cockpit that cannot spawn the very tickets
finishing this cut. Do not reorder 02 and 03.

**Land small, independently-green commits** within each ticket, never a big-bang.
The discipline is the operator's, backed by the build/test tripwire.

**The spec is stale about more than kind — fix only kind.** `.plan/chartr-design/spec.md`
still describes agent review and the human review hub (line 11, 186, 189), which
the `simplify` effort cut and did not fully sweep from the spec. That is
pre-existing debt and **not this cut's job**. Ticket 04 amends only what kind
touches, and leaves the review staleness alone rather than quietly widening.

**Resolved tickets are history, not spec.** `chartr-design-impl` built the
classify flow and its answers describe it; leave those answers alone. The live
source of truth is the spec.

**Before commit:** the CLAUDE.md gates — the frontend `check` / `build` scripts
and `vitest`, plus `go vet ./...` and `go test ./...` (the embed test compiles
against `web/dist/`), and no amber in the built CSS. Follow
`docs/design-system.md` for any markup that changes: vendored primitives and
tokens, no hand-rolled chrome.

**The wayfinder-adapter step is done for this map — and ticket 04 undoes it.**
`[maps."kind-cut"] kind = "implementation"` is recorded in `.chartr/config.toml`,
committed alongside these files, because the rule is live *now* and this map must
not sit inert in the running cockpit while it is being worked. Ticket 04 removes
that table along with the rule that required it. The map deletes its own
admission ticket on the way out; this is the self-hosting working as intended,
not an inconsistency to resolve early.

## Decisions so far

<!-- one line per resolved ticket: gist + link. -->

- **01 — roles come from the ticket, not the map**: the cut's one real behaviour
  change has landed on its own, ahead of the deletions. `config.RoleForTicketType`
  (taking `wayfinder.Type` — `config → wayfinder` closes no cycle, so the
  one-to-one mapping is stated once, not restated as literals) picks a ticket's
  role from its own `type:`; every ticket offers all four roles and the operator
  picks at the gate. The spawn path's `KindOffersRole` refusal is gone; the
  unclassified refusal stays for ticket 03. `model.ts`'s `defaultRole` dropped its
  `offered` parameter and its clamp, and `PayloadPreview`'s byte-identical private
  copy folded into it. **Judgement call:** `DetailPane`'s footer now offers all
  four `ROLES`, with `rolesForKind` kept in the same function reduced to the
  classified-or-inert gate only — leaving it as the offered set would have left the
  pane's emphasised role missing from its own buttons for one commit, and removing
  the gate outright would have shown spawn buttons that the backend still 409s.
  Covered by a spawn test proving a `task` ticket on a `planning` map spawns as
  `implement`, plus a new `model.test.ts`. [ticket](./tickets/01-roles-come-from-the-ticket.md)
- **02 — the frontend stops reading kind**: `Kind`, `Map.kind`, `Map.kindGuess`,
  `rolesForKind` and `classifyMap` are gone, along with the picker's plan/impl
  confirm, its unclassified section and Settings' "Map kinds" section. Three
  deletions the ticket did not list followed from ones it did: `MapPickerCard`'s
  now-purposeless `spaceId` prop, `attention.ts`'s copy of the same inert-map
  gate the detail pane carried, and `spawnSession`'s doc listing an unclassified
  map among its refusals. **`mapsHash` kept** — `onOpenMaps` and `App.openMaps`
  went with the button, but `#s=…&maps=1` is still a live route SpacePane parses,
  and deleting half a live route's grammar is a different decision. **The
  done-when's spawn clause is unmeetable here by design**: the backend still 409s
  an unclassified spawn until 03, so such a map now opens and offers its four
  spawn buttons while the click surfaces the chartr's refusal inline — the
  window ticket 01 named, closing in 03, needing nothing revisited. Verified
  against a real running backend (socket still sending `kind`/`kindGuess`; those
  verbatim bytes mounted through the real `MapCard` render every map as an open
  target) — but **no eyes-on cockpit pass**: the browser extension was not
  connected. [ticket](./tickets/02-the-frontend-stops-reading-kind.md)
- **03 — the backend stops serving kind**: `internal/config/kinds.go` deleted;
  `RolesForKind`, `KindOffersRole`, `resolveKinds`, `rawMap`, `Resolution.Kinds`,
  `GuessKind`, `ValidKind`, the `Kind*` constants, `Map.Kind`, `Map.KindGuess`,
  `handleClassify` and its route all gone, along with the spawn path's
  unclassified refusal — so a discovered map is live, spawnable on no config at
  all, and the window ticket 02 named is closed. **Old configs cost nothing:**
  `toml.Decode` is non-strict, so leftover `[maps.*]` tables are simply not
  decoded — no error, no warning, no placeholder field — pinned at both the
  resolution level and the process boundary. **Judgement call:** landed as one
  commit rather than the map's usual several, because kind's producer, resolver,
  carrier and four consumers are one type and its transitive users; every
  partition left either a non-compiling tree or a commit of pure dead code.
  **The done-when's 404 clause is not met and was not forced**: `spaHandler` is
  mounted at `/` and serves `index.html` for *any* unmatched path, so the dead
  route answers 200 with the SPA shell — pre-existing for every unregistered API
  path, and making `/api/` 404 as JSON is a decision about the whole HTTP surface,
  flagged rather than taken. Test pruning went past the three named files:
  `implConfig` / `planningConfig` and all sixteen call sites are gone, so those
  tests now run against a space with no `.chartr/config.toml` at all.
  [ticket](./tickets/03-the-backend-stops-serving-kind.md)

## Not yet specified

<!-- Empty. The cut is settled; a ticket that surfaces a genuinely new question
     flags it for the operator rather than deciding it here. -->

## Out of scope

- **Changing wayfinder's ticket types.** The four types are the method's, not the
  chartr's. This cut *reads* them; it does not touch the format, the linter, or
  the `tracker-convention` skill's restatement of it.
- **Changing the role set.** `grill` / `prototype` / `research` / `implement`
  stays exactly as it is (ADR 0002 untouched). Only what selects among them
  changes.
- **Re-litigating role bindings or the config layers.** `[roles.*]`, the user
  layer, and the settings surface's binding editor are untouched except for the
  one read-only "Map kinds" section that has nothing left to show.
- **Sweeping the spec's review staleness.** Named in Notes; separate debt.
- **Adding any replacement gate.** No confirm, no "activate this map", no
  per-space allowlist. A discovered map is live. If that turns out to be wrong,
  it comes back as a new decision, not as kind under another name.
