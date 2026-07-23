import { afterEach, describe, expect, it } from 'vitest'
import type { TerminalPrefs } from './model'
import { terminalSettingsSummary, type TerminalSettingRow } from './terminalsummary'

// The read half of the Settings surface's Terminal section: prefs in, the rows
// the section renders out. It is a formatter over the same Seam 2 resolve the
// island mounts with, so what is asserted here is the promise the section makes —
// a row the file set reads as set, a row it left alone still shows the default in
// force, and a colour slot carries the concrete colour the terminal paints with.
//
// Under jsdom the design tokens are inline hex on the root (no oklch maths), the
// same arrangement tokens.test.ts uses.

function seedTokens() {
  const root = document.documentElement.style
  root.setProperty('--background', '#101010')
  root.setProperty('--foreground', '#f0f0f0')
  root.setProperty('--muted-foreground', '#808080')
  root.setProperty('--muted', '#303030')
  root.setProperty('--ring', '#00ff00')
  root.setProperty('--destructive', '#ff0000')
}

afterEach(() => {
  document.documentElement.removeAttribute('style')
})

function find(prefs: TerminalPrefs | undefined, group: string, label: string): TerminalSettingRow {
  const g = terminalSettingsSummary(prefs).find((g) => g.title === group)
  const row = g?.rows.find((r) => r.label === label)
  if (!row) throw new Error(`no ${group} / ${label} row in the summary`)
  return row
}

describe('terminalSettingsSummary', () => {
  it('shows the defaults in force on a machine with no terminal.toml', () => {
    seedTokens()
    expect(find(undefined, 'Font', 'family')).toMatchObject({
      value: 'IBM Plex Mono (bundled)',
      set: false,
    })
    expect(find(undefined, 'Font', 'size')).toMatchObject({ value: '13 px', set: false })
    expect(find(undefined, 'Rendering', 'renderer')).toMatchObject({ value: 'GPU (WebGL)' })
    expect(find(undefined, 'Cursor', 'style')).toMatchObject({ value: 'block', set: false })
    expect(find(undefined, 'Scrolling', 'scrollback')).toMatchObject({
      value: '1000 lines',
      set: false,
    })
  })

  it('marks the values terminal.toml actually set', () => {
    seedTokens()
    const prefs: TerminalPrefs = { fontSize: 15, fontFamily: 'Fira Code', scrollback: 5000 }
    expect(find(prefs, 'Font', 'size')).toMatchObject({ value: '15 px', set: true })
    expect(find(prefs, 'Font', 'family')).toMatchObject({ value: 'Fira Code', set: true })
    expect(find(prefs, 'Scrolling', 'scrollback')).toMatchObject({ value: '5000 lines', set: true })
    // An untouched neighbour in the same group still reads as a default.
    expect(find(prefs, 'Scrolling', 'sensitivity').set).toBe(false)
  })

  it('carries the resolved colour of a slot the file left unset', () => {
    seedTokens()
    const row = find(undefined, 'Theme', 'background')
    expect(row.set).toBe(false)
    expect(row.value).toBe('from the theme')
    // The token-derived colour the terminal genuinely paints with.
    expect(row.swatch).toBe('rgb(16, 16, 16)')
  })

  it('carries an explicit slot verbatim, as both value and swatch', () => {
    seedTokens()
    const row = find({ background: '#1e2530' }, 'Theme', 'background')
    expect(row).toMatchObject({ value: '#1e2530', swatch: '#1e2530', set: true })
  })

  it('shows a preset name and the colours it puts in force', () => {
    seedTokens()
    const rows = (label: string) => find({ preset: 'dracula' }, 'Theme', label)
    expect(rows('preset')).toMatchObject({ value: 'dracula', set: true })
    // The slot itself is not set by the file — the preset below it is what shows.
    expect(rows('background')).toMatchObject({ set: false, swatch: '#282a36' })
  })

  it('reads Shift+Enter as on when the file is silent, and off only when it says so', () => {
    seedTokens()
    expect(find(undefined, 'Keys & selection', 'Shift+Enter writes a newline')).toMatchObject({
      value: 'on',
      set: false,
    })
    expect(
      find({ shiftEnterNewline: false }, 'Keys & selection', 'Shift+Enter writes a newline'),
    ).toMatchObject({ value: 'off', set: true })
    // Every other tri-state is unset-means-off.
    expect(find(undefined, 'Keys & selection', 'copy on select')).toMatchObject({
      value: 'off',
      set: false,
    })
  })

  it('follows the renderer decision ligatures forces', () => {
    seedTokens()
    expect(find({ ligatures: true }, 'Rendering', 'renderer').value).toBe('canvas')
    expect(find({ ligatures: true }, 'Rendering', 'ligatures')).toMatchObject({
      value: 'on',
      set: true,
    })
    // A non-bundled family suppresses them, and the row says why rather than
    // claiming an 'on' the terminal does not honour.
    const custom = find({ ligatures: true, fontFamily: 'Comic Mono' }, 'Rendering', 'ligatures')
    expect(custom.value).toBe('off — needs a bundled font')
    expect(find({ ligatures: true, fontFamily: 'Comic Mono' }, 'Rendering', 'renderer').value).toBe(
      'GPU (WebGL)',
    )
  })

  it('reads an unset padding side as the flush-to-the-edge default', () => {
    seedTokens()
    expect(find({ paddingLeft: 12 }, 'Scrollbar & padding', 'padding left')).toMatchObject({
      value: '12 px',
      set: true,
    })
    expect(find({ paddingLeft: 12 }, 'Scrollbar & padding', 'padding top')).toMatchObject({
      value: '0 px',
      set: false,
    })
  })
})
