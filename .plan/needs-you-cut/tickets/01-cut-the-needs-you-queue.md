---
type: task
blocked_by: []
---

# Cut the "Needs you" queue; the sidebar flag becomes the jump

## Question

Delete the cross-space "Needs you" queue and move its one non-duplicated
capability — jump-to-the-halted-ticket — onto the sidebar's existing per-space
halt flag, so the signal has one surface instead of two.

**Delete, in `web/src/`:**

- `lib/NeedsYouQueue.svelte` — the whole component (`git rm`).
- `lib/attention.ts`: `needsYouQueue()` and the `QueueEntry` interface, plus the
  file-header comment that names the queue as one of the two altitudes.
  `mapActionItems` / `mapActionCount` / `spaceActionCount` / `spaceAttention` /
  `spaceLiveness` all stay.
- `App.svelte`: the `NeedsYouQueue` import and its instance at the bottom of the
  markup; the `needsYouQueue` / `QueueEntry` imports; `queueOpen` and
  `queueCount`; the branding-bar bell `Button`, its count badge and their
  wrapping `<span class="relative">`; the `q`/`Q` branch in `onGlobalKey`; and
  the `Bell` icon import if nothing else uses it.
- `lib/attention.test.ts`: the `describe('needsYouQueue', …)` block. Leave the
  `mapActionItems` and sidebar-echo blocks alone.

**Rework `jumpToQueueEntry` into the flag's action.** The existing function
already does the whole job — set `selectedId`, `navigate()` the
`#s=…&m=…&t=…` star deep link — it just needs a caller that isn't the sheet.
Give `attention.ts` a small replacement for the deleted queue: a function that,
for one space, returns the halted session's `{ mapSlug, ticketNum }` or `null`,
derived from the same terminal predicate `spaceAttention` uses so the flag and
the jump can never disagree. If a space has more than one halted terminal, take
the first in terminal order and say so in a comment — the flag is one glyph and
cannot offer a choice.

**Make the flag interactive** in the space card's name line. It currently renders
as a bare `<Warning>` icon with an `aria-label`; it becomes a real control that
jumps on click. Hold these:

- It lives inside a card that is itself `role="button"` with an `onclick` that
  selects the space, so the flag's handler must `stopPropagation` — the pattern
  the adjacent forget button already uses. Keyboard-reachable, with an
  `aria-label` that says what the click does, not just what the state is.
- Per `docs/design-system.md`: use the vendored `Button` primitive (the forget
  action's `variant="ghost" size="icon-xs"` is the precedent) and tokens only —
  `--destructive` for the warning, no raw colour, no hand-rolled chrome class.
- Story 8's rule is untouched: the flag flags a row, it never re-sorts one.
  Card ordering, the selection target, and the forget action all behave exactly
  as before.

**Amend the spec** at `.plan/chartr-design/spec.md` — it is the live source of
truth and must not keep describing a surface that does not exist:

- Strike **story 63** (line ~96) outright, or restate it as the sidebar flag's
  jump. Note the strike so the change is legible rather than silent.
- Line ~13 (the one-paragraph "concretely, it is…") drops "a cross-space
  'Needs you' queue".
- Line ~174 (cross-space attention "at two altitudes") collapses to the one
  ambient altitude that survives, with jump-to on the flag.
- Line ~175 and **story 30** (keyboard-first navigation) drop "queue open" from
  the keys that must exist; map summon and space switch remain.

Leave `chartr-design-impl` ticket 14 and its answer untouched — a resolved ticket
is history, not spec.

Done when: `git grep -i "needs.you\|needsYouQueue\|QueueEntry\|queueOpen\|queueCount"` returns
nothing under `web/src/` and nothing in `.plan/chartr-design/spec.md`; pressing
`Q` does nothing and the branding bar has no bell; a space with a halted session
still shows its flag, and clicking that flag selects the space and lands on the
halted ticket exactly as the queue's row did (verified in the running cockpit,
not just by reading the diff); the frontend `check` / `build` / `vitest` and
`go vet ./...` / `go test ./...` are green; and there is no amber in the built
CSS.

