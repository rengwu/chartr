import type { ITheme, ITerminalOptions } from '@xterm/xterm'
import type { ISearchOptions } from '@xterm/addon-search'
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

// The curated set of fonts the app actually ships (self-hosted, no CDN — CLAUDE.md),
// keyed by the lower-case name an operator writes in `font.family`. A bundled name
// resolves to a clean stack that is guaranteed to render offline; a family that is
// not here is the operator's own (it depends on their OS) and stacks ahead of the
// system fallback instead. Only IBM Plex Mono ships today; the ligatures ticket,
// which needs a bundled font to point its addon at, is what grows this list.
const BUNDLED_FONTS: Record<string, string> = {
  'ibm plex mono': DEFAULT_FONT_STACK,
}

// The default cell size, unchanged from the island's old literal. A pref overrides
// it; an unset (zero) size falls here.
const DEFAULT_FONT_SIZE = 13

// Resolve the operator's chosen family into a CSS font-family string. A bundled
// name (case-insensitive) resolves to its guaranteed-to-render stack; any other
// family is a custom string that stacks ahead of the system fallback, so a font the
// operator's OS lacks degrades to fixed-width text rather than a proportional face.
// Empty is unset — the bundled default stands.
function resolveFontFamily(family: string | undefined): string {
  const name = family?.trim()
  if (!name) return DEFAULT_FONT_STACK
  const bundled = BUNDLED_FONTS[name.toLowerCase()]
  if (bundled) return bundled
  return `${name}, ${DEFAULT_FONT_STACK}`
}

// Whether the operator's chosen family is one the app bundles. An unset family is
// bundled — it falls through to the default (IBM Plex Mono), which ships with the
// binary. This gates ligatures: the addon reads its ligature data from the font,
// so a family the app does not bundle has no asset to point it at.
function isBundledFont(family: string | undefined): boolean {
  const name = family?.trim()
  if (!name) return true
  return name.toLowerCase() in BUNDLED_FONTS
}

// The terminal's renderer, and whether ligatures are active on it — the pure
// decision the island mounts from (Seam 2). It is one code path because the two are
// coupled: the WebGL renderer is the default, but the ligatures addon and WebGL
// cannot coexist, so switching ligatures on is exactly what forces the terminal onto
// the canvas renderer.
export type TerminalRenderer = 'webgl' | 'canvas'

export interface RendererChoice {
  // 'webgl' is the GPU default; 'canvas' is what ligatures force. The DOM renderer
  // is not chosen here — it is the runtime fallback the island drops to on a WebGL
  // context loss, off this same 'webgl' choice.
  renderer: TerminalRenderer
  // Whether to load the ligatures addon at mount. True only when the pref is on
  // *and* the resolved font is bundled; a non-bundled family suppresses it (and so
  // leaves the terminal on WebGL).
  ligatures: boolean
}

/**
 * Resolve `TerminalPrefs` into the renderer/ligatures choice the terminal island
 * mounts with. Pure over the prefs alone (no tokens, no DOM):
 *
 *   - **default** — WebGL, no ligatures. This is the zero-config terminal.
 *   - **ligatures on, bundled font** — canvas + ligatures. The addon and WebGL
 *     cannot coexist, so enabling ligatures forces the canvas renderer.
 *   - **ligatures on, non-bundled font** — WebGL, no ligatures. Ligatures need a
 *     bundled font to read their data from, so they are suppressed and the terminal
 *     stays on the GPU renderer as if they were off.
 *
 * Because the island fully remounts on any prefs change, this is decided once per
 * mount from the current prefs — there is no hot-swap between renderers.
 */
export function resolveRenderer(prefs: TerminalPrefs | undefined): RendererChoice {
  const p = prefs ?? {}
  const ligatures = p.ligatures === true && isBundledFont(p.fontFamily)
  return { renderer: ligatures ? 'canvas' : 'webgl', ligatures }
}

// A font weight as xterm types it — a keyword or a number. `terminal.toml` carries
// the weight as a normalised string (server-side); this maps it onto the option,
// passing a keyword straight through and a numeric string ('600') as a number.
type FontWeight = NonNullable<ITerminalOptions['fontWeight']>

function resolveWeight(weight: string | undefined): FontWeight | undefined {
  const w = weight?.trim()
  if (!w) return undefined
  if (w === 'normal' || w === 'bold') return w
  const n = Number(w)
  return Number.isFinite(n) ? (n as FontWeight) : undefined
}

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

