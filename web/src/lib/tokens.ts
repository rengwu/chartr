import type { ITheme, ITerminalOptions } from '@xterm/xterm'
import type { TerminalPrefs } from './model'

// Token bridge — the seam between the design-system CSS tokens and the two
// imperative islands (xterm's ITheme, the star-map palette). The chrome owns
// the tokens as CSS custom properties; the islands are canvas/renderer code
// that needs concrete colour strings. This helper is the one place that reads
// resolved token values off the live document so the islands stay in sync with
// the theme without reaching inside their renderers (ADR 0010).
//
// It is also the client resolve seam (Seam 2) of terminal customization: the
// pure `buildTerminalOptions` below turns the `TerminalPrefs` off the model
// snapshot into the concrete xterm options object and ITheme the terminal island
// consumes, resolving every unset colour slot against the live design tokens
// exactly as the island's old hard-coded `buildTheme` did. The island stays
// imperative and never re-themes itself — it is handed a resolved object (ADR
// 0010).

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

// The bundled default font stack. The terminal renders IBM Plex Mono (self-hosted,
// no CDN — CLAUDE.md), falling back through the platform monospace faces so a
// machine mid-font-load still shows fixed-width text. A pref-set family stacks in
// front of this, so a non-bundled family the operator names degrades to the same
// floor rather than to a proportional font.
const DEFAULT_FONT_STACK =
  "'IBM Plex Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace"

// The default cell size, unchanged from the island's old literal. A pref overrides
// it; an unset (zero) size falls here.
const DEFAULT_FONT_SIZE = 13

// The six ANSI hues the monochrome chrome has no token for — green, yellow, blue,
// magenta, cyan — as muted values tuned to sit quietly on the token surface. They
// were literals inside the island's `buildTheme`; the resolve seam owns them now
// as the default preset layer (spec, Theme layering), and a later ticket lets a
// named preset or per-slot override replace them. Background, foreground, cursor,
// selection and red still resolve off live tokens so the terminal tracks the theme.
const ANSI_DEFAULTS = {
  green: '#9cb68c',
  brightGreen: '#b3cba3',
  yellow: '#d1b374',
  brightYellow: '#e0c88f',
  blue: '#82a8c9',
  brightBlue: '#9dbdd9',
  magenta: '#b48cc2',
  brightMagenta: '#c7a5d3',
  cyan: '#7fb3ab',
  brightCyan: '#99c7c0',
} as const

/**
 * Resolve `TerminalPrefs` into the concrete xterm options and theme the terminal
 * island mounts with — the client half of the customization seam (Seam 2). It is
 * pure over its inputs: the prefs plus the live design tokens (read through
 * `readColor`, defaulting to the document root). An unset colour slot falls
 * through to its token-derived default exactly as the old `buildTheme` resolved
 * it, so a partial `terminal.toml` still composes with the reskin; an unset font
 * or size falls to the bundled default. A pref colour is a concrete `#hex`
 * (validated server-side) and is used verbatim.
 */
export function buildTerminalOptions(
  prefs: TerminalPrefs | undefined,
  el: Element = document.documentElement,
): { options: ITerminalOptions; theme: ITheme } {
  const p = prefs ?? {}

  const background = p.background || readColor('--background', el)
  const foreground = p.foreground || readColor('--foreground', el)
  const dim = readColor('--muted-foreground', el)
  const red = readColor('--destructive', el)

  const theme: ITheme = {
    background,
    foreground,
    cursor: readColor('--ring', el),
    cursorAccent: background,
    selectionBackground: readColor('--muted', el),
    black: background,
    brightBlack: dim,
    white: foreground,
    brightWhite: foreground,
    red,
    brightRed: red,
    ...ANSI_DEFAULTS,
  }

  const family = p.fontFamily?.trim()
  const options: ITerminalOptions = {
    fontFamily: family ? `${family}, ${DEFAULT_FONT_STACK}` : DEFAULT_FONT_STACK,
    fontSize: p.fontSize && p.fontSize > 0 ? p.fontSize : DEFAULT_FONT_SIZE,
    theme,
  }

  return { options, theme }
}
