---
type: prototype
blocked_by: []
assets: [.plan/harness-design/assets/08-cockpit-layout.html]
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

Prototype: [08-cockpit-layout.html](../assets/08-cockpit-layout.html) — open in a browser; seven variants cycled with ←/→ or `?variant=`; `?` explains how each answers the ticket. Round one: A Orbit (map as home), B Cockpit (persistent split), C Tower (terminal-first). Round two, from review: D Bridge (the B+C hybrid: space→map sidebar, space-scoped session tabs, badge-toggled action station, reading drawer). Round three, D's decisions in wildly different bodies: E Helm (terminal-anchored; the map is a floating panel sliding over the terminal's right half, one-click full-ticket bottom sheet with camera ease, branded collapsible sidebar), F Fleet (all spaces tiled at once, global action station), G Inbox (the action queue as the primary rail; map, tickets, terminals open as documents).

## Answer

**The cockpit is variant E ("Helm") in its final reviewed form — three surfaces: a branded collapsible sidebar for navigation, a full-width terminal column that owns the screen, and the star-map as a floating card summoned over it.** Seven variants were prototyped across three rounds ([the asset](../assets/08-cockpit-layout.html) holds all of them; E is canonical). Rejected: A map-as-home (immersive but the terminal is where the hours go), B/D persistent splits (neither surface ever big enough — the honest weakness a fixed split can't shed), C map-as-modal-overlay (right instinct, wrong chrome: its call to action hid with the map), F all-spaces-tiled (a monitoring posture, not a working one — a second-monitor view someday), G action-queue-first (demotes the map to a report; but its wants-you-first ordering survives in the action station).

**How map and terminal share the screen: they don't split — the map slides *over* the terminal, covering all but a sliver of padding.** The deciding argument is mechanical: xterm.js re-wraps every line on resize, so a pushing split makes the terminal churn on every toggle; an overlay leaves it untouched at full width. The map is a place you *consult* — summoned by its edge handle, `M`, or Esc-dismissed — and the covered terminal doesn't matter in the moment you're map-focused. Map visibility belongs to the human: switching spaces or maps never toggles it; only explicit acts do (spawn and open-a-terminal drop you onto the terminal). When the map is tucked away, its handle — sitting below the tab strip, never over it — carries the action-station badge, so hiding the map costs ambience, never awareness.

**The nesting tension: ADR 0003 stands, and the multiplexer's job is attention and history, not concurrency.** Nested under a space: its one live session plus its past sessions, as tabs on the terminal column — live first, history behind. Other spaces are present only as attention cues (status dots and a ⚑ on the sidebar rows), never as terminals of the current space. The sidebar's nesting is *spaces → maps*, pure navigation: selecting a map swaps the stars, never filters the terminals — sessions are space-scoped because the working tree is (ADR 0003). Spaces themselves are selectable; a selected space defaults to its first *unfinished* map (finished maps sort last, ✓-marked), and a mapless space is still usable: a "+" by the tabs opens an **ad-hoc shell** in the space's working tree. That shell is deliberately *outside* the session model — no ticket, no lifecycle — a human convenience that shares the tree with agent sessions; ticket 06 should account for a human dirtying the tree from one.

**From looking at a star to reading to running is one click each.** Clicking a star opens the *full ticket file* — question, Done-when, blockers with answers inline, session history — as a bottom sheet on the map card, capped at half its height, while the camera eases the star into the space the sheet actually leaves free (the wayfinder-maps pattern, kept). No compact-pane-then-expand: the sheet *is* the ticket pane. Its actions are status-appropriate (spawn / open review / open running session), and spawning tucks the map away onto the newly-live tab. The map's own material — destination, notes, decisions, fog — opens the same way from the map's title. Deep-links (`?sel=N`) name a star.

**The action station is a birds-eye "Next up" on the map card: a numbered badge toggling a drawer of everything actionable on this map** — reviews waiting first (the gate must be unmissable — G's lesson), then spawnable frontier tickets ranked by how many tickets each unblocks. Hovering an action highlights its star; clicking acts. It is map-scoped; cross-space attention stays in the sidebar. A global everywhere-queue (F's idea) is left as a lead for ticket 11's switcher. Cost and token figures are deliberately absent from the cockpit for now — ticket 14 owns that surface.

**Consequences for the successors:** ticket 10 (human review hub) inherits the bottom-sheet-on-map pattern and the reviews-first action station as its entry points; ticket 11 (space registry) inherits the sidebar — registration, discovery, ordering, and the unclassified-map state (ADR 0007) all live there; ticket 09 renders its new states onto stars that this layout keeps deterministic and never moves. The prototype is throwaway; the decisions above are the deliverable.

## Amendment (2026-07-19 — operator-approved, via ticket 11's prototype)

Two calls above were revisited while making [ticket 11's asset](../assets/11-space-registry-and-switching.html) drive a live map, and the operator relaxed them. They are recorded here rather than in ticket 11 because the map⇄terminal layout is 08's territory.

- **Split is now offered, not forbidden.** Above, map and terminal *don't split* — the map slides over the terminal — on the mechanical argument that xterm.js re-wraps on resize and a pushing split would churn the terminal. That objection is accepted but no longer judged decisive: a window resize already reflows the terminal regardless, and the split is made **terminal-priority** — the terminal holds its pixel width while the map (flex) absorbs window-resize slack — so reflow churn is confined to *deliberate* divider drags, not incidental. The **overlay remains the default**; split is an operator-toggled alternative that docks the map as a resizable column beside the terminal.
- **Ticket detail is a responsive pane, not a fixed bottom sheet.** The "bottom sheet on the map card, capped at half its height" becomes a pane that **docks to the right**, and **re-docks to the bottom (capped at 50% of the map panel) when the map panel is narrow** (< ~780px). The camera-eases-the-star-into-free-space behaviour is kept and generalised: it measures the pane's actual size (its bottom height is content-dependent up to the cap) and seats the star in whatever space the pane leaves.

Everything else in this Answer stands. The successors' inheritances are unchanged; ticket 10's hub still leads with the brief, and where it reused the bottom-sheet framing it may adopt the same responsive pane. Prototype still throwaway; this amendment is the decision.

## Amendment (2026-07-20 — operator-approved, via ticket 06/07 implementation)

Building the docked split (tickets 06–07) refined one call from the 2026-07-19 amendment above.

- **The terminal-priority split yields at the small end.** Above, docked, "the terminal holds its pixel width while the map (flex) absorbs window-resize slack." That holds while both panes fit — but on a window too narrow to seat the terminal's frozen width *and* the map, the earlier rule left the map to collapse to nothing. Corrected to prioritise the new behaviour: the docked map keeps a min-width floor (300px) and the terminal becomes shrinkable to its own floor (240px), so a shrinking window makes the **terminal** yield rather than the map vanish. The frozen-width feel is unchanged wherever both panes fit; the yield is confined to the narrow tail, and neither pane ever disappears.

Everything else in the prior amendment stands. (Not recorded here as a decision but noted for the badge's later home: the summon affordance moved from the map's edge handle to a toggle in the space header, so ticket 08's "handle carries the Next up badge" needs a new anchor when the action station lands.)
