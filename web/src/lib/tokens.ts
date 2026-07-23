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

// A single colour slot on xterm's ITheme — every key but `extendedAnsi`, which is
// a `string[]` we never drive from a pref.
type ThemeColorKey = Exclude<keyof ITheme, 'extendedAnsi'>

// SLOT_KEYS maps each per-slot pref key to the xterm ITheme slot it drives — the
// full theme surface a `terminal.toml` can override (spec: the sixteen ANSI slots
// plus background, foreground, cursor, cursorAccent, selection). All map 1:1 except
// `selection`, which xterm calls `selectionBackground`. This table is the single
// source for the explicit-override layer.
const SLOT_KEYS: ReadonlyArray<readonly [keyof TerminalPrefs, ThemeColorKey]> = [
  ['background', 'background'],
  ['foreground', 'foreground'],
  ['cursor', 'cursor'],
  ['cursorAccent', 'cursorAccent'],
  ['selection', 'selectionBackground'],
  ['black', 'black'],
  ['red', 'red'],
  ['green', 'green'],
  ['yellow', 'yellow'],
  ['blue', 'blue'],
  ['magenta', 'magenta'],
  ['cyan', 'cyan'],
  ['white', 'white'],
  ['brightBlack', 'brightBlack'],
  ['brightRed', 'brightRed'],
  ['brightGreen', 'brightGreen'],
  ['brightYellow', 'brightYellow'],
  ['brightBlue', 'brightBlue'],
  ['brightMagenta', 'brightMagenta'],
  ['brightCyan', 'brightCyan'],
  ['brightWhite', 'brightWhite'],
]

// The bundled theme presets, by the same names the server validates (config
// terminal.go's `terminalPresets` — keep the two sets in lockstep). Each is a full
// palette an operator selects with `theme.preset = "<name>"`; the resolve stacks it
// over the token-derived base and under any explicit per-slot override. These are
// data, not chrome: the terminal is an imperative island whose *content* is
// operator-themed, so a preset's own hues (a Dracula yellow, a Solarized amber) are
// legitimate here and never touch the monochrome chrome (ADR 0010, ADR 0012). No
// network — the palettes are inlined and bundled. Values are each theme's canonical
// published colours; `cursorAccent` (the glyph under a block cursor) tracks the
// background.
const PRESETS: Record<string, Partial<ITheme>> = {
  dracula: {
    background: '#282a36',
    foreground: '#f8f8f2',
    cursor: '#f8f8f2',
    cursorAccent: '#282a36',
    selectionBackground: '#44475a',
    black: '#21222c',
    red: '#ff5555',
    green: '#50fa7b',
    yellow: '#f1fa8c',
    blue: '#bd93f9',
    magenta: '#ff79c6',
    cyan: '#8be9fd',
    white: '#f8f8f2',
    brightBlack: '#6272a4',
    brightRed: '#ff6e6e',
    brightGreen: '#69ff94',
    brightYellow: '#ffffa5',
    brightBlue: '#d6acff',
    brightMagenta: '#ff92df',
    brightCyan: '#a4ffff',
    brightWhite: '#ffffff',
  },
  nord: {
    background: '#2e3440',
    foreground: '#d8dee9',
    cursor: '#d8dee9',
    cursorAccent: '#2e3440',
    selectionBackground: '#434c5e',
    black: '#3b4252',
    red: '#bf616a',
    green: '#a3be8c',
    yellow: '#ebcb8b',
    blue: '#81a1c1',
    magenta: '#b48ead',
    cyan: '#88c0d0',
    white: '#e5e9f0',
    brightBlack: '#4c566a',
    brightRed: '#bf616a',
    brightGreen: '#a3be8c',
    brightYellow: '#ebcb8b',
    brightBlue: '#81a1c1',
    brightMagenta: '#b48ead',
    brightCyan: '#8fbcbb',
    brightWhite: '#eceff4',
  },
  gruvbox: {
    background: '#282828',
    foreground: '#ebdbb2',
    cursor: '#ebdbb2',
    cursorAccent: '#282828',
    selectionBackground: '#504945',
    black: '#282828',
    red: '#cc241d',
    green: '#98971a',
    yellow: '#d79921',
    blue: '#458588',
    magenta: '#b16286',
    cyan: '#689d6a',
    white: '#a89984',
    brightBlack: '#928374',
    brightRed: '#fb4934',
    brightGreen: '#b8bb26',
    brightYellow: '#fabd2f',
    brightBlue: '#83a598',
    brightMagenta: '#d3869b',
    brightCyan: '#8ec07c',
    brightWhite: '#ebdbb2',
  },
  'solarized-dark': {
    background: '#002b36',
    foreground: '#839496',
    cursor: '#839496',
    cursorAccent: '#002b36',
    selectionBackground: '#073642',
    black: '#073642',
    red: '#dc322f',
    green: '#859900',
    yellow: '#b58900',
    blue: '#268bd2',
    magenta: '#d33682',
    cyan: '#2aa198',
    white: '#eee8d5',
    brightBlack: '#002b36',
    brightRed: '#cb4b16',
    brightGreen: '#586e75',
    brightYellow: '#657b83',
    brightBlue: '#839496',
    brightMagenta: '#6c71c4',
    brightCyan: '#93a1a1',
    brightWhite: '#fdf6e3',
  },
  'solarized-light': {
    background: '#fdf6e3',
    foreground: '#657b83',
    cursor: '#657b83',
    cursorAccent: '#fdf6e3',
    selectionBackground: '#eee8d5',
    black: '#073642',
    red: '#dc322f',
    green: '#859900',
    yellow: '#b58900',
    blue: '#268bd2',
    magenta: '#d33682',
    cyan: '#2aa198',
    white: '#eee8d5',
    brightBlack: '#002b36',
    brightRed: '#cb4b16',
    brightGreen: '#586e75',
    brightYellow: '#657b83',
    brightBlue: '#839496',
    brightMagenta: '#6c71c4',
    brightCyan: '#93a1a1',
    brightWhite: '#fdf6e3',
  },
}

