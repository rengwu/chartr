// The star-map island's seam is the one frontend test point (spec, Testing
// Decisions): mount, receive the model, emit selection. These tests pin the
// renderer's two binding guarantees — deterministic layout from ticket data, and
// zero star movement across the full lifecycle — and the selection emission.
// The canvas *feel* is not tested here (it can only be judged by eye); the
// island runs headless, so no 2D context is required.

import { describe, it, expect, afterEach, beforeEach } from 'vitest'
import { StarMap } from './starmap'
import { computeLayout, structureSignature } from './layout'
import { GRAMMAR, nonColorSignature, type SessionState } from './session'
import type { Ticket } from '../model'

// A small implementation fixture — a real-shaped graph with a fan-out and a
// join, so layout has edges to relax and ranks to bias.
function fixture(overrides: Partial<Record<number, Ticket['status']>> = {}): Ticket[] {
  const base: Array<[number, string, number[]]> = [
    [1, 'The filtering seam and the streaming CSV download', []],
    [2, 'Per-date historical conversion and rate backfill', [1]],
    [3, 'The Export page and its preview fragment', [1]],
    [4, 'The live-preview endpoint and island', [3]],
    [5, 'Docs and architecture catch up with what shipped', [2, 4]],
  ]
  return base.map(([num, title, blockedBy]) => ({
    num,
    slug: `${num}`,
    title,
    type: 'task',
    status: overrides[num] ?? 'open',
    blockedBy,
    frontier: overrides[num] === undefined && blockedBy.length === 0,
  }))
}

// Walk one ticket through the whole lifecycle, returning a distinct status map
// per beat. The structure (tickets + edges) never changes — only statuses do.
const LIFECYCLE: Array<Record<number, Ticket['status']>> = [
  { 1: 'resolved', 2: 'resolved', 3: 'claimed' },
  { 1: 'resolved', 2: 'resolved', 3: 'resolved', 4: 'claimed' },
  { 1: 'resolved', 2: 'resolved', 3: 'resolved', 4: 'resolved', 5: 'out_of_scope' },
]

function mounted(): { sm: StarMap; host: HTMLDivElement } {
  const host = document.createElement('div')
  Object.defineProperty(host, 'clientWidth', { value: 1000, configurable: true })
  Object.defineProperty(host, 'clientHeight', { value: 700, configurable: true })
  document.body.appendChild(host)
  const sm = new StarMap()
  sm.mount(host)
  return { sm, host }
}

// The same fixture with the frontier flag stated explicitly — approval igniting a
// dependent moves a ticket onto the frontier, which the status alone can't say.
function beat(overrides: Partial<Record<number, Ticket['status']>>, frontier: number[] = []): Ticket[] {
  return fixture(overrides).map((t) => ({ ...t, frontier: frontier.includes(t.num) }))
}

// Drag the map, the way an operator does: mousedown on the canvas, a move past
// the click threshold, mouseup. The island's own pointer path, no test seam.
function pan(host: HTMLElement, dx: number, dy: number): void {
  const canvas = host.querySelector('canvas')!
  canvas.dispatchEvent(new MouseEvent('mousedown', { clientX: 0, clientY: 0, bubbles: true }))
  window.dispatchEvent(new MouseEvent('mousemove', { clientX: dx, clientY: dy, bubbles: true }))
  window.dispatchEvent(new MouseEvent('mouseup', { clientX: dx, clientY: dy, bubbles: true }))
}

describe('deterministic layout', () => {
  it('lays the same data out to the same positions every time', () => {
    const a = computeLayout(fixture())
    const b = computeLayout(fixture())
    expect(b).toEqual(a)
  })

  it('is independent of the order tickets arrive in', () => {
    const forward = fixture()
    const shuffled = [...forward].reverse()
    expect(computeLayout(shuffled)).toEqual(computeLayout(forward))
  })

  it('does not take status as an input to layout', () => {
    const open = computeLayout(fixture())
    const mixed = computeLayout(fixture({ 1: 'resolved', 3: 'claimed', 5: 'out_of_scope' }))
    expect(mixed).toEqual(open)
  })
})

