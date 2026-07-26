// The star-map island's seam is the one frontend test point (spec, Testing
// Decisions): mount, receive the model, emit selection. These tests pin the
// renderer's two binding guarantees — deterministic layout from ticket data, and
// zero star movement across the full lifecycle — and the selection emission.
// The canvas *feel* is not tested here (it can only be judged by eye); the
// island runs headless, so no 2D context is required.

import { describe, it, expect, afterEach, beforeEach, vi } from 'vitest'
import { StarMap, titleBudget, clipTitle } from './starmap'
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

// Zoomed out, stars crowd and the same words need more room than exists, so a
// label's title is rationed before the label itself is given up.
describe('rationing the title as the camera pulls back', () => {
  it('ramps the budget with scale, and clamps at both ends', () => {
    const wide = titleBudget(2)
    const tight = titleBudget(0.42)
    expect(tight).toBeLessThan(wide)
    // Monotonic across the ramp — no band where pulling back buys more title.
    let prev = 0
    for (let s = 0.42; s <= 1.2; s += 0.02) {
      const b = titleBudget(s)
      expect(b).toBeGreaterThanOrEqual(prev)
      prev = b
    }
    // Clamped: past the top of the ramp the budget stops growing.
    expect(titleBudget(10)).toBe(wide)
    expect(titleBudget(0.1)).toBe(tight)
  })

  it('clips on a word boundary when one is near the cut', () => {
    expect(clipTitle('The agent adapter contract', 12)).toBe('The agent…')
    // No usable boundary near the cut — fall back to a hard slice.
    expect(clipTitle('Supercalifragilistic', 12)).toBe('Supercalifra…')
    // Inside budget, the title is left exactly alone.
    expect(clipTitle('Short one', 12)).toBe('Short one')
  })

  it('never exceeds its budget by more than the ellipsis', () => {
    const titles = [
      'The filtering seam and the streaming CSV download',
      'Per-date historical conversion and rate backfill',
      'Docs and architecture catch up with what shipped',
    ]
    for (const t of titles) {
      for (let b = 8; b <= 60; b++) {
        expect(clipTitle(t, b).length).toBeLessThanOrEqual(b + 1)
      }
    }
  })
})

