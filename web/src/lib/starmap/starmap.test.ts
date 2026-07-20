// The star-map island's seam is the one frontend test point (spec, Testing
// Decisions): mount, receive the model, emit selection. These tests pin the
// renderer's two binding guarantees — deterministic layout from ticket data, and
// zero star movement across the full lifecycle — and the selection emission.
// The canvas *feel* is not tested here (it can only be judged by eye); the
// island runs headless, so no 2D context is required.

import { describe, it, expect, beforeEach } from 'vitest'
import { StarMap } from './starmap'
import { computeLayout, structureSignature } from './layout'
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
  { 1: 'resolved', 2: 'resolved', 3: 'proposed' },
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

  it('renders all six base states without a 2D context', () => {
    // open→frontier, open→blocked, claimed, proposed, resolved, out_of_scope —
    // one push carrying all six, and every star present in the model.
    sm.setModel([
      { num: 1, slug: '1', title: 'frontier', type: 'task', status: 'open', frontier: true, blockedBy: [] },
      { num: 2, slug: '2', title: 'blocked', type: 'task', status: 'open', frontier: false, blockedBy: [1] },
      { num: 3, slug: '3', title: 'claimed', type: 'task', status: 'claimed', frontier: false, blockedBy: [] },
      { num: 4, slug: '4', title: 'proposed', type: 'task', status: 'proposed', frontier: false, blockedBy: [] },
      { num: 5, slug: '5', title: 'resolved', type: 'task', status: 'resolved', frontier: false, blockedBy: [] },
      { num: 6, slug: '6', title: 'oos', type: 'task', status: 'out_of_scope', frontier: false, blockedBy: [] },
    ])
    expect(Object.keys(sm.positions()).sort()).toEqual(['1', '2', '3', '4', '5', '6'])
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
