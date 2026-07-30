---
type: task
blocked_by: [02]
claimed_by: s1bc34dfce0e4
claimed_at: 2026-07-30T08:16:56Z
---

# Delete `pinned`

## Question

Remove the ordering authority the stored order replaced. This is the contract half
of the sequence: tickets 01 and 02 left `pinned` in place but reading it for
nothing, and it now goes end to end so the codebase describes what the app
actually does.

**What goes.** `Entry.Pinned` and `Registry.SetPin` in `internal/registry`; the
`POST /api/spaces/{id}/pin` route and its handler in `internal/server`; the
`pinned` field on the wire model in `internal/model` and its mirror in
`web/src/lib/model.ts`. There is no pin control in the cockpit to remove — the
flag was reachable only through the API — so this is a smaller deletion than it
looks. Confirm that before starting rather than assuming it: a control added since
this ticket was written is part of the deletion.

**The TOML key is ignored, not rejected.** An existing `spaces.toml` still
carrying `pinned` must load without complaint; the key is simply not read and
stops being written on the next save. An operator who upgrades does not have to
touch the file, and a stale key never produces a warning about something they
cannot act on.

**Comments are part of the deletion.** Several comments across `internal/model`,
`internal/registry`, `web/src/App.svelte` and the frontend tests describe the
sidebar as pinned-first-then-recency. That description is now false in every one
of them. A stale comment about ordering is exactly the drift that makes the next
session re-derive the wrong rule, so grep for it and fix each site.

**Existing tests that assert pin ordering are the specification of the old
behaviour** — `spaces_test.go` covers pin ordering directly. They are replaced by
the ordering assertions ticket 02 added, not deleted quietly; the replacement
should be visible in the diff as a substitution.

Tests lead: assert that `POST /api/spaces/{id}/pin` returns `404`, and that a
registry file still carrying `pinned = true` loads with every space present and in
its stored order — the flag having no effect on the sequence. `CONTEXT.md` has no
entry for pin; check rather than assume, and add nothing.

Done when: no `Pinned`, `SetPin`, `pin` route or `pinned` wire field remains in
the Go or TypeScript source; an old registry file loads cleanly and drops the key
on its next save; no comment anywhere still describes the sidebar as ordered by
pin or recency; `go vet ./...`, `go test ./...` and the frontend `check`, `build`
and `vitest` scripts pass, with no amber in the built CSS.

## Answer

`pinned` is gone end to end. `Entry.Pinned`, `Registry.SetPin`, the
`POST /api/spaces/{id}/pin` route, `handlePin` and `Space.pinned` on both sides of
the wire (`internal/model`, `web/src/lib/model.ts`) are deleted; the key is no
longer written, and an old `spaces.toml` still carrying it loads without complaint
and drops it on the next save. **One read survives, deliberately** — see the flag
below.

**Confirmed before starting, as the ticket asked.** There is still no pin control
in the cockpit: nothing under `web/src` read `space.pinned` except the type itself
and one test fixture, so nothing rendered it and nothing has been added since. The
deletion was as small as the ticket predicted.

**Each Done-when clause.**

- *No `Pinned`, `SetPin`, `pin` route or `pinned` wire field.* Deleted. The only
  `pin` left in the Go/TS source is the unrelated death-halt vocabulary (a dead
  session "pinned to its ticket"), the two tests that assert pin is gone, and the
  one migration read below.
- *An old registry file loads cleanly and drops the key on its next save.*
  `TestPinnedKeyIsIgnoredAndDroppedOnSave` writes three entries carrying
  `order` + `pinned`, asserts the stored sequence (the flag moves nothing), forces
  a save, and greps the file for `pinned`. Also verified against a real binary,
  isolated via `XDG_CONFIG_HOME` so no operator registry was touched: a
  pre-existing file with `pinned = true` loaded both spaces, `POST .../pin`
  returned `404`, and a reorder rewrote the file with the order applied,
  `last_active` intact and `pinned` gone.
- *No comment describes the sidebar as ordered by pin or recency.* Fixed at the
  registry package doc, `Entry`, `Deregister`, `List`, `handleDeregister`,
  `model.Space`, and the `registry_test` header. `App.svelte`'s two were already
  corrected by ticket 02. The remaining mentions of the old rule are in
  `ordered`'s doc and the migration test, where they describe the *frozen
  historical* rule — which is what that code implements.
- *The checks.* `go vet ./...` and `go test ./...` pass; frontend `check` (0
  errors), `build`, and `vitest` (196 tests) pass. The built CSS carries chroma at
  hue 27.325 and 22.216 only — the two `--destructive` reds — every other colour
  hue ~107. No amber. (`gofmt -l` reports `internal/terminal/detect/detect.go` and
  `internal/terminal/osc.go`, both pre-existing and untouched here.)

**Two things worth flagging.**

1. *The migration still reads the raw `pinned` key at load — the one deviation
   from "the key is simply not read".* The freeze is defined as *today's*
   sequence, and today's sequence is pinned-first-then-recency. 01, 02 and 03 ship
   as one upgrade, so the freeze that runs on an operator's machine is **this**
   code: if it cannot see `pinned`, a file carrying a pinned space freezes into
   the wrong sequence and the sidebar visibly reshuffles on first launch — exactly
   what story 4 and the spec's *"the migration is a freeze, not a reset"* forbid.
   Verified load-bearing: with the read stubbed out, `TestMigrationFreezesTodaysOrder`
   fails (`delta beta gamma alpha` instead of `gamma alpha delta beta`). So the
   flag is read in exactly one place — `Load`'s existing raw second decode, into a
   `legacyRow` that never reaches `Entry` — and only by `ordered`'s malformed
   branch. It orders nothing after the freeze, rides no struct, and is never
   written. Reading the ticket's clause as *"never rejected, never warned about,
   never written"* is what makes both halves of the spec true at once; deleting
   those four lines is the whole reversal if a human disagrees.
2. *The `404` needed one change beyond deleting the route.* `spaHandler` falls back
   to `index.html` for any unrouted path, so with the route simply gone a
   `POST .../pin` returned **200 and the app's HTML** — verified, not assumed. The
   SPA fallback now excludes `/api/`: an unrouted API path is a missing endpoint,
   not a client route. Small and correct on its own terms, but it is a behaviour
   change to every unknown `/api/` path, not just this one.

**Tests, as substitutions.** `TestPinNoLongerReorders` → `TestPinRouteIsGone`
(same shape, asserts `404` and an unchanged sidebar); ticket 02's ordering
assertions are what actually stand in for the deleted pin-ordering coverage.
`registry_test`'s `oldRule` oracle had to go — it sorted on `Entry.Pinned`, which
no longer exists — so `preUpgrade`'s expected sequence is now a literal
(`preUpgradeOrder`) with the derivation written beside the fixture. The other
degradation tests were untouched and still pass, including the two that depend on
a pinned entry leading the malformed tail. `payload_test.go` used the pin route
purely to force a rebuild; it now uses `PUT /api/spaces/{id}/agent`.

**Deliberately left out.** `CONTEXT.md` has no pin entry — checked, nothing added.
The README's roadmap still lists *"Drag to reorder the sidebar"* under **Not yet**:
its wording ("`pinned` goes away with the recency sort it fought over") is now
accurate rather than stale, and moving it to **Shipped in `v0.2.1`** would claim a
release that has not been cut. That is the operator's call at release time, not
this ticket's.