// Label placement is the one part of the drawing the eye judges but arithmetic
// can pin: a label may not sit on a star, and two labels may not sit on each
// other. The solver is allowed to drop a label it cannot fit — what it is not
// allowed to do is draw one on top of something.
describe('label placement', () => {
  const realGetContext = HTMLCanvasElement.prototype.getContext
  const realRaf = globalThis.requestAnimationFrame

  // A wide fan-out on one root: every child lands at a similar rank with a long
  // title, which is exactly where a vertical-only stack runs out of slots and
  // starts dropping labels onto stars.
  const CROWDED: Ticket[] = [
    { num: 1, slug: '1', title: 'The root everything hangs from', type: 'task', status: 'open', blockedBy: [], frontier: true },
    ...Array.from({ length: 12 }, (_, i) => ({
      num: i + 2,
      slug: `${i + 2}`,
      title: `A dependent with a fairly long title ${i + 2}`,
      type: 'task',
      status: 'open' as const,
      blockedBy: [1],
      frontier: false,
    })),
  ]

  // Like stubContext, but recording where each string landed and how it was
  // aligned, so the drawn boxes can be reconstructed.
  function recordingContext() {
    const drawn: { text: string; x: number; y: number; align: string; font: string }[] = []
    const ctx: Record<string, unknown> = {
      textAlign: 'center',
      font: '',
      createRadialGradient: () => ({ addColorStop: () => {} }),
      measureText: (s: string) => ({ width: s.length * 6 }),
      fillText(this: Record<string, unknown>, text: string, x: number, y: number) {
        drawn.push({
          text,
          x,
          y,
          align: this.textAlign as string,
          font: this.font as string,
        })
      },
    }
    for (const m of [
      'setTransform', 'fillRect', 'beginPath', 'arc', 'fill', 'stroke', 'moveTo', 'lineTo',
      'closePath', 'quadraticCurveTo', 'setLineDash', 'save', 'restore', 'translate', 'scale', 'rotate',
    ]) {
      ctx[m] = () => {}
    }
    return { ctx, drawn }
  }

  afterEach(() => {
    HTMLCanvasElement.prototype.getContext = realGetContext
    globalThis.requestAnimationFrame = realRaf
  })

  function place(tickets: Ticket[], sessions: Record<number, SessionState> = {}) {
    const { ctx, drawn } = recordingContext()
    let frames: FrameRequestCallback[] = []
    HTMLCanvasElement.prototype.getContext = (() => ctx) as never
    globalThis.requestAnimationFrame = ((cb: FrameRequestCallback) => {
      frames.push(cb)
      return frames.length
    }) as never
    globalThis.cancelAnimationFrame = (() => {}) as never

    const { sm } = mounted()
    sm.setModel(tickets, sessions)
    const cb = frames.pop()
    frames = []
    cb?.(0)

    // The ticker writes through the same fillText; it is chrome, not a label.
    const labels = drawn.filter((d) => !d.text.startsWith('▸'))
    const fs = parseFloat(labels[0]?.font ?? '11')
    const boxes = labels.map((d) => {
      const w = d.text.length * 6
      const left = d.align === 'center' ? d.x - w / 2 : d.align === 'left' ? d.x : d.x - w
      return {
        text: d.text,
        x0: left,
        y0: d.y - fs * 0.82,
        x1: left + w,
        y1: d.y + fs * 0.22,
      }
    })
    return { sm, labels, boxes }
  }

  const overlaps = (a: { x0: number; y0: number; x1: number; y1: number },
                    b: { x0: number; y0: number; x1: number; y1: number }) =>
    a.x0 < b.x1 && b.x0 < a.x1 && a.y0 < b.y1 && b.y0 < a.y1

  it('never draws a label across a star', () => {
    const tickets = CROWDED
    const { sm, boxes } = place(tickets)
    expect(boxes.length).toBeGreaterThan(0)

    // Reconstruct each star's screen disc from the seam, at the scale the
    // camera settled on.
    const p = sm.positions()
    const sa = sm.screenOf(1)!,
      sb = sm.screenOf(5)!
    const world = Math.hypot(p[5].x - p[1].x, p[5].y - p[1].y)
    const s = Math.hypot(sb.x - sa.x, sb.y - sa.y) / world

    for (const t of tickets) {
      const c = sm.screenOf(t.num)!
      // The smallest core radius in the palette — a conservative disc, so the
      // assertion cannot pass by claiming the star is tiny.
      const r = 4.5 * s
      const star = { x0: c.x - r, y0: c.y - r, x1: c.x + r, y1: c.y + r }
      for (const b of boxes) {
        expect(overlaps(b, star), `label "${b.text}" sits on star #${t.num}`).toBe(false)
      }
    }
  })

  it('never draws two labels across each other', () => {
    const { boxes } = place(CROWDED)
    for (let i = 0; i < boxes.length; i++) {
      for (let j = i + 1; j < boxes.length; j++) {
        expect(
          overlaps(boxes[i], boxes[j]),
          `"${boxes[i].text}" overlaps "${boxes[j].text}"`,
        ).toBe(false)
      }
    }
  })

  it('keeps the selected star labelled when the map is crowded', () => {
    const { ctx, drawn } = recordingContext()
    let frames: FrameRequestCallback[] = []
    HTMLCanvasElement.prototype.getContext = (() => ctx) as never
    globalThis.requestAnimationFrame = ((cb: FrameRequestCallback) => {
      frames.push(cb)
      return frames.length
    }) as never
    globalThis.cancelAnimationFrame = (() => {}) as never

    const { sm, host } = mounted()
    sm.setModel(CROWDED)
    const canvas = host.querySelector('canvas')!
    const at = sm.screenOf(7)!
    canvas.dispatchEvent(new MouseEvent('mousedown', { clientX: at.x, clientY: at.y, bubbles: true }))
    canvas.dispatchEvent(new MouseEvent('mouseup', { clientX: at.x, clientY: at.y, bubbles: true }))
    drawn.length = 0
    const cb = frames.pop()
    frames = []
    cb?.(0)

    // Crowding may cost some labels, but never the one the operator picked.
    expect(drawn.some((d) => d.text.startsWith('07'))).toBe(true)
  })
})

