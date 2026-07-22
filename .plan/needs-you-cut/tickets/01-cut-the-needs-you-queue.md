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
