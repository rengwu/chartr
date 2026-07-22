---
type: task
blocked_by: [14]
---

# Fix: the action-station hover ring and the dialog-guard keyboard races

## Question

The browser walkthrough ticket 14's Proposed Answer flagged as undone (browser
pass never run — no Chrome connection in the implementing session) surfaced
three real bugs once actually driven in Chrome. None block ticket 14's core
done-when — the drawer's ranking, spawn-from-drawer, the tucked-map badge, the
halt/queue flow, and the no-color-only audit all independently pass — but the
hover-highlight and keyboard-toggle promises are broken.

**Hover draws nothing.** `web/src/lib/starmap/starmap.ts`'s `hover(num)`
(~line 240) only sets `this.#hovered`; nothing calls it to trigger a redraw.
`select()` happens to repaint because it also calls `#seat()`/
`#applySelection()`, which moves the camera and forces a frame; `hover()` has
no such side effect, so `#drawStar`'s dashed hover ring (~line 705) is
computed correctly but never reaches the canvas until some unrelated repaint
happens to run. Verified by direct canvas pixel sampling — the star center
located by color-matching (camera easing shifts it between interactions), the
ring-detection method cross-checked against the solid selection ring (which
scores clearly nonzero at the same radius band) — hovering an action-station
row never lights the ring, with real OS-level mouse hover onto the exact
button carrying the `onmouseenter`/`onmouseleave` handlers.

**`q` opens the Needs-you queue but can't close it.** Once the queue (or the
action-station drawer) opens, focus auto-lands on an element inside the
`role="dialog"` sheet. `web/src/lib/keys.ts`'s shared `isEditingTarget()` then
returns `true` for anything inside that dialog, which blocks the very `q`
handler (`App.svelte`'s `onGlobalKey`) meant to toggle it closed — `q` only
opens, never closes, once focus has moved inside the sheet it just opened.

**`Esc` closes the drawer *and* the map card underneath it in one keypress.**
Reproduced with focus confirmed to be inside the dialog beforehand (on the
Sheet's own Close button, `closestDialog: true`). `SpacePane.svelte`'s
`onKey` (~line 236) does gate its Escape-driven `dismiss()` on
`!isEditingTarget()`, but the Sheet primitive's own Escape handling appears to
run first, close the dialog, and return focus out of it — by the time
`SpacePane`'s bubble-phase listener evaluates the guard, focus has already
left the dialog, so the guard reads `false` and the map card closes too. Same
root cause as the `q` bug, opposite direction: the shared guard is a live
DOM-focus check evaluated independently by each listener rather than a
snapshot of state at the top of the event, so the ordering between the
dialog's own internal handler and the app's global listeners decides whether
it fires correctly or not.

Done when: hovering an action-station row visibly rings the star every time
(a redraw/dirty-flag call added to `hover()`, or equivalent); `q` toggles the
queue closed as reliably as it opens it, regardless of where focus landed
inside the sheet; a single `Esc` while the action-station drawer or the queue
is open closes only that sheet, never the map card underneath, no matter which
element inside the sheet currently holds focus. The keyboard races most likely
want one deliberate ordering decision rather than three independent patches —
e.g. each Sheet's own `onOpenChange`/close explicitly owning suppression of
the outer bindings while it's open, rather than every listener re-deriving
"am I inside a dialog" from live focus state at its own, uncoordinated time.

<!--
Source: a by-eye browser walkthrough of ticket 14, requested after the
implementing session flagged the browser pass as the one thing it couldn't
run (no Chrome connection). Filed as its own optional ticket rather than
folded back into 14, since 14's actual done-when (ranking, spawn, badges, halt
flow, color audit) all independently pass under this same walkthrough — this
is scoped to the hover/keyboard promises specifically, and does not currently
block anything.
-->
