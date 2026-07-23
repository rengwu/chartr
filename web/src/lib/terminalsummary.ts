import type { TerminalPrefs } from './model'
import { buildTerminalOptions, resolveRenderer } from './tokens'

// The read half of the Settings surface's Terminal section (ticket 08): the
// operator's *effective* terminal settings, grouped for display. It is a pure
// formatter over the same resolve the island mounts with — `buildTerminalOptions`
// and `resolveRenderer` (Seam 2) — so what the surface shows can never drift from
// what the terminal actually does. Nothing here writes: editing happens in
// `terminal.toml` in the operator's own editor (spec, Storage & ownership).
//
// Every row carries `set`, which is whether `terminal.toml` named this one. An
// unset row still shows a value — the default that is genuinely in force — so the
// section answers "what is in effect", not merely "what did I write down".

export interface TerminalSettingRow {
  label: string
  // The value in force, already formatted for display ('13 px', 'on', 'block').
  value: string
  // A concrete colour to paint a swatch with, on the rows that are colours. It is
  // the resolved colour (token-derived when the file left the slot unset), which
  // is why the swatch is honest even for a default row.
  swatch?: string
  // Whether `terminal.toml` set this value, as opposed to a default showing
  // through. The section renders the two differently; it is the whole legibility
  // story of a read-only surface over a file the operator owns.
  set: boolean
}

export interface TerminalSettingGroup {
  title: string
  rows: TerminalSettingRow[]
}

// xterm's own documented defaults for the options the resolve deliberately leaves
// alone when a pref is unset (`setOpt` never writes `undefined`). They are stated
// here only so an unset row can show the value actually in force; the resolve
// itself still passes nothing, so xterm remains the authority.
const XTERM_DEFAULTS = {
  fontWeight: 'normal',
  fontWeightBold: 'bold',
  lineHeight: '1',
  letterSpacing: '0 px',
  cursorStyle: 'block',
  cursorInactiveStyle: 'outline',
  cursorWidth: '1 px',
  scrollback: '1000 lines',
  scrollSensitivity: '1',
  fastScrollModifier: 'alt',
  fastScrollSensitivity: '5',
} as const

// Formatters for the four shapes a pref takes on the wire. Each takes the pref
// value and the default to show when it is unset, and answers both halves of a
// row at once — the string to show and whether the file set it.
function num(v: number | undefined, fallback: string, unit = ''): [string, boolean] {
  if (typeof v === 'number' && v > 0) return [unit ? `${v} ${unit}` : `${v}`, true]
  return [fallback, false]
}

function text(v: string | undefined, fallback: string): [string, boolean] {
  const s = v?.trim()
  return s ? [s, true] : [fallback, false]
}

// A tri-state: nil is unset and `whenUnset` says which way that falls. Shift+Enter
// is the one pref whose unset default is *on* (spec, story 14), which is exactly
// what this argument exists for.
function flag(v: boolean | undefined, whenUnset = false): [string, boolean] {
  if (typeof v === 'boolean') return [v ? 'on' : 'off', true]
  return [whenUnset ? 'on' : 'off', false]
}

function row(label: string, [value, set]: [string, boolean], swatch?: string): TerminalSettingRow {
  return swatch ? { label, value, swatch, set } : { label, value, set }
}

// Letter spacing is legitimately negative (tighter cells), so it cannot ride
// `num`'s positive test; zero is both unset and the default.
function signed(v: number | undefined, fallback: string, unit: string): [string, boolean] {
  return typeof v === 'number' && v !== 0 ? [`${v} ${unit}`, true] : [fallback, false]
}

function ratio(v: number | undefined): [string, boolean] {
  return typeof v === 'number' && v > 1 ? [`${v}:1`, true] : ['off', false]
}

// The five base theme slots and the sixteen ANSI ones, each paired with the
// resolved-theme key it reads its swatch from. `selection` is xterm's
// `selectionBackground`, the one slot whose names differ.
const BASE_SLOTS: ReadonlyArray<readonly [keyof TerminalPrefs, string, string]> = [
  ['background', 'background', 'background'],
  ['foreground', 'foreground', 'foreground'],
  ['cursor', 'cursor', 'cursor'],
  ['cursorAccent', 'cursor accent', 'cursorAccent'],
  ['selection', 'selection', 'selectionBackground'],
]

const ANSI_SLOTS: ReadonlyArray<readonly [keyof TerminalPrefs, string]> = [
  ['black', 'black'],
  ['red', 'red'],
  ['green', 'green'],
  ['yellow', 'yellow'],
  ['blue', 'blue'],
  ['magenta', 'magenta'],
  ['cyan', 'cyan'],
  ['white', 'white'],
  ['brightBlack', 'bright black'],
  ['brightRed', 'bright red'],
  ['brightGreen', 'bright green'],
  ['brightYellow', 'bright yellow'],
  ['brightBlue', 'bright blue'],
  ['brightMagenta', 'bright magenta'],
  ['brightCyan', 'bright cyan'],
  ['brightWhite', 'bright white'],
]

/**
 * The operator's effective terminal settings, grouped for the Settings surface.
 * Pure over the prefs plus the live design tokens (through the resolve seam), so
 * a colour slot the file left unset still shows the concrete colour the terminal
 * paints with — the token-derived one — rather than a blank.
 *
 * `el` is the element the tokens are read from, defaulting to the document root,
 * exactly as `buildTerminalOptions` takes it.
 */
