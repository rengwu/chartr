# cut the "Needs you" queue — one signal, one surface

## Destination

The cross-space "Needs you" queue is gone, and the signal it carried lives in
exactly one place: the sidebar's per-space halt flag, which becomes the jump
affordance it always implied. Done looks like `NeedsYouQueue.svelte`,
`needsYouQueue`, `QueueEntry`, the branding-bar bell + badge, the `Q` shortcut
and `queueOpen`/`queueCount` all deleted; clicking the halt flag on a space card
jumps to the halted ticket exactly as the queue's row did; and the spec's story
63 is struck rather than left describing a surface that no longer exists.

## Notes

**This is a cut, not a redesign.** The queue duplicated a condition the sidebar
already renders always-on: `needsYouQueue` and `spaceAttention` test the same
predicate (`session && !alive`), and `attention.ts` says so in its own comments.
At the scale this cockpit actually runs — a handful of spaces, all visible
without scrolling — the queue could never report anything the sidebar was not
already reporting. What it added over the sidebar was one thing, **jump-to**, and
that is five lines (`App.svelte`'s `jumpToQueueEntry`: set the space, navigate the
star deep link). Move those five lines onto the flag and the whole surface is
surplus.

**The spec's own rationale had already eroded.** Story 63 asked for
"gate-level signals only, **reviews first**, jump-to". The review feature was cut
on the `simplify` effort, so the queue lost half of what it was for; what remained
was a dedicated sheet, a keyboard shortcut, a badge, a component and a typed
`QueueEntry` wrapped around one boolean per space, with `kind` a one-member union
(`'halt'`) held open for signals that never arrived.

**The ambient echo is the whole point — do not weaken it.** Story 8's rule
stands: the flag flags a row, it never re-sorts the list. Making it clickable must
not change the card's ordering, its selection behaviour, or the space-selects-on-
click target — the flag's click stops propagation and jumps; everything else about
the card is untouched.

**The known regression, accepted.** The sidebar's filter box hides spaces, and
with them their flags, so a halted space can be invisible while a filter is
typed — the queue was immune to that. This is accepted: the cockpit's spaces fit
on screen, and the queue is cheap to reintroduce (it is one pure function over the
pushed model) if the sidebar ever grows long enough to feel it. If this ticket's
session finds the sidebar routinely scrolling in real use, say so in the answer
rather than quietly keeping the queue.

**Resolved tickets are history, not spec.** `chartr-design-impl` ticket 14
built the queue and its answer describes it; leave that answer alone. The spec at
`.plan/chartr-design/spec.md` is the live source of truth and is what gets
amended.

**Before commit:** the CLAUDE.md gates — the frontend `check` / `build` scripts
and `vitest`, plus `go vet ./...` and `go test ./...` (the embed test compiles
against `web/dist/`), and no amber in the built CSS. Follow
`docs/design-system.md` for the flag's new interactive form: a vendored primitive
and tokens, no hand-rolled chrome.

**The wayfinder-adapter step is done for this map:** `[maps."needs-you-cut"]
kind = "implementation"` is recorded in `.chartr/config.toml`, committed alongside
these files (see `docs/wayfinder-adapter.md`).

## Decisions so far

<!-- one line per resolved ticket: gist + link. -->

## Not yet specified

<!-- Empty. The cut is settled; a ticket that surfaces a genuinely new question
     flags it for the operator rather than deciding it here. -->

## Out of scope

- **Touching `spaceLiveness` or the liveness dots.** Ambient liveness is a
  separate, weaker signal and stays exactly as it is.
- **Touching the map's own action station** (`mapActionItems` / `mapActionCount`).
  That is a different altitude — what to spawn next *within* a map — and is not
  what this cut is about.
- **Redesigning the sidebar card.** Only the halt flag gains behaviour; layout,
  ordering, selection and the forget action are untouched.
- **Rewriting resolved tickets or ADRs.** No ADR covers the queue; nothing here
  amends one.
