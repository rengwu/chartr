import { describe, expect, it } from 'vitest'
import { dropIndex, reorder } from './reorder'

const ids = ['a', 'b', 'c', 'd']

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