/**
 * Collect the explicit per-slot colour overrides from the prefs into a partial
 * ITheme — the top layer of the theme stack. Only slots the file actually set
 * (a non-empty string) appear, so an unset slot leaves the preset / token layer
 * below it showing through.
 */
function explicitSlots(p: TerminalPrefs): Partial<ITheme> {
  const out: Partial<ITheme> = {}
  for (const [prefKey, themeKey] of SLOT_KEYS) {
    const v = p[prefKey]
    if (typeof v === 'string' && v.trim()) out[themeKey] = v.trim()
  }
  return out
}

/**
 * Resolve `TerminalPrefs` into the concrete xterm options and theme the terminal
 * island mounts with — the client half of the customization seam (Seam 2). It is
 * pure over its inputs: the prefs plus the live design tokens (read through
 * `readColor`, defaulting to the document root).
 *
 * The theme is stacked in three layers, lowest first (spec, Theme layering):
 *
 *   1. **token-derived base** — every slot the chrome has a token for resolves off
 *      the live design tokens exactly as the old `buildTheme` did, and the six ANSI
 *      hues the monochrome chrome has no token for ride `ANSI_DEFAULTS` as the
 *      default preset layer. With no preset and no overrides this is today's look,
 *      unchanged.
 *   2. **named preset** — a validated bundled preset (`p.preset`) overrides every
 *      base slot it names. The server only hands over a known name, so a miss here
 *      is simply "no preset".
 *   3. **explicit slots** — any colour slot the file set wins over both.
 *
 * An unset font or size falls to the bundled default; a pref colour is a concrete
 * `#hex` (validated server-side) used verbatim.
 */
export function buildTerminalOptions(
  prefs: TerminalPrefs | undefined,
  el: Element = document.documentElement,
): { options: ITerminalOptions; theme: ITheme } {
  const p = prefs ?? {}

  const background = readColor('--background', el)
  const foreground = readColor('--foreground', el)
  const dim = readColor('--muted-foreground', el)
  const red = readColor('--destructive', el)

  // Layer 1 — token-derived base (with ANSI_DEFAULTS as the default preset layer).
  const base: ITheme = {
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

  // Layer 2 — named preset. Layer 3 — explicit per-slot overrides.
  const preset = (p.preset && PRESETS[p.preset]) || undefined
  const overrides = explicitSlots(p)

  const theme: ITheme = { ...base, ...preset, ...overrides }

  const family = p.fontFamily?.trim()
  const options: ITerminalOptions = {
    fontFamily: family ? `${family}, ${DEFAULT_FONT_STACK}` : DEFAULT_FONT_STACK,
    fontSize: p.fontSize && p.fontSize > 0 ? p.fontSize : DEFAULT_FONT_SIZE,
    theme,
  }

  return { options, theme }
}
