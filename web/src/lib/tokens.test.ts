import { afterEach, describe, expect, it } from 'vitest'
import {
  buildTerminalOptions,
  readColor,
  readToken,
  readTokens,
  resolveColor,
} from './tokens'

// The bridge reads live custom properties off the document. Under jsdom we set
// tokens as inline styles on the root and assert the reader returns them; jsdom
// does not do oklch colour maths, so colour-resolution asserts use hex/rgb
// inputs that any engine passes through unchanged.

afterEach(() => {
  document.documentElement.removeAttribute('style')
})

describe('readToken', () => {
  it('returns the declared value of a known token', () => {
    document.documentElement.style.setProperty('--probe-token', '#1e2530')
    expect(readToken('--probe-token')).toBe('#1e2530')
  })

  it('trims surrounding whitespace', () => {
    document.documentElement.style.setProperty('--probe-token', '  #abcdef  ')
    expect(readToken('--probe-token')).toBe('#abcdef')
  })

  it('returns empty string for an unset token', () => {
    expect(readToken('--not-a-real-token')).toBe('')
  })
})

describe('resolveColor', () => {
  it('normalises a hex colour to rgb', () => {
    expect(resolveColor('#ff0000')).toBe('rgb(255, 0, 0)')
  })

  it('passes an empty value through', () => {
    expect(resolveColor('')).toBe('')
  })
})

describe('readColor', () => {
  it('reads a token and resolves it to a concrete colour', () => {
    document.documentElement.style.setProperty('--probe-surface', '#00ff00')
    expect(readColor('--probe-surface')).toBe('rgb(0, 255, 0)')
  })
})

describe('readTokens', () => {
  it('reads a named map of tokens in one pass', () => {
    document.documentElement.style.setProperty('--probe-bg', '#000000')
    document.documentElement.style.setProperty('--probe-fg', '#ffffff')
    expect(readTokens({ bg: '--probe-bg', fg: '--probe-fg' })).toEqual({
      bg: '#000000',
      fg: '#ffffff',
    })
  })
})

