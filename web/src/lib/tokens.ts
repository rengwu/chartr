// Token bridge — the seam between the design-system CSS tokens and the two
// imperative islands (xterm's ITheme, the star-map palette). The chrome owns
// the tokens as CSS custom properties; the islands are canvas/renderer code
// that needs concrete colour strings. This helper is the one place that reads
// resolved token values off the live document so the islands stay in sync with
// the theme without reaching inside their renderers (ADR 0010).
//
// Ticket 01 ships the scaffold and its test only; ticket 04 wires the islands
// to it.

/**
 * Read a CSS custom property's declared value off an element (the document root
 * by default). Returns the trimmed value exactly as the cascade resolves it —
 * e.g. `oklch(0.228 0.013 107.4)` or `#1e2530`. Empty string if unset.
 */
export function readToken(name: string, el: Element = document.documentElement): string {
  return getComputedStyle(el).getPropertyValue(name).trim()
}

/**
 * Resolve any CSS colour string (oklch, hsl, hex, named, or a `var(--token)`
 * reference) to a concrete `rgb(...)` / `rgba(...)` string by letting the
 * browser compute it through a throwaway probe element. This is how oklch
 * theme tokens become the hex/rgb that xterm's ITheme and the star-map palette
 * require. In a headless jsdom test the browser does no colour maths, so pass
 * values the environment already understands (hex/rgb) when asserting.
 */
export function resolveColor(value: string, host: Element = document.body): string {
  if (!value) return value
  const probe = document.createElement('span')
  probe.style.color = value
  probe.style.display = 'none'
  host.appendChild(probe)
  const resolved = getComputedStyle(probe).color
  probe.remove()
  return resolved || value
}

/**
 * Read a token and resolve it to a concrete colour string in one step — the
 * common case for the islands, which want `--card` etc. as usable colour.
 */
export function readColor(name: string, el: Element = document.documentElement): string {
  return resolveColor(readToken(name, el), el.ownerDocument?.body ?? document.body)
}

/**
 * Read several tokens at once, keyed by a caller-friendly name. Convenience for
 * assembling a full palette (e.g. xterm's 16 ANSI slots) in a single pass.
 */
export function readTokens<K extends string>(
  map: Record<K, string>,
  el: Element = document.documentElement,
): Record<K, string> {
  const out = {} as Record<K, string>
  for (const key in map) {
    out[key] = readToken(map[key], el)
  }
  return out
}