## Answer

Cut as charted. `NeedsYouQueue.svelte` is `git rm`'d; `needsYouQueue`/`QueueEntry`
are gone from `attention.ts` along with the header comment that named the queue as
an altitude; `App.svelte` lost the import, the instance, `queueOpen`/`queueCount`,
the branding-bar bell and its badge, the `q`/`Q` branch, and the now-unused `Bell`
icon. `mapActionItems` / `mapActionCount` / `spaceActionCount` / `spaceAttention` /
`spaceLiveness` are untouched.

**The replacement derivation.** `attention.ts` gains `spaceHaltTarget(space)`,
returning the halted session's `{ mapSlug, ticketNum }` or `null`, read off the
same `session && !alive` predicate `spaceAttention` tests — so the flag and its
jump cannot disagree. Multiple halted terminals take the first in terminal order,
noted in a comment. The deleted `describe('needsYouQueue', …)` block is replaced by
a `spaceHaltTarget` block covering the target, the first-in-order rule, and the
flag/jump agreement.

**The flag.** `jumpToQueueEntry` became `jumpToHalt(space)` — same five lines
(set `selectedId`, `navigate()` the `#s=…&m=…&t=…` star link). The bare `<Warning>`
on the space card's name line is now a vendored `Button variant="ghost"
size="icon-xs"` on `text-destructive`, matching the adjacent forget action, with
`stopPropagation` on click and an `aria-label` naming the action. Ordering,
selection target and forget behaviour are unchanged — story 8 holds.

**Two seam bugs the jump exposed, fixed in `SpacePane.svelte`.** The jump did not
land at first, and the cause was not the flag: a cross-space deep link never
worked. (a) `SpacePane` learned about links only via `hashchange`, delivered a
task later — by then its own URL-reflecting effect, seeing a pane with nothing
open, had already `replaceState`d the link away. The hash-applying body is now a
named `applyHash()`, shared by the listener and a new effect that runs it when the
pane swings to another space, declared so one flush orders reset → apply →
reflect. Note the pane is a *single* instance whose `space` prop changes and whose
`active` prop is only `!route.settings`, so the effect keys on `space.id`, not
`active`. (b) The drop-on-map-change guard then cleared the star, because
`applyHash` sets `openSlug` and `selectedTicket` together; `applyHash` now records
`lastOpen` when a link seeds both — the same exemption that guard's first run
already makes for the boot link. This is why the ticket asked for verification in
a running cockpit rather than by reading the diff: the diff looked right twice.

**Spec amended.** Line 13 drops the queue from the one-paragraph description;
line ~174 collapses to the one ambient altitude with jump-to on the flag; line ~175
and story 30 drop "queue open" from the required keys. Story 63 is restated as the
flag's jump with a parenthetical recording what was struck and why.
`chartr-design-impl` ticket 14 is untouched.

**Verified in a running cockpit**, not by reading the diff — a second `chartr` on
:8811 over a throwaway data dir, two staged spaces (one halted session, staged by
binding `implement` to `true` so the process exits at once), driven headless over
CDP. With the *other* space selected: clicking the flag flips `aria-pressed` to the
halted space, sets the hash to `#s=…&m=demo-map&t=4`, and seats that ticket's
detail pane. Enter on the focused flag does the same. `Q` opens nothing and the bar
has no bell. Card order did not change.

**On the sidebar-scrolling regression:** nothing to report — with two spaces the
list never scrolled, so this session saw no evidence either way for the accepted
filter-box blind spot.

**Gates:** frontend `check` (0 errors), `build`, `vitest` (61 passed), `go vet`,
`go test` all green; no amber in the built CSS (hues 106–107 olive plus 22/27
destructive only).

**One deviation from Done-when, flagged rather than hidden:** the grep still
returns one line — story 63's provenance note names the map, `needs-you-cut`, which
matches `needs.you`. Every description of the surface is gone; the hit is the slug
of the map that cut it. Kept because the ticket also asked the strike be legible
rather than silent.
