---
type: prototype
blocked_by: []
claimed_by: claude-fable-5
claimed_at: 2026-07-18T15:01:27Z
---

# The cockpit layout

## Question

What does the thing look like, and how does it feel to drive?

The elements are known: a **space** switcher; **sessions nested under a space**, multiplexed; the **star-map**; and a **ticket pane** the human acts from — start a session on this ticket, review that one. herdr.dev is the reference for space-switching and nested-session multiplexing; wayfinder-maps' star-map is the reference for the map, and its `docs/starmap-design.md` explains what its look is protecting.

Make something cheap and concrete to react to, and settle:

- How the star-map and the terminals **share the screen**. Side by side, tabs, or the map as a home screen you dive out of and back into? A session needs a big terminal; the map wants room to breathe. They are both the main thing at different moments.
- Whether the **ticket pane** is a panel on the map, a modal, or its own surface — and how the human gets from *looking at a star* to *this ticket is now running*.
- **The tension worth resolving first:** the vision nests sessions under a space and multiplexes them, but only one session may run per space at a time (ADR 0003). So what *is* the nesting — live sessions across *other* spaces, the history of this space's past sessions, or something else? Either the multiplexer's job here is attention and history rather than concurrency, or ADR 0003 is being asked the wrong question. Say which.
- How **other spaces cooking in the background** are present without stealing focus.

The prototype is the deliverable — link it as an asset. Do not build the real frontend.
