import { describe, expect, it } from 'vitest'
import { mapsHash, parseRoute, settingsHash } from './route'

// The one route the cockpit has (ticket 05). What matters at this seam is that
// the settings prefix and the star deep link never read each other's hashes —
// the two schemes share one address bar and must stay disjoint.

describe('parseRoute', () => {
  it('routes the settings prefix and its scopes', () => {
    expect(parseRoute('#/settings')).toEqual({ settings: true, scope: { kind: 'default' } })
    expect(parseRoute('#/settings/user')).toEqual({ settings: true, scope: { kind: 'user' } })
    expect(parseRoute('#/settings/s=abc123')).toEqual({
      settings: true,
      scope: { kind: 'space', spaceId: 'abc123' },
    })
  })

  it('leaves the star deep link on the cockpit', () => {
    // The scheme the star-map has used since ticket 07: no leading slash.
    for (const hash of ['', '#', '#s=abc', '#s=abc&m=widget&t=3', '#s=abc&mat=1', '#s=abc&maps=1']) {
      expect(parseRoute(hash).settings, hash).toBe(false)
    }
  })

  it('does not mistake a space id that starts with settings-ish text', () => {
    expect(parseRoute('#s=settings').settings).toBe(false)
    expect(parseRoute('#settings').settings).toBe(false)
    expect(parseRoute('#/settingsX').settings).toBe(false)
  })

  it('decodes a space id and tolerates a hash with no leading #', () => {
    expect(parseRoute('/settings/s=a%2Fb')).toEqual({
      settings: true,
      scope: { kind: 'space', spaceId: 'a/b' },
    })
  })

  it('lands an unknown or empty sub-path on the settings route, not the cockpit', () => {
    expect(parseRoute('#/settings/nonsense')).toEqual({ settings: true, scope: { kind: 'default' } })
    expect(parseRoute('#/settings/s=')).toEqual({ settings: true, scope: { kind: 'default' } })
  })
})

describe('settingsHash', () => {
  it('round-trips every scope through parseRoute', () => {
    for (const scope of [
      { kind: 'default' },
      { kind: 'user' },
      { kind: 'space', spaceId: 'abc' },
      { kind: 'space', spaceId: 'a/b c' },
    ] as const) {
      expect(parseRoute(settingsHash(scope))).toEqual({ settings: true, scope })
    }
  })
})

describe('mapsHash', () => {
  it('builds a star deep link, never a settings one', () => {
    expect(mapsHash('abc')).toBe('#s=abc&maps=1')
    expect(parseRoute(mapsHash('abc')).settings).toBe(false)
  })
})
