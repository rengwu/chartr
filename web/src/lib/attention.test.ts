// Attention (ticket 14): the action station's ranking, the cross-space queue's
// gate-level filter, and the sidebar's ambient echo — all pure derivations
// over a snapshot, tested the same way ticket 13's session.test.ts tests
// `sessionStates`: tiny fixture builders, no DOM.

import { describe, it, expect } from 'vitest'
import {
  mapActionItems,
  mapActionCount,
  spaceActionCount,
  needsYouQueue,
  spaceAttention,
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

function atTheGate(t: Ticket): Ticket {
  return { ...t, review: { sessionId: 's1', recommendation: 'Send back', blocking: 1, advisories: 0 } }
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
  it('ranks reviews first, then the frontier by unblock count, ties by ticket number', () => {
    // 1 blocks 2 and 3 (unblocks 2); 4 blocks nothing (unblocks 0); both are
    // frontier. 5 is proposed with a review waiting.
    const m = map(
      'impl',
      'implementation',
      ticket(1, { frontier: true }),
      ticket(2, { blockedBy: [1] }),
      ticket(3, { blockedBy: [1] }),
      ticket(4, { frontier: true }),
      atTheGate(ticket(5, { status: 'proposed' })),
    )
    const items = mapActionItems(m)
    expect(items.map((i) => [i.kind, i.ticket.num, i.unblockCount])).toEqual([
      ['review', 5, 0],
      ['frontier', 1, 2],
      ['frontier', 4, 0],
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

describe('needsYouQueue', () => {
  it('pulls exactly the gate-level signals — a review, a halted session', () => {
    const reviewed = map('impl', 'implementation', atTheGate(ticket(1, { status: 'proposed' })))
    const s1 = space('s1', { maps: [reviewed] })

    const withHalt = map('impl2', 'implementation', ticket(2))
    const s2 = space('s2', { maps: [withHalt], terminals: [haltedTerminal('impl2', 2)] })

    // A live, working session is ambient, not actionable — it must not appear.
    const withLiveSession = map('impl3', 'implementation', ticket(3))
    const s3 = space('s3', { maps: [withLiveSession], terminals: [workingTerminal('impl3', 3)] })

    const entries = needsYouQueue([s1, s2, s3])
    expect(entries).toEqual([
      {
        spaceId: 's1',
        spaceName: 's1',
        mapSlug: 'impl',
        mapName: 'impl',
        ticketNum: 1,
        ticketTitle: 'Ticket 1',
        kind: 'review',
      },
      {
        spaceId: 's2',
        spaceName: 's2',
        mapSlug: 'impl2',
        mapName: 'impl2',
        ticketNum: 2,
        ticketTitle: 'Ticket 2',
        kind: 'halt',
      },
    ])
  })

  it('is empty when nothing across any space needs a decision', () => {
    const quiet = map('m', 'implementation', ticket(1, { frontier: true }))
    expect(needsYouQueue([space('s', { maps: [quiet] })])).toEqual([])
  })
})

describe('the sidebar echo', () => {
  it('flags a space wanting a review over one merely halted', () => {
    const reviewed = map('m', 'implementation', atTheGate(ticket(1, { status: 'proposed' })))
    const both = space('s', {
      maps: [reviewed],
      terminals: [haltedTerminal('m', 2)],
    })
    expect(spaceAttention(both)).toBe('review')
  })

  it('flags halt when there is no review waiting', () => {
    const s = space('s', { maps: [map('m', 'implementation', ticket(1))], terminals: [haltedTerminal('m', 1)] })
    expect(spaceAttention(s)).toBe('halt')
  })

  it('flags nothing for a space with no gate-level signal', () => {
    const s = space('s', { maps: [map('m', 'implementation', ticket(1, { frontier: true }))] })
    expect(spaceAttention(s)).toBe(null)
  })

  it('reads liveness independently of attention — both can hold at once', () => {
    const reviewed = map('m', 'implementation', atTheGate(ticket(1, { status: 'proposed' })))
    const s = space('s', {
      maps: [reviewed],
      terminals: [workingTerminal('m', 9)],
    })
    expect(spaceAttention(s)).toBe('review')
    expect(spaceLiveness(s)).toBe('working')
  })

  it('prefers working over quiet, and is null with no live session', () => {
    expect(spaceLiveness(space('s', { terminals: [workingTerminal('m', 1, 'quiet')] }))).toBe('quiet')
    expect(spaceLiveness(space('s', { terminals: [] }))).toBe(null)
  })
})
