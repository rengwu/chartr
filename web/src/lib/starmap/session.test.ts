// The session overlay's two halves, tested where they are decidable: the
// derivation (pushed snapshot → a state per ticket) and the grammar's
// no-colour-only property. What the states *look* like is the renderer's, and is
// asserted through the island seam in starmap.test.ts.

import { describe, it, expect } from 'vitest'
import { GRAMMAR, nonColorSignature, sessionStates, type SessionState } from './session'
import type { Map as WMap, Terminal, Ticket } from '../model'

function ticket(num: number, status: Ticket['status']): Ticket {
  return { num, slug: `${num}`, title: `t${num}`, type: 'task', status, blockedBy: [], frontier: false }
}

function map(...tickets: Ticket[]): WMap {
  return { slug: 'm', name: 'M', dir: '/m', destination: '', tickets, finished: false, kind: 'implementation' }
}

function tab(ticketNum: number, role: string, status: Terminal['status'], alive: boolean): Terminal {
  return {
    id: `${role}-${ticketNum}`,
    title: role,
    proc: 'agent',
    status,
    alive,
    session: { mapSlug: 'm', ticketNum, role, agent: 'claude', model: 'opus' },
  }
}

describe('deriving the session overlay from a pushed snapshot', () => {
  it('reads liveness off the session tab holding the ticket', () => {
    const m = map(ticket(1, 'claimed'), ticket(2, 'claimed'), ticket(3, 'claimed'))
    const states = sessionStates(m, [
      tab(1, 'implement', 'working', true),
      tab(2, 'implement', 'quiet', true),
      tab(3, 'implement', 'dead', false),
    ])
    expect(states).toEqual({ 1: 'implementing', 2: 'quiet', 3: 'dead' })
  })

  it('says nothing about a ticket no session holds', () => {
    // A claim whose session is gone, a frontier ticket, a resolved one: the
    // overlay is about sessions, so all three carry only their base star.
    const m = map(ticket(1, 'claimed'), ticket(2, 'open'), ticket(3, 'resolved'))
    expect(sessionStates(m, [])).toEqual({})
  })

  it('walks a proposal through the review pipeline', () => {
    const m = map(ticket(1, 'proposed'))
    // The implementer proposed and died: work landed, nobody circling.
    expect(sessionStates(m, [tab(1, 'implement', 'dead', false)])).toEqual({ 1: 'proposed' })
    // A live reviewer circles it.
    const reviewing = [tab(1, 'implement', 'dead', false), tab(1, 'review', 'working', true)]
    expect(sessionStates(m, reviewing)).toEqual({ 1: 'agent-review' })
    // The reviewer exits: the verdict is written and the brief awaits a human.
    const reviewed = [tab(1, 'implement', 'dead', false), tab(1, 'review', 'dead', false)]
    expect(sessionStates(m, reviewed)).toEqual({ 1: 'human-review' })
  })

  it('ignores tabs belonging to another map or to no ticket at all', () => {
    const m = map(ticket(1, 'claimed'))
    const shell: Terminal = { id: 'sh', title: 'zsh', proc: 'zsh', status: 'idle', alive: true }
    const elsewhere = tab(1, 'implement', 'working', true)
    elsewhere.session = { ...elsewhere.session!, mapSlug: 'other' }
    expect(sessionStates(m, [shell, elsewhere])).toEqual({})
  })
})

describe('the grammar', () => {
  const ALL: SessionState[] = [
    'implementing',
    'quiet',
    'dead',
    'proposed',
    'agent-review',
    'human-review',
  ]

  it('carries a non-colour channel for every state', () => {
    // Motion or shape, never colour alone: each state names at least one, and no
    // two states share the same set — so the overlay survives greyscale.
    for (const s of ALL) {
      const g = GRAMMAR[s]
      expect(g.motion).toBeTruthy()
      expect(g.moon).toBeTruthy()
    }
    const sigs = ALL.map(nonColorSignature)
    expect(new Set(sigs).size).toBe(ALL.length)
  })

  it('spends exactly one new hue', () => {
    // The base six statuses own the palette; the session axis adds violet for
    // agent review and otherwise re-uses the claim's amber and its warm light.
    const hues = new Set(ALL.map((s) => GRAMMAR[s].hue))
    expect(hues.size).toBeLessThanOrEqual(5)
    expect(GRAMMAR['agent-review'].hue).not.toBe(GRAMMAR.implementing.hue)
  })
})
