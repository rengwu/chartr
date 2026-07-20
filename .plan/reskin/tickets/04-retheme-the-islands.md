---
type: task
blocked_by: [01]
---

# Re-theme the islands — xterm and the star-map from the shared tokens

## Question

Point the two imperative islands at the design-system tokens through the seam, so the whole app reads as one system — **without** reaching inside either renderer (ADR 0010). Use the `tokens.ts` bridge from ticket 01 at the mount/model seam only.

**xterm (`Terminal.svelte`):** build the xterm `ITheme` (background, foreground, cursor, selection, and the 16 ANSI slots) from the theme tokens — `--card`/`--background` for the surface, `--foreground` for text, `--muted-foreground` for dim, `--destructive` for red — so the terminal surface matches the reskinned tab strip instead of xterm's defaults. The mapping is computed in the wrapper and handed to the island; the island's internals don't change.

**star-map (`starmap/theme.ts`):** the six status hues are **kept** as exempt categorical data-viz colour (map decision), but re-tuned so they sit legibly on the theme's warm near-black `--card`, and the card's own background (currently the hard-coded `#05070d`) is derived from the token surface rather than a magic constant. Reconcile against ADR 0010's feel-drift risk: this is a palette re-tune at the seam, not a renderer change — the star sizes/glows/pulse and the tuned camera constants are untouched, and the star-map's determinism and zero-movement guarantees (its seam tests) must still hold. Amber remains only as the `claimed` star. If a status hue needs a token, add a named data-viz token (e.g. `--chart-*` or a small `--status-*` set) rather than sinking a hex into the renderer.

Done when: the terminal renders on a token-derived `ITheme` that visually agrees with the reskinned chrome; the star-map's six states render legibly on the token-derived card surface with the categorical hues intact and amber only on `claimed`; the star-map island-seam tests (determinism, zero-movement on status pushes, selection) still pass unchanged; and `svelte-check`, the Vite build, `vitest`, `go vet`, and `go test` are green.