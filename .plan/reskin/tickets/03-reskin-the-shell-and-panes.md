---
type: task
blocked_by: [01]
---

# Reskin the shell and panes — sidebar, stage, tabs, map card

## Question

Rebuild the shell — the surfaces that own the cockpit layout — onto tokens and primitives, retiring the rest of the bespoke `app.css`. In scope: `App.svelte` (the `.cockpit` grid, `.sidebar` with its space/map rows and filter, the `.statusbar`); `SpacePane.svelte` (the `.space-header` identity bar, the `.space-panes` row, the `.map-toggle`, the `.term-*` tab strip and bindings drawer → shadcn **Tabs** + **Sheet**); `MapCard.svelte` (the `.map-card*` chrome, float/dock, the left-border resize grip); `StarMap.svelte` (the island wrapper's card frame only — **not** the canvas). The sidebar adopts the theme's `--sidebar-*` tokens.

This is where de-ambering lands across the shell: `.map-toggle[aria-pressed]`, `.term-config`, `.map-card-btn`, `.space-row.active`, the pinned-pin highlight, the amber hover borders — all move to `--primary` / `--ring` / `--accent` (the subtle neutral), keeping only `--destructive` for forget/close-danger affordances. Emoji glyphs become Phosphor icons: 📌 pin, ＋ add, × close/forget, ▲ agent-missing badge, ⚠ malformation, ✓ finished, the map-toggle glyph. Apply distill: the space-row's pin/forget/classify controls fold into the shared icon-Button with hover-reveal preserved; the duplicated bar styles (`.term-bar`, `.map-card-bar`, `.space-header` all share `--bar-h`) collapse into one bar primitive/util.

Behaviour is fixed (map "Out of scope"): the `--bar-h` one-tier alignment of the three chrome bars, the float/dock split with the terminal's frozen width and no reflow on window resize, the drawer summon, the `M`/`Esc` bindings, and the hover-reveal row affordances all survive unchanged. The xterm and star-map canvases are untouched here — only their surrounding chrome.

Done when: the shell, sidebar, space header, terminal tab strip + bindings drawer, and map-card chrome all run on tokens + primitives + Phosphor icons with the remaining bespoke `app.css` chrome deleted; no amber survives in the chrome; the three bars still align on one tier; float/dock, resize-without-reflow, the drawer, and the keyboard bindings all behave as before; `svelte-check`, the Vite build, `vitest`, `go vet`, and `go test` pass.