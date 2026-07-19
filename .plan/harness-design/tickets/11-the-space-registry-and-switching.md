---
type: prototype
blocked_by: [08]
claimed_by: claude-opus-4-8
claimed_at: 2026-07-19T09:15:07Z
---

# The space registry and switching

## Question

A space is a git repository, registered once and switched between (ADR 0003). Design the registry and the switching.

- **Registering.** Point the harness at a path — then what? Must it already be a git repository, and must it already have a `.plan/`? A repo with no maps is legal; ticket 07 charts one. What does the harness write, and *where*: this is the first harness-owned state that is not a space's own committed config, so it needs a home and a story for what happens when it is lost.
- **Discovery.** wayfinder-maps' picker already lists every map under a project's `.plan/`, and that code is liftable. The harness needs it plus a layer above: many repos, each with many maps.
- **Switching.** What switching means when the space you left has a session still cooking. The switcher has to carry state — *this one is running, that one wants you* — which starts to overlap with the attention question the fog is holding. Decide whether the switcher is where attention lives, or whether attention deserves its own surface.
- **Ordering and scale.** Most-recent, manual, favourites? Is the realistic number of spaces five or fifty? The answer changes the design completely.
- **Removal.** De-registering a space that has live work, and whether the harness ever touches the repository on the way out. It should probably touch nothing — but say so deliberately.

Link the prototype as an asset.