export function terminalSettingsSummary(
  prefs: TerminalPrefs | undefined,
  el: Element = document.documentElement,
): TerminalSettingGroup[] {
  const p = prefs ?? {}
  const { options, theme } = buildTerminalOptions(p, el)
  const { renderer, ligatures } = resolveRenderer(p)
  const colour = (key: string) => (theme as Record<string, string | undefined>)[key] ?? ''

  return [
    {
      title: 'Font',
      rows: [
        row('family', text(p.fontFamily, 'IBM Plex Mono (bundled)')),
        row('size', [`${options.fontSize} px`, Boolean(p.fontSize && p.fontSize > 0)]),
        row('weight', text(p.fontWeight, XTERM_DEFAULTS.fontWeight)),
        row('bold weight', text(p.fontWeightBold, XTERM_DEFAULTS.fontWeightBold)),
        row('line height', num(p.lineHeight, XTERM_DEFAULTS.lineHeight)),
        row('letter spacing', signed(p.letterSpacing, XTERM_DEFAULTS.letterSpacing, 'px')),
      ],
    },
    {
      title: 'Rendering',
      rows: [
        // The renderer is a decision, not a pref: it follows from ligatures, which is
        // why it is never marked as set. The island drops to the DOM renderer only on
        // a live WebGL context loss, which no read of the file can predict.
        row('renderer', [renderer === 'canvas' ? 'canvas' : 'GPU (WebGL)', false]),
        row('ligatures', [
          ligatures ? 'on' : p.ligatures === true ? 'off — needs a bundled font' : 'off',
          typeof p.ligatures === 'boolean',
        ]),
        row('wide-glyph widths', flag(p.unicode11)),
        // A ratio of 1 is the unset default and does nothing, so it reads as 'off'
        // rather than as a number that implies a contrast floor is in force.
        row('minimum contrast', ratio(p.minimumContrastRatio)),
      ],
    },
    {
      title: 'Theme',
      rows: [
        row('preset', text(p.preset, 'the app theme')),
        ...BASE_SLOTS.map(([prefKey, label, themeKey]) =>
          row(label, text(p[prefKey] as string | undefined, 'from the theme'), colour(themeKey)),
        ),
      ],
    },
    {
      title: 'ANSI palette',
      rows: ANSI_SLOTS.map(([prefKey, label]) =>
        row(label, text(p[prefKey] as string | undefined, 'default'), colour(prefKey as string)),
      ),
    },
    {
      title: 'Cursor',
      rows: [
        row('style', text(p.cursorStyle, XTERM_DEFAULTS.cursorStyle)),
        // Unset blink is the island's own behaviour, not an xterm default: it blinks
        // while the session is alive and stops when it dies.
        row('blink', [
          typeof p.cursorBlink === 'boolean' ?
            p.cursorBlink ? 'on'
            : 'off'
          : 'while the shell is alive',
          typeof p.cursorBlink === 'boolean',
        ]),
        row('when unfocused', text(p.cursorInactiveStyle, XTERM_DEFAULTS.cursorInactiveStyle)),
        row('bar width', num(p.cursorWidth, XTERM_DEFAULTS.cursorWidth, 'px')),
      ],
    },
    {
      title: 'Scrolling',
      rows: [
        row('scrollback', num(p.scrollback, XTERM_DEFAULTS.scrollback, 'lines')),
        row('sensitivity', num(p.scrollSensitivity, XTERM_DEFAULTS.scrollSensitivity)),
        row('fast-scroll key', text(p.fastScrollModifier, XTERM_DEFAULTS.fastScrollModifier)),
        row('fast sensitivity', num(p.fastScrollSensitivity, XTERM_DEFAULTS.fastScrollSensitivity)),
        row('smooth scroll', num(p.smoothScrollDuration, 'off', 'ms')),
      ],
    },
    {
      title: 'Scrollbar & padding',
      rows: [
        row('scrollbar width', num(p.scrollbarWidth, 'the chrome default', 'px')),
        row('thumb', text(p.scrollbarThumb, 'the chrome default'), p.scrollbarThumb?.trim()),
        row('track', text(p.scrollbarTrack, 'the chrome default'), p.scrollbarTrack?.trim()),
        row('auto-hide', flag(p.scrollbarAutoHide)),
        // Zero padding is unset *and* the default — today's grid is flush to the
        // pane edge — so an unset side reads as the 0 px it genuinely is.
        row('padding top', num(p.paddingTop, '0 px', 'px')),
        row('padding right', num(p.paddingRight, '0 px', 'px')),
        row('padding bottom', num(p.paddingBottom, '0 px', 'px')),
        row('padding left', num(p.paddingLeft, '0 px', 'px')),
      ],
    },
    {
      title: 'Keys & selection',
      rows: [
        // The one unset-means-on pref: Shift+Enter is the capability the file exists
        // to deliver, so `false` is how an operator gets plain submit back.
        row('Shift+Enter writes a newline', flag(p.shiftEnterNewline, true)),
        row('copy on select', flag(p.copyOnSelect)),
        row('right-click selects word', flag(p.rightClickSelectsWord)),
        row('Option is Meta (macOS)', flag(p.macOptionIsMeta)),
      ],
    },
  ]
}