describe('the island seam', () => {
  let sm: StarMap
  beforeEach(() => {
    sm = mounted().sm
  })

  it('renders all five base states without a 2D context', () => {
    // open→frontier, open→blocked, claimed, resolved, out_of_scope — one push
    // carrying all five, and every star present in the model.
    sm.setModel([
      { num: 1, slug: '1', title: 'frontier', type: 'task', status: 'open', frontier: true, blockedBy: [] },
      { num: 2, slug: '2', title: 'blocked', type: 'task', status: 'open', frontier: false, blockedBy: [1] },
      { num: 3, slug: '3', title: 'claimed', type: 'task', status: 'claimed', frontier: false, blockedBy: [] },
      { num: 4, slug: '4', title: 'resolved', type: 'task', status: 'resolved', frontier: false, blockedBy: [] },
      { num: 5, slug: '5', title: 'oos', type: 'task', status: 'out_of_scope', frontier: false, blockedBy: [] },
    ])
    expect(Object.keys(sm.positions()).sort()).toEqual(['1', '2', '3', '4', '5'])
  })

  it('never moves a star as the model pushes new statuses across the lifecycle', () => {
    sm.setModel(fixture())
    const start = sm.positions()
    for (const beat of LIFECYCLE) {
      sm.setModel(fixture(beat))
      expect(sm.positions()).toEqual(start)
    }
  })

  it('recomputes layout only when the structure changes', () => {
    sm.setModel(fixture())
    const five = sm.positions()
    expect(structureSignature(fixture())).toBe(structureSignature(fixture({ 3: 'claimed' })))

    // Dropping a ticket changes the structure — positions may legitimately move.
    const four = fixture().filter((t) => t.num !== 5)
    sm.setModel(four)
    expect(Object.keys(sm.positions()).sort()).toEqual(['1', '2', '3', '4'])
    expect(sm.positions()[5]).toBeUndefined()
    // The remaining stars are the deterministic four-node layout.
    expect(sm.positions()).toEqual(computeLayout(four))
    // And re-pushing the five-node map restores exactly the original positions.
    sm.setModel(fixture())
    expect(sm.positions()).toEqual(five)
  })

  it('emits selection when a star is clicked, and deselection on empty space', () => {
    sm.setModel(fixture())
    const emitted: (number | null)[] = []
    sm.onSelect((n) => emitted.push(n))

    const p = sm.screenOf(3)!
    expect(sm.selectAtScreen(p.x, p.y)).toBe(3)
    expect(emitted.at(-1)).toBe(3)

    // A click far from any star deselects.
    expect(sm.selectAtScreen(-9999, -9999)).toBe(null)
    expect(emitted.at(-1)).toBe(null)
  })

  it('seats a selected star in the free rect the pane leaves, in either docking', () => {
    // Viewport is 1000×700 (see mounted()). A star must never sit under the pane.
    sm.setModel(fixture())

    // Right dock: a 400px pane on the right. The star seats left of it.
    sm.setInsets({ top: 16, right: 400, bottom: 16, left: 16 })
    sm.select(3)
    const right = sm.screenOf(3)!
    expect(right.x).toBeLessThan(1000 - 400)

    // Re-dock to the bottom: the same star re-seats above the pane.
    sm.setInsets({ top: 16, right: 16, bottom: 300, left: 16 })
    const bottom = sm.screenOf(3)!
    expect(bottom.y).toBeLessThan(700 - 300)
    expect(bottom.x).toBeGreaterThan(400) // no longer squeezed left — full width free
  })

  it('never moves a star in world space when the pane insets change', () => {
    // Insets ease the *camera*, not the layout: world positions stay put.
    sm.setModel(fixture())
    const before = sm.positions()
    sm.select(2)
    sm.setInsets({ right: 380 })
    sm.setInsets({ bottom: 260, right: 16 })
    expect(sm.positions()).toEqual(before)
  })

  it('drops a stale selection when its ticket leaves the map', () => {
    sm.setModel(fixture())
    sm.select(5)
    const emitted: (number | null)[] = []
    sm.onSelect((n) => emitted.push(n))
    // #5 is gone from the structure; selecting it again is a no-op, and the map
    // simply no longer holds it.
    sm.setModel(fixture().filter((t) => t.num !== 5))
    sm.select(5)
    expect(emitted).toEqual([])
    expect(sm.screenOf(5)).toBe(null)
  })
})

// The session overlay (ticket 13): the whole lifecycle scripted as pushed models,
// with the two guarantees of ticket 06 still holding underneath it — a session
// state is appearance, never position.
const SESSION_LIFECYCLE: Array<{
  what: SessionState | 'ignition'
  tickets: Ticket[]
  sessions: Record<number, SessionState>
}> = [
  {
    what: 'implementing',
    tickets: beat({ 1: 'resolved', 2: 'resolved', 3: 'claimed' }),
    sessions: { 3: 'implementing' },
  },
  {
    what: 'blocked',
    tickets: beat({ 1: 'resolved', 2: 'resolved', 3: 'claimed' }),
    sessions: { 3: 'blocked' },
  },
  {
    what: 'dead',
    tickets: beat({ 1: 'resolved', 2: 'resolved', 3: 'claimed' }),
    sessions: { 3: 'dead' },
  },
  {
    // The answer lands, the star resolves, and #04 ignites onto the frontier —
    // the base language, with no session overlay left to speak.
    what: 'ignition',
    tickets: beat({ 1: 'resolved', 2: 'resolved', 3: 'resolved' }, [4]),
    sessions: {},
  },
]

