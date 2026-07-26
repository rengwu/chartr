import { describe, expect, it } from 'vitest'
import { defaultRole, heldLive, sinceLabel, ROLES, type Role, type Terminal } from './model'

// The role a ticket spawns as comes from the ticket's own type — the one
// behaviour the kind cut changes. A map's kind used to clamp this to the roles
// its lifecycle offered; nothing clamps it now.
describe('defaultRole', () => {
  it('maps each of wayfinder’s four ticket types onto its role', () => {
    expect(defaultRole('grilling')).toBe('grill')
    expect(defaultRole('prototype')).toBe('prototype')
    expect(defaultRole('research')).toBe('research')
    expect(defaultRole('task')).toBe('implement')
  })

  it('falls to implement on an unrecognised type', () => {
    expect(defaultRole('')).toBe('implement')
    expect(defaultRole('spelunking')).toBe('implement')
  })

  it('never returns a role outside the closed set', () => {
    for (const type of ['grilling', 'prototype', 'research', 'task', 'nonsense']) {
      expect(ROLES).toContain(defaultRole(type) as Role)
    }
  })
})

// A tab holding a ticket, for heldLive.
function tab(alive: boolean, slug: string, ticketNum: number): Terminal {
  return {
    id: `t-${slug}-${ticketNum}-${alive}`,
    title: 'implement #01',
    proc: 'claude',
    status: alive ? 'working' : 'dead',
    alive,
    session: { mapSlug: slug, ticketNum, role: 'implement', agent: 'claude' },
  }
}

// Whether a live session is actually running a claimed ticket is what separates a
// claim being honoured from one left behind — the state the ticket-level release
// exists for. It is derived from the tabs, never from the ticket file.
describe('heldLive', () => {
  it('finds the live session on the ticket', () => {
    expect(heldLive([tab(true, 'widget', 1)], 'widget', 1)).toBe(true)
  })

  it('does not count a dead pinned tab — its claim is exactly what wants clearing', () => {
    expect(heldLive([tab(false, 'widget', 1)], 'widget', 1)).toBe(false)
  })

  it('does not count a session working elsewhere on the map, or on another map', () => {
    expect(heldLive([tab(true, 'widget', 2)], 'widget', 1)).toBe(false)
    expect(heldLive([tab(true, 'gadget', 1)], 'widget', 1)).toBe(false)
  })

  it('reads an orphaned claim — no tabs at all — as held by nobody', () => {
    expect(heldLive([], 'widget', 1)).toBe(false)
  })

  it('ignores ad-hoc shells, which hold no ticket', () => {
    const shell: Terminal = { id: 's1', title: 'zsh', proc: 'zsh', status: 'idle', alive: true }
    expect(heldLive([shell], 'widget', 1)).toBe(false)
  })
})

// A claim's age is read for "is this stuck?", so it is coarse on purpose.
describe('sinceLabel', () => {
  const now = Date.parse('2026-07-26T12:00:00Z')

  it('coarsens from minutes to days', () => {
    expect(sinceLabel('2026-07-26T11:59:30Z', now)).toBe('just now')
    expect(sinceLabel('2026-07-26T11:30:00Z', now)).toBe('30m ago')
    expect(sinceLabel('2026-07-26T09:00:00Z', now)).toBe('3h ago')
    expect(sinceLabel('2026-07-20T12:00:00Z', now)).toBe('6d ago')
  })

  it('renders nothing rather than a wrong number for an absent or unparseable stamp', () => {
    expect(sinceLabel(undefined, now)).toBe('')
    expect(sinceLabel('', now)).toBe('')
    expect(sinceLabel('last tuesday', now)).toBe('')
  })
})
