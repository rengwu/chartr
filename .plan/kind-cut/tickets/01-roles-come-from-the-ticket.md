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
