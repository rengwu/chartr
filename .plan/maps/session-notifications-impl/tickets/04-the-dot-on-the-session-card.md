---
type: task
blocked_by: [02]
claimed_by: s1113c0491582
claimed_at: 2026-07-30T19:04:24Z
---

# The dot on the session card

## Question

Record in the cockpit what the notification announced elsewhere. A tab that
finished a qualifying run the operator has not looked at carries a dot on its
card; focusing that tab clears it. This is the quieter half of the same event
ticket 03 sends to the OS, and the two are independent consumers — this ticket
does not depend on ticket 03 and must not import from it.

**The flag is server state, not client state.** `model.Terminal` gains a boolean
recording that the tab finished a qualifying run the operator has not yet seen,
set when the clock emits. Keeping it in the snapshot is what makes it survive a
browser reload — the event may well have fired with no browser attached at all,
which is the whole point of the effort — and a client-side flag would show nothing
in exactly that case.

**Focus clears it.** The client posts to a small per-terminal endpoint when the
operator focuses that tab, and the flag clears in the snapshot. There is no manual
dismiss, no clear-all and no unread count: focusing is the acknowledgement, which
is what keeps stale dots unrepresentable.

**`attention.ts`'s behaviour is untouched.** Its single `halt` flag, its separate
`Liveness`, and the precedence between them that the module explicitly leaves
undecided all stay exactly as they are. The dot renders on the session's card
only; no space row shows anything new. One comment in that file cites the
`agent-state-detection` map's *Not yet specified — Notifications* patch, which
this effort has settled and struck; correct the citation to point at this effort's
spec. That comment is the only permitted edit to the file. A collapsed space therefore surfaces nothing, which is a
knowingly accepted limit recorded in the spec — the OS notification covers that
case. Do not extend `Attention`, and do not add a roll-up to the sidebar row.

**Styling is tokens and primitives.** Read `docs/design-system.md` first. The mark
uses `--primary`, the emphasis role the chrome already reserves — no raw hex, no
amber, no hand-rolled component, Phosphor if it needs a glyph at all. The
star-map's status hues are not involved and `web/src/lib/starmap/theme.ts` is not
touched.

**It must be legible as a state, not just a decoration.** The dot needs an
accessible name so the card announces the difference; a bare coloured circle tells
a screen reader nothing.

Tests lead. In `internal/server`, through the same seam ticket 03 uses: the flag
appears in the model snapshot when the clock emits and clears after the seen
endpoint is posted; a run under *n* sets nothing; posting the endpoint for a tab
that carries no flag is a no-op rather than an error. In `web`, a vitest over the
pure derivation that decides whether a card shows the dot, following the existing
pure-helper tests in `web/src/lib`. Add a vitest asserting `spaceAttention` and
`spaceLiveness` return exactly what they returned before — the guard that this
ticket did not quietly widen the attention grammar.

Done when: a session that finishes a qualifying run shows a dot on its card; the
dot survives a browser reload; focusing the tab clears it; no space row changed;
`attention.ts` has no behaviour change and no export added; `go vet ./...`,
`go test ./...` and the frontend
`check`, `build` and `vitest` scripts pass, with no amber in the built CSS.

## Answer

The dot is one boolean on the tab, raised where ticket 02 left the seam and
cleared by the operator looking at it. Nothing imports ticket 03, and ticket 03's
notifier hangs off the same callback without touching any of this.

**What shipped.**

- `model.Terminal.FinishedUnseen` (`finishedUnseen` on the wire) is the flag: this
  tab finished a run worth interrupting the operator over and they have not looked
  at it since. It is per-tab runtime state on `terminal.Terminal`, guarded by the
  tab's own lock beside `proc`/`state`, surfaced through `Info` into every snapshot.
  Living with the tab is what makes its lifecycle free — a dot cannot outlive the
  run it records, and a dead session keeps its dot while it stays pinned.
- `Manager.MarkFinishedUnseen` / `Manager.MarkSeen` are the two writes.
  `MarkFinishedUnseen` no-ops on a tab that ended between the clock emitting and
  the call; `MarkSeen` reports whether it cleared anything, so the server pushes a
  model only when there was a dot to clear.
- `server.onRunFinished` is the fan-out: it is now what `NewManager` is handed in
  place of `nil`, it marks the tab and rebuilds. It is deliberately the *only* new
  place a consumer of `RunFinished` goes — ticket 03 adds its notifier call beside
  the mark, and the two never learn about each other.
- `POST /api/spaces/{id}/terminals/{termID}/seen` clears it. 204 whether or not
  there was a dot (the client posts on focus without knowing), 404 for an id that
  names no tab — the same answer ending a shell gives.
- `web/src/lib/unseen.ts` is the pure derivation, and it is deliberately two
  functions over one flag: `showsFinishedDot(t, inView)` for the card and
  `acknowledgesFinishedRun(t, inView)` for the post. For a flagged tab they are
  exact complements, which is what keeps a dot from sitting on the tab being
  stared at for the round-trip the clear takes. `App.svelte` posts through an
  effect over the active tab, with `inView` = the active row of the selected space
  and the settings surface not standing over the cockpit.
- The mark itself is a `bg-primary` circle on the tab's status line in
  `SpaceCard.svelte` — no new component, no raw colour, no glyph (a dot needs
  none). It is `role="img"` with the accessible name *"finished while you were
  away"*, so the card announces the state rather than leaving a screen reader a
  bare circle.

**Done-when, clause by clause.** A qualifying run shows a dot on the tab's card
(`TestFinishedRunRaisesTheDotAndFocusClearsIt`, driving a real shell through a
real 2s run against a 10ms threshold). It survives a reload because the flag is in
the snapshot — the test's every poll dials a fresh control socket, which is what a
reload is. Focusing clears it (the same test, through the endpoint). No space row
changed: the only edit to `SpaceCard.svelte` outside the terminal list is the
import. `attention.ts` has no behaviour change and no new export — only the stale
citation of the struck *Not yet specified — Notifications* patch, corrected to
this effort's spec, which the ticket named as the one permitted edit.

**Tests.** `internal/server/sessiondot_test.go`: the dot appears in the snapshot
and clears after the seen post; a 2s run under a 1m threshold raises nothing (the
tab reading idle again *is* the sample the clock ended the run on, so an absent dot
there is one that was never raised); the endpoint is a no-op on an unflagged tab
and a 404 on an unknown one. `web/src/lib/unseen.test.ts`: the derivation, the
complement property, and the guard — a tab carrying the flag reads to
`spaceAttention` and `spaceLiveness` exactly as the same tab without it, including
alongside a halted and a working session.

`go vet ./...`, `go test ./...`, `npm run check`, `npm run build` and `npm test`
all pass; the built CSS contains no `amber`.

**Deliberately not done, and one thing worth knowing.**

- No notifier, no notification content, no platform exec — ticket 03's, untouched.
- No roll-up to the space row, no extension of `Attention`, no unread count, no
  manual dismiss. A collapsed space still surfaces nothing, as the spec accepts.
- The flag is raised for **any** tab that finishes a qualifying run, not only a
  session — ticket 01 settled that an ad-hoc shell running a long build is a run
  like any other, and the sidebar renders both through the same card, so the dot
  follows the event rather than the session binding.
- The chrome half was verified by `check`, `build`, `vitest` and the Go
  process-boundary tests, not by driving a browser: the repo tests the frontend as
  pure helpers and has no component-render harness, and this session had no browser
  to drive. What a human should eyeball once is only the dot's size and placement
  on the status line — the data path behind it is covered.
