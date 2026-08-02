import { describe, expect, it } from 'vitest'
import { dropIndex, dropTarget, reorder, type RowBox } from './reorder'

const ids = ['a', 'b', 'c', 'd']

// Four 40px rows with an 8px gutter between them, the sidebar's own geometry
// (`gap-2` on the list), starting 100px down the viewport. The gutters matter:
// the drop indicator is drawn in them, so they are what the operator aims at.
const rows: RowBox[] = [
  { id: 'a', top: 100, bottom: 140 },
  { id: 'b', top: 148, bottom: 188 },
  { id: 'c', top: 196, bottom: 236 },
  { id: 'd', top: 244, bottom: 284 },
]

describe('reorder', () => {
  it('moves a row to the top', () => {
    expect(reorder(ids, 2, 0)).toEqual(['c', 'a', 'b', 'd'])
  })

  it('moves a row to the bottom', () => {
    expect(reorder(ids, 0, 3)).toEqual(['b', 'c', 'd', 'a'])
  })

  it('moves a row one step either way', () => {
    expect(reorder(ids, 1, 0)).toEqual(['b', 'a', 'c', 'd'])
    expect(reorder(ids, 1, 2)).toEqual(['a', 'c', 'b', 'd'])
  })

  it('returns the whole list every time, so the caller can always post it', () => {
    expect(reorder(ids, 2, 2)).toEqual(ids)
    expect(reorder(ids, 2, 2)).not.toBe(ids)
    expect(reorder([], 0, 0)).toEqual([])
    expect(reorder(['a'], 0, 0)).toEqual(['a'])
  })

  it('never mutates the list it was given', () => {
    const before = ids.slice()
    reorder(ids, 0, 3)
    expect(ids).toEqual(before)
  })

  it('clamps a nudge past either end rather than dropping the row', () => {
    // ⌥↑ on the first row and ⌥↓ on the last: an ordinary keypress, and the
    // right answer is that nothing moves.
    expect(reorder(ids, 0, -1)).toEqual(ids)
    expect(reorder(ids, 3, 4)).toEqual(ids)
  })

  it('ignores a source index that is not a row', () => {
    expect(reorder(ids, -1, 0)).toEqual(ids)
    expect(reorder(ids, 4, 0)).toEqual(ids)
  })
})

describe('dropIndex', () => {
  it('reads a drop above a row as that row’s place', () => {
    expect(dropIndex(3, 0, 'before')).toBe(0)
    expect(dropIndex(3, 1, 'before')).toBe(1)
  })

  it('shifts a downward drag up by one, the row itself having left', () => {
    // Drag 'a' below 'd': the insertion point is 4, but 'a' is removed first, so
    // it lands at 3 — the end of the list, not past it.
    expect(dropIndex(0, 3, 'after')).toBe(3)
    expect(dropIndex(0, 1, 'after')).toBe(1)
  })

  it('resolves a drop on either of the row’s own edges to no move', () => {
    expect(dropIndex(2, 2, 'before')).toBe(2)
    expect(dropIndex(2, 2, 'after')).toBe(2)
    // And the same for the gap on the far side of each neighbour, which is the
    // same gap: below the row above, and above the row below.
    expect(dropIndex(2, 1, 'after')).toBe(2)
    expect(dropIndex(2, 3, 'before')).toBe(2)
  })

  it('composes with reorder into the sequence the drop describes', () => {
    const move = (from: number, over: number, edge: 'before' | 'after') =>
      reorder(ids, from, dropIndex(from, over, edge))

    expect(move(3, 0, 'before')).toEqual(['d', 'a', 'b', 'c'])
    expect(move(0, 3, 'after')).toEqual(['b', 'c', 'd', 'a'])
    expect(move(1, 2, 'after')).toEqual(['a', 'c', 'b', 'd'])
    expect(move(2, 1, 'before')).toEqual(['a', 'c', 'b', 'd'])
    expect(move(1, 1, 'after')).toEqual(ids)
  })
})

describe('dropTarget', () => {
  it('reads the half of the row the pointer is in', () => {
    expect(dropTarget(rows, 110)).toEqual({ overId: 'a', edge: 'before' })
    expect(dropTarget(rows, 130)).toEqual({ overId: 'a', edge: 'after' })
    expect(dropTarget(rows, 200)).toEqual({ overId: 'c', edge: 'before' })
    expect(dropTarget(rows, 230)).toEqual({ overId: 'c', edge: 'after' })
  })

  it('lands a release in the gutter between two rows on that seam', () => {
    // The gutter is where the drop indicator is drawn, so it must not be the one
    // strip that rejects a drop — which is exactly what it was under HTML5
    // drag-and-drop, the rows being the only elements that could accept one.
    for (const y of [141, 144, 147]) {
      expect(dropTarget(rows, y)).toEqual({ overId: 'b', edge: 'before' })
    }
    // And that is the same position as the trailing edge of the row above: both
    // describe the gap between a and b.
    expect(dropIndex(0, 1, 'before')).toBe(dropIndex(0, 0, 'after'))
  })

  it('lands a release past either end of the list on the nearest end', () => {
    // Dragged clear above the sidebar, or below the last row into the empty
    // space under the list — including a pointer that has left the window, whose
    // Y the browser still reports (and reports out of range).
    expect(dropTarget(rows, 40)).toEqual({ overId: 'a', edge: 'before' })
    expect(dropTarget(rows, -600)).toEqual({ overId: 'a', edge: 'before' })
    expect(dropTarget(rows, 300)).toEqual({ overId: 'd', edge: 'after' })
    expect(dropTarget(rows, 5000)).toEqual({ overId: 'd', edge: 'after' })
  })

  it('has nowhere to land when the dragged row was the only one', () => {
    expect(dropTarget([], 120)).toBeNull()
  })

  it('resolves a drag of the first row to the fourth position', () => {
    // The operator's own report: grab the top row, pull it down past the last
    // one, let go somewhere out of bounds. The rows given exclude the dragged
    // row, which is riding the pointer and is a reference for nothing.
    const others = rows.filter((r) => r.id !== 'a')
    const at = dropTarget(others, 9999)!
    expect(at).toEqual({ overId: 'd', edge: 'after' })

    const from = ids.indexOf('a')
    const over = ids.indexOf(at.overId)
    expect(reorder(ids, from, dropIndex(from, over, at.edge))).toEqual(['b', 'c', 'd', 'a'])
  })
})
