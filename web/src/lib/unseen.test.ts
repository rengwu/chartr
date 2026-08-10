// The dot on a tab's card (session-notifications): pure derivations over one
// snapshot flag, tested the way `attention.test.ts` tests the sidebar echo —
// tiny fixture builders, no DOM.

import { describe, it, expect } from 'vitest'
import { showsFinishedDot, acknowledgesFinishedRun } from './unseen'
import { spaceAttention, spaceLiveness } from './attention'
import type { Space, Terminal } from './model'

function terminal(extra: Partial<Terminal> = {}): Terminal {
  return {
    id: 't1',
    title: 'implement',
    proc: 'agent',
    status: 'idle',
    alive: true,
    session: { mapSlug: 'impl', ticketNum: 4, role: 'implement', agent: 'claude' },
    ...extra,
  }
}

function space(id: string, terminals: Terminal[]): Space {
  return {
    id,
    name: id,
    path: `/${id}`,
    dirty: false,
    maps: [],
    terminals,
  }
}

describe('the dot on a tab’s card', () => {
  it('shows on a tab that finished a qualifying run the operator has not looked at', () => {
    expect(showsFinishedDot(terminal({ finishedUnseen: true }), false)).toBe(true)
  })

  it('shows on nothing else — an ordinary tab carries no dot', () => {
    expect(showsFinishedDot(terminal(), false)).toBe(false)
    expect(showsFinishedDot(terminal({ status: 'working' }), false)).toBe(false)
    // A dead session is the halt's business, not the dot's; the two are separate
    // signals and a death alone raises no dot.
    expect(showsFinishedDot(terminal({ status: 'dead', alive: false }), false)).toBe(false)
  })

  // The flag is server state and clears through the endpoint, so there is a
  // round-trip in which the tab being looked at still carries it. The dot must not
  // flash on the tab in view for that beat.
  it('never shows on the tab in view', () => {
    expect(showsFinishedDot(terminal({ finishedUnseen: true }), true)).toBe(false)
  })

  it('is the same flag an ad-hoc shell can carry — a long build is a run like any other', () => {
    const shell = terminal({ session: undefined, proc: 'make', finishedUnseen: true })
    expect(showsFinishedDot(shell, false)).toBe(true)
  })
})

describe('acknowledging a finished run', () => {
  it('acknowledges exactly when the flagged tab is the one in view', () => {
    const t = terminal({ finishedUnseen: true })
    expect(acknowledgesFinishedRun(t, true)).toBe(true)
    expect(acknowledgesFinishedRun(t, false)).toBe(false)
  })

  it('acknowledges nothing for a tab with no dot, or no tab at all', () => {
    expect(acknowledgesFinishedRun(terminal(), true)).toBe(false)
    expect(acknowledgesFinishedRun(null, true)).toBe(false)
    expect(acknowledgesFinishedRun(undefined, true)).toBe(false)
  })

  // The dot and the acknowledgement read one flag from opposite sides: whichever
  // is true, the other is false, so a dot can never sit un-acknowledged on the tab
  // in view, and a tab out of view is never cleared behind the operator's back.
  it('is the exact complement of the dot for a flagged tab', () => {
    const t = terminal({ finishedUnseen: true })
    for (const inView of [true, false]) {
      expect(acknowledgesFinishedRun(t, inView)).toBe(!showsFinishedDot(t, inView))
    }
  })
})

// The guard: this signal is deliberately not part of the attention grammar. A tab
// carrying the dot must read to `attention.ts` exactly as it did before the flag
// existed — no new flag on a space row, no new liveness — which is what keeps a
// collapsed space showing nothing new (the accepted limit; the OS notification
// covers that case).
describe('the attention grammar is untouched', () => {
  it('leaves spaceAttention and spaceLiveness reading exactly what they read before', () => {
    const idle = terminal({ id: 't1' })
    const flagged = terminal({ id: 't1', finishedUnseen: true })
    expect(spaceAttention(space('s', [flagged]))).toBe(spaceAttention(space('s', [idle])))
    expect(spaceLiveness(space('s', [flagged]))).toBe(spaceLiveness(space('s', [idle])))
    expect(spaceAttention(space('s', [flagged]))).toBe(null)
    expect(spaceLiveness(space('s', [flagged]))).toBe(null)
  })

  it('does not promote a finished run over a live session’s own signals', () => {
    const halted = terminal({ id: 't1', status: 'dead', alive: false, finishedUnseen: true })
    const working = terminal({ id: 't2', status: 'working', finishedUnseen: true })
    const s = space('s', [halted, working])
    expect(spaceAttention(s)).toBe('halt')
    expect(spaceLiveness(s)).toBe('working')
  })
})