// The CSS custom properties the resolve hands the island for the settings xterm
// has no option for — the scrollbar's width and colours, and the padding around
// the grid. Keyed by the full property name (`--terminal-padding-top`) so the
// island's only job is to set each one on its host.
export type TerminalCss = Record<string, string>

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
 *
 * The third return, `css`, is the scrollbar and padding half of the resolve: xterm
 * has no options for either, so they leave here as CSS custom properties the island
 * sets on its own host element and `app.css` consumes (still the seam — the chrome
 * styles its wrapper, never the renderer inside it; ADR 0010).
 */
export function buildTerminalOptions(
  prefs: TerminalPrefs | undefined,
  el: Element = document.documentElement,
): { options: ITerminalOptions; theme: ITheme; css: TerminalCss } {
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

  const options: ITerminalOptions = {
    fontFamily: resolveFontFamily(p.fontFamily),
    fontSize: p.fontSize && p.fontSize > 0 ? p.fontSize : DEFAULT_FONT_SIZE,
    theme,
  }

  // Every remaining pref maps 1:1 onto an xterm option, and is set only when the
  // file actually carried it — an unset value leaves xterm's own default in place,
  // so a partial file still behaves like today. The server has already validated
  // each value (enum, range, sign), so the resolve is a straight pass-through.
  setOpt(options, 'fontWeight', resolveWeight(p.fontWeight))
  setOpt(options, 'fontWeightBold', resolveWeight(p.fontWeightBold))
  setOpt(options, 'lineHeight', positive(p.lineHeight))
  setOpt(options, 'letterSpacing', p.letterSpacing)

  setOpt(options, 'cursorStyle', enumOpt(p.cursorStyle, ['block', 'bar', 'underline']))
  setOpt(options, 'cursorBlink', typeof p.cursorBlink === 'boolean' ? p.cursorBlink : undefined)
  setOpt(
    options,
    'cursorInactiveStyle',
    enumOpt(p.cursorInactiveStyle, ['outline', 'block', 'bar', 'underline', 'none']),
  )
  setOpt(options, 'cursorWidth', positive(p.cursorWidth))

  setOpt(options, 'scrollback', positive(p.scrollback))
  setOpt(options, 'scrollSensitivity', positive(p.scrollSensitivity))
  setOpt(options, 'fastScrollModifier', enumOpt(p.fastScrollModifier, ['alt', 'ctrl', 'shift', 'none']))
  setOpt(options, 'fastScrollSensitivity', positive(p.fastScrollSensitivity))
  setOpt(options, 'smoothScrollDuration', positive(p.smoothScrollDuration))

  setOpt(options, 'minimumContrastRatio', inRange(p.minimumContrastRatio, 1, 21))

  // The two selection/key behaviours xterm *does* own. Copy-on-select is not among
  // them — xterm has no such option, so the island wires it to `onSelectionChange`
  // off the pref directly, and Shift+Enter is the pure predicate below.
  setOpt(options, 'rightClickSelectsWord', p.rightClickSelectsWord)
  setOpt(options, 'macOptionIsMeta', p.macOptionIsMeta)

  return { options, theme, css: buildTerminalCss(p, theme) }
}

/**
 * The CSS custom properties the island sets on its host — the scrollbar and the
 * padding, neither of which xterm exposes an option for. Only a property the file
 * actually set appears: every rule in `app.css` reads these through a `var(…, …)`
 * fallback, so an unset pref leaves the chrome's own scrollbar styling and the
 * flush-to-the-edge grid exactly as they are today.
 *
 * Auto-hide rides `--terminal-scrollbar-thumb-idle`: it is the thumb's colour at
 * rest, so setting it transparent hides the thumb until `.terminal-island:hover`
 * paints it back with the real thumb colour.
 */
function buildTerminalCss(p: TerminalPrefs, theme: ITheme): TerminalCss {
  const css: TerminalCss = {}

  const width = positive(p.scrollbarWidth)
  if (width !== undefined) css['--terminal-scrollbar-width'] = `${width}px`
  if (p.scrollbarThumb?.trim()) css['--terminal-scrollbar-thumb'] = p.scrollbarThumb.trim()
  if (p.scrollbarTrack?.trim()) css['--terminal-scrollbar-track'] = p.scrollbarTrack.trim()
  if (p.scrollbarAutoHide) css['--terminal-scrollbar-thumb-idle'] = 'transparent'

  let padded = false
  for (const [prefKey, side] of PADDING_SIDES) {
    const v = p[prefKey]
    // Zero is unset *and* the default (flush to the edge), so it need not be emitted.
    if (typeof v === 'number' && v > 0) {
      css[`--terminal-padding-${side}`] = `${v}px`
      padded = true
    }
  }
  // The padded frame is outside the grid xterm paints, so it is the chrome that has
  // to fill it — with the terminal's own resolved background, or a Dracula terminal
  // would sit in a token-coloured surround. Only a padded island has a frame at all.
  if (padded && theme.background) css['--terminal-background'] = theme.background

  return css
}

