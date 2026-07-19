---
type: task
blocked_by: [03]
---

# The star-map card

## Question

The star-map renderer, reimplemented cleanly in TypeScript behind the narrow island seam — mount, receive the pushed model, emit selection — never decomposed into components (ADR 0010). Deterministic layout seeded from ticket data, the six base states, camera easing, with tuned constants (easing, zoom coupling, parallax, dpr) cribbed from the wayfinder-maps renderer against feel-drift; `starmap-design.md` and the planning map's prototype assets are the open references. The map is summoned as a floating card over the terminal — edge handle, `M`, Esc — never toggled by switching spaces or maps; the operator can instead dock it as the terminal-priority split (the terminal holds its pixel width; the map absorbs resize slack — planning ticket 08's amendment). An all-open map renders as-is.

Done when: fixture maps render with the six states; island-seam tests prove determinism (same data, same positions) and that model pushes change overlays but never move a star; summon/dismiss works by handle, key, and Esc; the split toggle docks the map without reflowing the terminal on window resize; clicking a star emits selection.