describe('the session overlay on the seam', () => {
  it('scripts the full lifecycle without ever moving a star', () => {
    const { sm } = mounted()
    sm.setModel(fixture())
    const start = sm.positions()
    for (const b of SESSION_LIFECYCLE) {
      sm.setModel(b.tickets, b.sessions)
      expect(sm.positions()).toEqual(start)
    }
  })

  it('renders each state per the grammar, and only where a session speaks', () => {
    const { sm } = mounted()
    sm.setModel(fixture())
    for (const b of SESSION_LIFECYCLE) {
      sm.setModel(b.tickets, b.sessions)
      expect(sm.overlays()).toEqual(b.sessions)
      if (b.what !== 'ignition') {
        // Every state the map paints is one the grammar defines — motion or shape
        // carries it, so the star reads without its colour.
        const g = GRAMMAR[b.what]
        expect(g.motion).toBeTruthy()
        expect(nonColorSignature(b.what)).toBeTruthy()
      }
    }
    // The last beat is approval: the moons are gone and the base star speaks.
    expect(sm.overlays()).toEqual({})
  })

  it('writes one fading ticker line per changed push, and none on the first', () => {
    const { sm } = mounted()
    sm.setModel(SESSION_LIFECYCLE[0].tickets, SESSION_LIFECYCLE[0].sessions)
    expect(sm.ticker()).toBe(null) // the first ingest is not a change

    sm.setModel(SESSION_LIFECYCLE[1].tickets, SESSION_LIFECYCLE[1].sessions)
    expect(sm.ticker()).toContain('#03')
    expect(sm.ticker()).toContain('blocked')

    // A push that changes nothing says nothing — the map goes calm again.
    const held = sm.ticker()
    sm.setModel(SESSION_LIFECYCLE[1].tickets, SESSION_LIFECYCLE[1].sessions)
    expect(sm.ticker()).toBe(held)
  })
})

// One frame against a recording stub context. The canvas *feel* can only be
// judged by eye (starmap-design.md, Open risk), but the draw path for every
// session state should at least run, and the one thing the overlay writes as
// text — the ticker line — is assertable.
function stubContext(): { ctx: Record<string, unknown>; texts: string[] } {
  const texts: string[] = []
  const ctx: Record<string, unknown> = {
    createRadialGradient: () => ({ addColorStop: () => {} }),
    measureText: () => ({ width: 40 }),
    fillText: (s: string) => texts.push(s),
  }
  for (const m of [
    'setTransform', 'fillRect', 'beginPath', 'arc', 'fill', 'stroke', 'moveTo', 'lineTo',
    'closePath', 'quadraticCurveTo', 'setLineDash', 'save', 'restore', 'translate', 'scale', 'rotate',
  ]) {
    ctx[m] = () => {}
  }
  return { ctx, texts }
}

describe('painting the overlay', () => {
  let frames: FrameRequestCallback[] = []
  let texts: string[] = []
  const realGetContext = HTMLCanvasElement.prototype.getContext
  const realRaf = globalThis.requestAnimationFrame

  beforeEach(() => {
    frames = []
    const stub = stubContext()
    texts = stub.texts
    HTMLCanvasElement.prototype.getContext = (() => stub.ctx) as never
    globalThis.requestAnimationFrame = ((cb: FrameRequestCallback) => {
      frames.push(cb)
      return frames.length
    }) as never
    globalThis.cancelAnimationFrame = (() => {}) as never
  })
  afterEach(() => {
    HTMLCanvasElement.prototype.getContext = realGetContext
    globalThis.requestAnimationFrame = realRaf
  })

  function frame(): void {
    const cb = frames.pop()
    frames = []
    cb?.(0)
  }

  it('draws every session state without falling over', () => {
    const { sm } = mounted()
    for (const b of SESSION_LIFECYCLE) {
      sm.setModel(b.tickets, b.sessions)
      expect(() => frame()).not.toThrow()
    }
    // The ticker line the last change wrote is on the canvas.
    expect(texts.some((t) => t.startsWith('▸'))).toBe(true)
  })
})
