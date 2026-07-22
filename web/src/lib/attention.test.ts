// Attention (ticket 14): the action station's ranking, and the sidebar's
// ambient echo with the halt flag's jump target — all pure derivations
// over a snapshot, tested the same way ticket 13's session.test.ts tests
// `sessionStates`: tiny fixture builders, no DOM.

import { describe, it, expect } from 'vitest'
import {
  mapActionItems,
  mapActionCount,
  spaceActionCount,
  spaceAttention,
  spaceHaltTarget,
  spaceLiveness,
} from './attention'
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

function map(slug: string, kind: WMap['kind'], ...tickets: Ticket[]): WMap {
  return { slug, name: slug, dir: `/${slug}`, destination: '', tickets, finished: false, kind }
}

function space(id: string, extra: Partial<Space> = {}): Space {
  return {
    id,
    name: id,
    path: `/${id}`,
    pinned: false,
    dirty: false,
    bindings: [],
    skills: [],
    layers: [],
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
    session: { mapSlug, ticketNum, role: 'implement', agent: 'claude', model: 'opus' },
  }
}

function workingTerminal(mapSlug: string, ticketNum: number, status: Terminal['status'] = 'working'): Terminal {
  return {
    id: 't1',
    title: 'implement',
    proc: 'agent',
    status,
    alive: true,
    session: { mapSlug, ticketNum, role: 'implement', agent: 'claude', model: 'opus' },
  }
}

describe('mapActionItems', () => {
  it('ranks the frontier by unblock count, ties by ticket number', () => {
    // 1 blocks 2 and 3 (unblocks 2); 4 blocks nothing (unblocks 0); both are
    // frontier. 2 and 3 are blocked, so neither is actionable.
    const m = map(
      'impl',
      'implementation',
      ticket(1, { frontier: true }),
      ticket(2, { blockedBy: [1] }),
      ticket(3, { blockedBy: [1] }),
      ticket(4, { frontier: true }),
    )
    const items = mapActionItems(m)
    expect(items.map((i) => [i.ticket.num, i.unblockCount])).toEqual([
      [1, 2],
      [4, 0],
    ])
  })

  it('breaks an unblock-count tie by ticket number', () => {
    const m = map(
      'impl',
      'implementation',
      ticket(1, { frontier: true }),
      ticket(2, { frontier: true }),
    )
    const items = mapActionItems(m)
    expect(items.map((i) => i.ticket.num)).toEqual([1, 2])
  })

  it('offers nothing on an unclassified map, even with a frontier ticket', () => {
    const m = map('unk', '', ticket(1, { frontier: true }))
    expect(mapActionItems(m)).toEqual([])
  })

  it('counts mirror the item list, summed across a space', () => {
    const a = map('a', 'implementation', ticket(1, { frontier: true }))
    const b = map('b', 'planning', ticket(2, { frontier: true }), ticket(3, { frontier: true }))
    expect(mapActionCount(a)).toBe(1)
    expect(mapActionCount(b)).toBe(2)
    expect(spaceActionCount(space('s', { maps: [a, b] }))).toBe(3)
  })
})

describe('spaceHaltTarget', () => {
  it('names the halted session’s ticket — where the flag’s click lands', () => {
    const s = space('s2', {
      maps: [map('impl2', 'implementation', ticket(2))],
      terminals: [haltedTerminal('impl2', 2)],
    })
    expect(spaceHaltTarget(s)).toEqual({ mapSlug: 'impl2', ticketNum: 2 })
  })

  it('takes the first halted terminal in order — one glyph offers no choice', () => {
    const s = space('s', {
      maps: [map('m', 'implementation', ticket(1), ticket(2))],
      terminals: [workingTerminal('m', 9), haltedTerminal('m', 1), haltedTerminal('m', 2)],
    })
    expect(spaceHaltTarget(s)).toEqual({ mapSlug: 'm', ticketNum: 1 })
  })

  // The flag and the jump read the same predicate, so they can never disagree.
  it('is null exactly when the flag is not raised', () => {
    const s = space('s', {
      maps: [map('m', 'implementation', ticket(1, { frontier: true }))],
      terminals: [workingTerminal('m', 1)],
    })
    expect(spaceAttention(s)).toBe(null)
    expect(spaceHaltTarget(s)).toBe(null)
  })
})

describe('the sidebar echo', () => {
  it('flags a space with a halted session', () => {
    const s = space('s', { maps: [map('m', 'implementation', ticket(1))], terminals: [haltedTerminal('m', 1)] })
    expect(spaceAttention(s)).toBe('halt')
  })

  it('flags nothing for a space with no decision-level signal', () => {
    const s = space('s', { maps: [map('m', 'implementation', ticket(1, { frontier: true }))] })
    expect(spaceAttention(s)).toBe(null)
  })

  it('reads liveness independently of attention — both can hold at once', () => {
    const s = space('s', {
      maps: [map('m', 'implementation', ticket(1, { frontier: true }))],
      terminals: [workingTerminal('m', 9), haltedTerminal('m', 1)],
    })
    expect(spaceAttention(s)).toBe('halt')
    expect(spaceLiveness(s)).toBe('working')
  })

  it('prefers working over quiet, and is null with no live session', () => {
    expect(spaceLiveness(space('s', { terminals: [workingTerminal('m', 1, 'quiet')] }))).toBe('quiet')
    expect(spaceLiveness(space('s', { terminals: [] }))).toBe(null)
  })
})
