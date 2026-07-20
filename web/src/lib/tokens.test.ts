import { afterEach, describe, expect, it } from 'vitest'
import { readColor, readToken, readTokens, resolveColor } from './tokens'

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
