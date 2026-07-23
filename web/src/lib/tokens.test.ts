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
})
