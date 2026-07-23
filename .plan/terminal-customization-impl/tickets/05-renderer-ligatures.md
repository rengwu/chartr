---
type: task
blocked_by: [01]
---

# Renderer + ligatures

## Question

An operator gets a GPU-accelerated terminal by default that never goes blank, and
can opt into ligatures — accepting that ligatures switch that terminal off the GPU
renderer and only work for a bundled font. These are one code path: the
renderer-selection branch.

- **Renderer:** the WebGL renderer is the default, lazily imported and instantiated
  at mount. Wire the GPU context-loss event to fall back to the DOM renderer so a
  backgrounded tab or driver reset never leaves the terminal blank.
- **Ligatures:** when the ligatures pref is on, that terminal is forced onto the
  canvas renderer (WebGL off) — the ligatures addon and the WebGL renderer cannot
  coexist. The ligatures addon is lazily imported and pointed at the embedded
  bundled font asset; it never fetches a font over the network, and it applies
  only when the resolved font family is a bundled one.

All addons are bundled. Because the island remounts on any prefs change, the
renderer/ligatures choice is made fresh at each mount from the current prefs —
there is no hot-swap.

Tests lead: the Seam 2 table test asserts the renderer/ligatures decision — WebGL
by default; `ligatures on` selects canvas; ligatures suppressed for a non-bundled
family. The actual GPU mount and context-loss fallback are trusted at runtime, not
unit-tested.

Done when: terminals render on WebGL by default and survive a context loss via the
DOM fallback; enabling ligatures forces canvas and renders ligatures for a bundled
font; no network fetch occurs; the decision test is green; frontend + Go checks
pass.