// The padding prefs and the custom-property suffix each drives.
const PADDING_SIDES: ReadonlyArray<readonly [keyof TerminalPrefs, string]> = [
  ['paddingTop', 'top'],
  ['paddingRight', 'right'],
  ['paddingBottom', 'bottom'],
  ['paddingLeft', 'left'],
]

/**
 * The colours the in-terminal find widget paints its match decorations with,
 * resolved from the design tokens at the seam (Seam 2's colour bridge) — never
 * inlined into the search addon as raw hex (no raw colour in the chrome; ADR 0012).
 * The active match rides `--primary`, the emphasis token, so the current hit stands
 * out from the muted highlight every other match carries; borders and the overview
 * ruler track the same two tones. Concrete colours are read off the live document so
 * the highlight tracks the current theme exactly as the ITheme resolve does (ADR
 * 0010). The search addon owns the highlight overlay; this only hands it colour.
 */
export function terminalSearchDecorations(
  el: Element = document.documentElement,
): NonNullable<ISearchOptions['decorations']> {
  const primary = readColor('--primary', el)
  const ring = readColor('--ring', el)
  const muted = readColor('--muted-foreground', el)
  return {
    matchBackground: readColor('--muted', el),
    matchBorder: muted,
    matchOverviewRuler: muted,
    activeMatchBackground: primary,
    activeMatchBorder: ring,
    activeMatchColorOverviewRuler: primary,
  }
}

/**
 * What a key event means to the terminal — the pure `event → action` half of the
 * keybindings pref, decided here at the seam so the island only has to obey it.
 *
 * - `newline` — Shift+Enter with the pref on: the shell gets a literal newline
 *   (`TERMINAL_NEWLINE`) instead of submitting, the Ghostty behaviour every agent
 *   CLI wants for composing multi-line input (story 14).
 * - `submit` — an Enter we do not intercept, including Shift+Enter with the pref
 *   off: xterm's own handling stands and the line goes.
 * - `default` — everything else, including an Enter carrying another modifier
 *   (Ctrl/Alt/Meta already mean something to the shell and are never intercepted).
 *
 * `shiftEnterNewline` is unset-means-*on*; `false` is what restores plain submit.
 * Only a keydown acts — the matching keyup must stay `default` or the newline
 * would be written twice.
 */
export type TerminalKeyAction = 'newline' | 'submit' | 'default'

// What a `newline` action writes to the shell: a literal LF, the same bytes
// Ghostty's `shift+enter=text:\n` sends.
export const TERMINAL_NEWLINE = '\n'

// The slice of a KeyboardEvent the decision actually reads, so the predicate is
// callable from a test with a plain object and never needs a real event.
export type TerminalKeyEvent = Pick<
  KeyboardEvent,
  'type' | 'key' | 'shiftKey' | 'ctrlKey' | 'altKey' | 'metaKey'
>

export function terminalKeyAction(
  ev: TerminalKeyEvent,
  prefs: TerminalPrefs | undefined = undefined,
): TerminalKeyAction {
  if (ev.type !== 'keydown' || ev.key !== 'Enter') return 'default'
  if (ev.ctrlKey || ev.altKey || ev.metaKey) return 'default'
  if (ev.shiftKey && prefs?.shiftEnterNewline !== false) return 'newline'
  return 'submit'
}

// setOpt assigns an xterm option only when the resolved value is defined, so an
// unset pref never overwrites xterm's own default with `undefined`. Typed to the
// option key so a wrong value type is a compile error.
function setOpt<K extends keyof ITerminalOptions>(
  options: ITerminalOptions,
  key: K,
  value: ITerminalOptions[K] | undefined,
): void {
  if (value !== undefined) options[key] = value
}

// The numeric guards mirror the server's validation so the resolve is defensive on
// its own — an unset (zero / undefined) value returns undefined and the option is
// left at xterm's default. `positive` wants > 0 (a zero scrollback or duration means
// "the default"), and `inRange` treats the low bound as the unset value.
function positive(n: number | undefined): number | undefined {
  return typeof n === 'number' && n > 0 ? n : undefined
}
function inRange(n: number | undefined, lo: number, hi: number): number | undefined {
  return typeof n === 'number' && n > lo && n <= hi ? n : undefined
}
function enumOpt<T extends string>(value: string | undefined, allowed: readonly T[]): T | undefined {
  const v = value?.trim()
  return v && (allowed as readonly string[]).includes(v) ? (v as T) : undefined
}