// Seam 2 — the pure resolve of TerminalPrefs into xterm options + theme. Under
// jsdom the tokens are inline hex on the root (no oklch maths), so the base
// colours resolve to rgb; a pref colour is used verbatim, and an unset slot falls
// through to the token default.
describe('buildTerminalOptions', () => {
  // The base theme slots that resolve off live tokens; set them so the resolve
  // has something concrete to read, mirroring the reskin's real token names.
  function seedTokens() {
    const root = document.documentElement.style
    root.setProperty('--background', '#101010')
    root.setProperty('--foreground', '#f0f0f0')
    root.setProperty('--muted-foreground', '#808080')
    root.setProperty('--muted', '#303030')
    root.setProperty('--ring', '#00ff00')
    root.setProperty('--destructive', '#ff0000')
  }

  it('resolves unset colours against the live design tokens', () => {
    seedTokens()
    const { theme, options } = buildTerminalOptions(undefined)
    expect(theme.background).toBe('rgb(16, 16, 16)')
    expect(theme.foreground).toBe('rgb(240, 240, 240)')
    expect(theme.cursor).toBe('rgb(0, 255, 0)')
    // An unset font/size falls to the bundled default.
    expect(options.fontFamily).toContain('IBM Plex Mono')
    expect(options.fontSize).toBe(13)
  })

  it('uses a pref colour verbatim and stacks a pref font ahead of the default', () => {
    seedTokens()
    const { theme, options } = buildTerminalOptions({
      background: '#1e2530',
      fontFamily: 'Fira Code',
      fontSize: 16,
    })
    expect(theme.background).toBe('#1e2530')
    // Foreground was left unset, so it still resolves off the token.
    expect(theme.foreground).toBe('rgb(240, 240, 240)')
    expect(options.fontFamily).toBe(
      "Fira Code, 'IBM Plex Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
    )
    expect(options.fontSize).toBe(16)
  })

  it('carries the default ANSI hues the monochrome chrome has no token for', () => {
    seedTokens()
    const { theme } = buildTerminalOptions({})
    expect(theme.green).toBe('#9cb68c')
    expect(theme.blue).toBe('#82a8c9')
  })

  it('applies a named preset over the token base', () => {
    seedTokens()
    const { theme } = buildTerminalOptions({ preset: 'dracula' })
    // Every slot the preset names replaces the token/ANSI-default base.
    expect(theme.background).toBe('#282a36')
    expect(theme.foreground).toBe('#f8f8f2')
    expect(theme.green).toBe('#50fa7b')
    expect(theme.selectionBackground).toBe('#44475a')
  })

  it('lets an explicit slot win over the preset', () => {
    seedTokens()
    const { theme } = buildTerminalOptions({
      preset: 'dracula',
      background: '#000000',
      green: '#00ff00',
      selection: '#111111',
    })
    // Explicit slots win; `selection` drives xterm's selectionBackground.
    expect(theme.background).toBe('#000000')
    expect(theme.green).toBe('#00ff00')
    expect(theme.selectionBackground).toBe('#111111')
    // A slot the file left unset still comes from the preset.
    expect(theme.blue).toBe('#bd93f9')
  })

  it('resolves an unset slot to the token/default layer when no preset is set', () => {
    seedTokens()
    const { theme } = buildTerminalOptions({ blue: '#123456' })
    // The one explicit slot wins…
    expect(theme.blue).toBe('#123456')
    // …an ANSI slot with no token falls to its default preset layer…
    expect(theme.green).toBe('#9cb68c')
    // …and a tokened slot still resolves off the live token.
    expect(theme.background).toBe('rgb(16, 16, 16)')
  })

  it('ignores an unknown preset name and keeps the base', () => {
    // The server drops unknown names before the wire, but the resolve is defensive:
    // a name with no bundled palette simply leaves the base showing.
    seedTokens()
    const { theme } = buildTerminalOptions({ preset: 'not-a-preset' })
    expect(theme.background).toBe('rgb(16, 16, 16)')
    expect(theme.green).toBe('#9cb68c')
  })

  it('resolves a bundled font by name to its clean stack', () => {
    seedTokens()
    const { options } = buildTerminalOptions({ fontFamily: 'IBM Plex Mono' })
    // A bundled name resolves to the guaranteed-offline stack without doubling.
    expect(options.fontFamily).toBe(
      "'IBM Plex Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
    )
  })

  it('stacks a non-bundled family ahead of the system fallback', () => {
    seedTokens()
    const { options } = buildTerminalOptions({ fontFamily: 'Cascadia Code' })
    expect(options.fontFamily).toBe(
      "Cascadia Code, 'IBM Plex Mono', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
    )
  })

  it('maps the font, cursor, scrolling, and contrast options onto xterm', () => {
    seedTokens()
    const { options } = buildTerminalOptions({
      fontWeight: '300',
      fontWeightBold: 'bold',
      lineHeight: 1.4,
      letterSpacing: -0.5,
      cursorStyle: 'bar',
      cursorBlink: false,
      cursorInactiveStyle: 'none',
      cursorWidth: 3,
      scrollback: 8000,
      scrollSensitivity: 2,
      fastScrollModifier: 'ctrl',
      fastScrollSensitivity: 9,
      smoothScrollDuration: 150,
      minimumContrastRatio: 4.5,
    })
    // A numeric weight becomes a number; a keyword passes through.
    expect(options.fontWeight).toBe(300)
    expect(options.fontWeightBold).toBe('bold')
    expect(options.lineHeight).toBe(1.4)
    expect(options.letterSpacing).toBe(-0.5)
    expect(options.cursorStyle).toBe('bar')
    expect(options.cursorBlink).toBe(false)
    expect(options.cursorInactiveStyle).toBe('none')
    expect(options.cursorWidth).toBe(3)
    expect(options.scrollback).toBe(8000)
    expect(options.scrollSensitivity).toBe(2)
    expect(options.fastScrollModifier).toBe('ctrl')
    expect(options.fastScrollSensitivity).toBe(9)
    expect(options.smoothScrollDuration).toBe(150)
    expect(options.minimumContrastRatio).toBe(4.5)
  })

  it('leaves an unset pass-through option off the options object', () => {
    // An unset pref never overwrites xterm's own default with undefined — the key is
    // simply absent, so xterm keeps its built-in value.
    seedTokens()
    const { options } = buildTerminalOptions({})
    expect('fontWeight' in options).toBe(false)
    expect('cursorStyle' in options).toBe(false)
    expect('cursorBlink' in options).toBe(false)
    expect('scrollback' in options).toBe(false)
    expect('minimumContrastRatio' in options).toBe(false)
  })

  it('drops values the server would have rejected, defensively', () => {
    // The seam mirrors the server guards so it is safe on its own: a non-positive
    // size/line-height/width and an out-of-range contrast ratio leave the option
    // unset rather than passing a broken value to xterm.
    seedTokens()
    const { options } = buildTerminalOptions({
      lineHeight: 0,
      cursorWidth: -2,
      minimumContrastRatio: 30,
      cursorStyle: 'beam',
    })
    expect('lineHeight' in options).toBe(false)
    expect('cursorWidth' in options).toBe(false)
    expect('minimumContrastRatio' in options).toBe(false)
    expect('cursorStyle' in options).toBe(false)
  })
})
