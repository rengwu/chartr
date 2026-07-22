---
type: task
blocked_by: []
---

# Roles come from the ticket, not the map

## Question

Make a ticket's own `type:` select the role it spawns as, and stop letting the
map's kind clamp it — the one real behaviour change in this cut, landed on its
own so it is bisectable and reviewable apart from the deletions that follow.

**Add the derivation, in `internal/config/binding.go`:**

```go
// RoleForTicketType returns the role a ticket of this type spawns as. The
// method's four ticket types map one-to-one onto the four roles; an
// unrecognised type falls to implement, the same default the frontend has
// always used.
func RoleForTicketType(t string) Role {
	switch t {
	case string(wayfinder.TypeGrilling):  return RoleGrill
	case string(wayfinder.TypePrototype): return RolePrototype
	case string(wayfinder.TypeResearch):  return RoleResearch
	default:                              return RoleImplement
	}
}
```

Check the import direction before writing it this way — if `config` importing
`wayfinder` would cycle, take the type as a plain string and compare literals,
matching how `model.ts` already does it. Say which you did and why.

**Every ticket offers all four roles.** `config.Roles` is already the exported
closed set in display order; it becomes the offered set everywhere, with
`RoleForTicketType` picking the pre-selected default. This is trust-at-the-gate:
the operator chose a role in the spawn preview, so honour it.

**Drop the role refusal in `internal/server/spawn.go`** — the
`!config.KindOffersRole(m.Kind, role)` branch at ~:99 and its
"role X is not offered by a Y map" error. **Leave the unclassified refusal at
~:95 in place for now**; it still guards a real state until ticket 03 removes
kind from the model. Do not get ahead of the ordering.

**Frontend, `web/src/lib/model.ts`:** `defaultRole` loses its `offered`
parameter and returns the type's role directly — the `offered.includes(guess)`
clamp is exactly the behaviour being removed. Update its two callers (the detail
pane and the action station, ticket 14's shared default) so both still land on
the same role. Leave `rolesForKind` in place for ticket 02; just stop letting it
narrow the default.

**Cover the change that matters.** Add a test asserting a `task` ticket on a
map declared `planning` defaults to and successfully spawns as `implement` —
the clamp this ticket removes. That case is the whole behavioural delta; a
green suite that does not contain it has not tested this ticket.

Done when: `go vet ./...` / `go test ./...` and the frontend `check` / `build` /
`vitest` are green; a `task` ticket on a planning map spawns as `implement`
rather than `grill`; every ticket's spawn preview offers all four roles with its
type's role pre-selected; and `KindOffersRole` has no callers left (it is
deleted in 03, not here).

## Answer

**A ticket's own `type:` now picks its role, and nothing clamps it.** The clamp
is gone from both ends — the backend no longer refuses a role by the map's kind,
and the frontend's shared default no longer narrows to a kind's set.

**`config.RoleForTicketType` takes `wayfinder.Type`, not a plain string.**
Checked the import direction before writing it: `internal/wayfinder` imports no
internal package of ours, and `config` already imports `internal/model`, so
`config → wayfinder` closes no cycle. Taking the typed constants keeps the
one-to-one mapping stated once instead of restating the four literals a third
time (`wayfinder`, `model.ts`, and here). It lives in `internal/config/binding.go`
beside `Roles`, whose doc now says what it became: the set *every* ticket offers.

**`internal/server/spawn.go` lost the role refusal** (`KindOffersRole` branch and
its "role X is not offered by a Y map" 400). The unclassified refusal above it
stands untouched, as instructed — ticket 03 takes it with the rest of kind.
`KindOffersRole` and `RolesForKind` stay in place with no callers; their doc
comment now says so rather than claiming a gate that no longer exists.

**Frontend.** `defaultRole(type)` in `model.ts` dropped its `offered` parameter
and returns the type's role directly. Its two callers — the detail pane's
`preferredRole` and the action station's one-click `act()` — both pass just the
type, so they still land on the same role as each other. `PayloadPreview` had its
*own* byte-identical copy of the old un-clamped derivation; with the shared one
now identical, the copy is deleted and the preview imports the shared default, so
there is one definition on the frontend rather than two drifting ones.

**One judgement call, flagged: what the detail pane's footer offers.** The
ticket's "every ticket offers all four roles… everywhere" and its "leave
`rolesForKind` in place for ticket 02" pull in opposite directions for
`DetailPane.offeredRoles`. Leaving it as `rolesForKind(kind)` would have left the
pane visibly incoherent for one commit — on a planning map a `task` ticket's
`preferredRole` (`implement`) would not be among the buttons rendered, so no
button would be emphasised and the one-click implement spawn would be unreachable
from the pane. So the offered set became all four `ROLES`, and `rolesForKind` was
kept in the same function reduced to the only thing it still decides: whether the
map is classified at all (`.length === 0` → offer nothing). That is the inert-map
gate, which ticket 02 removes — deliberately *not* removed here, because with the
backend's unclassified 409 still standing until 03, offering spawn buttons on an
unclassified map would render an affordance that always fails.

**Tests.** Go: `TestSpawnHonoursTheTicketsOwnType` is the behavioural delta — a
`type: task` ticket on a map declared `planning` (via the existing
`planningConfig` helper) defaults to `implement` via `RoleForTicketType` and
spawns successfully as `implement`, seating a live `implement` session bound to
ticket 1. `TestSpawnRespectsKind` lost its "grill on an implementation map is
refused" half — that refusal is what this ticket deletes — and now asserts the
other side of the same gate: unclassified is refused, and the same spawn goes
through once classified. Frontend: a new `web/src/lib/model.test.ts` covers
`defaultRole` over all four types, the unrecognised-type fallback, and that it
never returns a role outside the closed set.

**Gates:** `go vet ./...` and `go test ./...` green; frontend `check` (0 errors),
`build`, and `vitest` (85 passed, 9 files) green; no amber in the built CSS (0
occurrences in `dist/assets/*.css`).

**Not verified in a running cockpit** — this ticket's delta is a pure logic change
covered end-to-end by the process-boundary spawn test (real HTTP, real git, real
PTY), and the UI change is one offered-set expression. Flagged rather than
claimed.
