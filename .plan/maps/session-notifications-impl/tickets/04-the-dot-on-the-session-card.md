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
