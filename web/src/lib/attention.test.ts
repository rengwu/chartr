// Attention (ticket 14): the sidebar's ambient echo with the halt flag's jump
// target — all pure derivations over a snapshot, tested the same way ticket
// 13's session.test.ts tests `sessionStates`: tiny fixture builders, no DOM.

import { describe, it, expect } from 'vitest'
import { spaceAttention, spaceHaltTarget, spaceLiveness } from './attention'
import type { Map as WMap, Space, Terminal, Ticket } from './model'

function ticket(num: number, extra: Partial<Ticket> = {}): Ticket {
  return {
    num,
    slug: `${num}`,
    title: `Ticket ${num}`,
    type: 'task',
    status: 'open',
    blockedBy: [],
    frontier: false,
    ...extra,
  }
}

function map(slug: string, ...tickets: Ticket[]): WMap {
  return { slug, name: slug, dir: `/${slug}`, destination: '', tickets, finished: false }
}

function space(id: string, extra: Partial<Space> = {}): Space {
  return {
    id,
    name: id,
    path: `/${id}`,
    maps: [],
    terminals: [],
    ...extra,
  }
}

function haltedTerminal(mapSlug: string, ticketNum: number): Terminal {
  return {
    id: 't1',
    title: 'implement',
    proc: 'agent',
    status: 'dead',
    alive: false,
    session: { mapSlug, ticketNum, role: 'implement', agent: 'claude' },
  }
}

function workingTerminal(mapSlug: string, ticketNum: number, status: Terminal['status'] = 'working'): Terminal {
  return {
    id: 't1',
    title: 'implement',
    proc: 'agent',
    status,
    alive: true,
    session: { mapSlug, ticketNum, role: 'implement', agent: 'claude' },
  }
}

describe('spaceHaltTarget', () => {
  it('names the halted session’s ticket — where the flag’s click lands', () => {
    const s = space('s2', {
      maps: [map('impl2', ticket(2))],
      terminals: [haltedTerminal('impl2', 2)],
    })
    expect(spaceHaltTarget(s)).toEqual({ mapSlug: 'impl2', ticketNum: 2 })
  })

  it('takes the first halted terminal in order — one glyph offers no choice', () => {
    const s = space('s', {
      maps: [map('m', ticket(1), ticket(2))],
      terminals: [workingTerminal('m', 9), haltedTerminal('m', 1), haltedTerminal('m', 2)],
    })
    expect(spaceHaltTarget(s)).toEqual({ mapSlug: 'm', ticketNum: 1 })
  })

  // The flag and the jump read the same predicate, so they can never disagree.
  it('is null exactly when the flag is not raised', () => {
    const s = space('s', {
      maps: [map('m', ticket(1, { frontier: true }))],
      terminals: [workingTerminal('m', 1)],
    })
    expect(spaceAttention(s)).toBe(null)
    expect(spaceHaltTarget(s)).toBe(null)
  })
})

describe('the sidebar echo', () => {
  it('flags a space with a halted session', () => {
    const s = space('s', { maps: [map('m', ticket(1))], terminals: [haltedTerminal('m', 1)] })
    expect(spaceAttention(s)).toBe('halt')
  })

  it('flags nothing for a space with no decision-level signal', () => {
    const s = space('s', { maps: [map('m', ticket(1, { frontier: true }))] })
    expect(spaceAttention(s)).toBe(null)
  })

  it('reads liveness independently of attention — both can hold at once', () => {
    const s = space('s', {
      maps: [map('m', ticket(1, { frontier: true }))],
      terminals: [workingTerminal('m', 9), haltedTerminal('m', 1)],
    })
    expect(spaceAttention(s)).toBe('halt')
    expect(spaceLiveness(s)).toBe('working')
  })

  it('prefers working over blocked, and is null with no live session', () => {
    expect(spaceLiveness(space('s', { terminals: [workingTerminal('m', 1, 'blocked')] }))).toBe('blocked')
    expect(spaceLiveness(space('s', { terminals: [] }))).toBe(null)
  })
})
