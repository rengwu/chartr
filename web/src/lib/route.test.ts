import { describe, expect, it } from 'vitest'
import { mapsHash, parseRoute, settingsHash } from './route'

// The one route the cockpit has (ticket 05). What matters at this seam is that
// the settings prefix and the star deep link never read each other's hashes —
// the two schemes share one address bar and must stay disjoint.

describe('parseRoute', () => {
  it('routes the settings prefix', () => {
    expect(parseRoute('#/settings')).toEqual({ settings: true })
    expect(parseRoute('/settings')).toEqual({ settings: true })
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

  it('lands an unknown sub-path on the settings route, not the cockpit', () => {
    expect(parseRoute('#/settings/nonsense')).toEqual({ settings: true })
    expect(parseRoute('#/settings/user')).toEqual({ settings: true })
  })
})

describe('settingsHash', () => {
  it('round-trips through parseRoute', () => {
    expect(parseRoute(settingsHash())).toEqual({ settings: true })
  })
})

describe('mapsHash', () => {
  it('builds a star deep link, never a settings one', () => {
    expect(mapsHash('abc')).toBe('#s=abc&maps=1')
    expect(parseRoute(mapsHash('abc')).settings).toBe(false)
  })
})