// How the camera *feels* is judged by eye, but what it feels like rests on three
// things that are exactly assertable: a zoom keeps the world pinned under the
// cursor, the ease runs on wall-clock time rather than on frames, and the input
// devices that mean "zoom" all arrive in the same units. Those need a live
// context — the ease only runs inside the render loop, and headless the camera
// settles instantly — plus a clock the test drives.
//
// The camera is read straight back off the seam: two stars' world positions and
// their screen positions pin the scale and translation exactly, no test hatch.
function camera(sm: StarMap): { s: number; x: number; y: number } {
  const p = sm.positions()
  const [a, b] = [1, 5]
  const world = Math.hypot(p[b].x - p[a].x, p[b].y - p[a].y)
  const sa = sm.screenOf(a)!,
    sb = sm.screenOf(b)!
  const s = Math.hypot(sb.x - sa.x, sb.y - sa.y) / world
  return { s, x: sa.x - p[a].x * s, y: sa.y - p[a].y * s }
}

// The world point sitting under a screen point — the thing a zoom must not move.
function worldAt(cam: { s: number; x: number; y: number }, sx: number, sy: number) {
  return { x: (sx - cam.x) / cam.s, y: (sy - cam.y) / cam.s }
}

describe('the camera', () => {
  let frames: FrameRequestCallback[] = []
  const realGetContext = HTMLCanvasElement.prototype.getContext
  const realRaf = globalThis.requestAnimationFrame

  beforeEach(() => {
    frames = []
    const stub = stubContext()
    HTMLCanvasElement.prototype.getContext = (() => stub.ctx) as never
    globalThis.requestAnimationFrame = ((cb: FrameRequestCallback) => {
      frames.push(cb)
      return frames.length
    }) as never
    globalThis.cancelAnimationFrame = (() => {}) as never
    // The island reads performance.now(); faking it puts the frame rate under the
    // test's control, which is the whole point of two of these tests.
    vi.useFakeTimers({ toFake: ['performance'] })
  })
  afterEach(() => {
    vi.useRealTimers()
    HTMLCanvasElement.prototype.getContext = realGetContext
    globalThis.requestAnimationFrame = realRaf
  })

  // Advance the clock by `ms` and run one frame at that instant.
  function step(ms: number): void {
    vi.advanceTimersByTime(ms)
    const cb = frames.pop()
    frames = []
    cb?.(0)
  }
  // Long enough for any ease to be not just visually over but inside the snap
  // epsilon, so the camera is exactly on its goal rather than a hair short of it.
  function settle(): void {
    for (let i = 0; i < 150; i++) step(16)
  }
  function live(): { sm: StarMap; canvas: HTMLCanvasElement } {
    const { sm, host } = mounted()
    sm.setModel(fixture())
    return { sm, canvas: host.querySelector('canvas')! }
  }
  function wheel(canvas: HTMLCanvasElement, init: WheelEventInit): void {
    canvas.dispatchEvent(new WheelEvent('wheel', { cancelable: true, ...init }))
  }
  // WebKit's pinch event, which no DOM lib types and jsdom does not construct.
  function gesture(canvas: HTMLCanvasElement, type: string, scale: number): void {
    const e = new Event(type, { cancelable: true })
    Object.assign(e, { scale, clientX: 500, clientY: 350 })
    canvas.dispatchEvent(e)
  }

  it('holds the world still under the cursor for the whole flight of a zoom', () => {
    const { sm, canvas } = live()
    const anchor = { x: 700, y: 200 }
    const pinned = worldAt(camera(sm), anchor.x, anchor.y)
    const before = camera(sm).s

    wheel(canvas, { deltaY: -240, clientX: anchor.x, clientY: anchor.y })
    // Not just where it lands — every frame in between. Easing the camera's own
    // translate and scale would let the map slide out from under the cursor for
    // the whole transit and only line up at the end.
    for (let i = 0; i < 40; i++) {
      step(16)
      const w = worldAt(camera(sm), anchor.x, anchor.y)
      expect(Math.hypot(w.x - pinned.x, w.y - pinned.y)).toBeLessThan(0.05)
    }
    expect(camera(sm).s).toBeGreaterThan(before * 1.2)
  })

  it('eases on the clock, not on the frame, so a 120Hz panel is not twice as fast', () => {
    const run = (stepMs: number, steps: number) => {
      const { sm, canvas } = live()
      wheel(canvas, { deltaY: -240, clientX: 700, clientY: 200 })
      for (let i = 0; i < steps; i++) step(stepMs)
      const cam = camera(sm)
      sm.destroy()
      return cam
    }
    // The same 160ms of wall clock, sampled at 60Hz and at 120Hz.
    const slow = run(16, 10)
    const fast = run(8, 20)
    expect(fast.s).toBeCloseTo(slow.s, 4)
    expect(fast.x).toBeCloseTo(slow.x, 3)
    expect(fast.y).toBeCloseTo(slow.y, 3)
    // And it is genuinely still in flight, not a comparison of two settled cameras.
    const { sm, canvas } = live()
    const settled = camera(sm)
    wheel(canvas, { deltaY: -240, clientX: 700, clientY: 200 })
    settle()
    expect(camera(sm).s).toBeGreaterThan(slow.s * 1.02)
    expect(slow.s).toBeGreaterThan(settled.s)
  })

  it('trails a drag and glides in behind it, landing exactly where it was put', () => {
    const { sm, canvas } = live()
    const start = camera(sm).x
    canvas.dispatchEvent(new MouseEvent('mousedown', { clientX: 0, clientY: 0 }))
    window.dispatchEvent(new MouseEvent('mousemove', { clientX: 300, clientY: 0 }))

    // The map is behind the cursor while the hand is moving — that lag *is* the
    // ease — but it is already on its way.
    step(16)
    const trailing = camera(sm).x - start
    expect(trailing).toBeGreaterThan(0)
    expect(trailing).toBeLessThan(300)

    // The release adds nothing of its own: no throw, no coast, no overshoot. The
    // camera simply finishes the trip the drag asked for.
    window.dispatchEvent(new MouseEvent('mouseup', { clientX: 300, clientY: 0 }))
    const closing = camera(sm).x - start
    expect(closing).toBeCloseTo(trailing, 6)

    step(16)
    expect(camera(sm).x - start).toBeGreaterThan(trailing)
    settle()
    expect(camera(sm).x - start).toBeCloseTo(300, 6)
    // And it stays put once it arrives.
    settle()
    expect(camera(sm).x - start).toBeCloseTo(300, 6)
  })

  it('reads a notched wheel, a trackpad scroll and a pinch in the same units', () => {
    // A wheel that counts lines and one that counts pixels mean the same thing.
    const lines = live()
    wheel(lines.canvas, { deltaY: -3, deltaMode: 1, clientX: 500, clientY: 350 })
    settle()
    const pixels = live()
    wheel(pixels.canvas, { deltaY: -48, deltaMode: 0, clientX: 500, clientY: 350 })
    settle()
    expect(camera(lines.sm).s).toBeCloseTo(camera(pixels.sm).s, 6)

    // A pinch is a ctrl-flagged wheel in Chrome and Firefox, and its deltas are a
    // fraction of a scroll's — so it earns a gain of its own, or a pinch barely
    // moves the map.
    const scroll = live()
    wheel(scroll.canvas, { deltaY: -10, clientX: 500, clientY: 350 })
    settle()
    const pinch = live()
    wheel(pinch.canvas, { deltaY: -10, ctrlKey: true, clientX: 500, clientY: 350 })
    settle()
    expect(camera(pinch.sm).s).toBeGreaterThan(camera(scroll.sm).s * 1.05)
  })

  it('zooms once for a WebKit pinch, however many ways the browser reports it', () => {
    const { sm, canvas } = live()
    const before = camera(sm).s

    gesture(canvas, 'gesturestart', 1)
    gesture(canvas, 'gesturechange', 1.5)
    // Safari has been known to report the same pinch as a ctrl-wheel as well; the
    // wheel path stands down while a gesture is live so the map zooms once.
    wheel(canvas, { deltaY: -40, ctrlKey: true, clientX: 500, clientY: 350 })
    gesture(canvas, 'gestureend', 1.5)
    settle()
    expect(camera(sm).s).toBeCloseTo(before * 1.5, 4)
  })
})
