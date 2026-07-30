// Where a dragged space lands, as pure index arithmetic over the sidebar's id
// list. The endpoint takes the complete ordered list (`POST /api/spaces/reorder`),
// so every way of moving a row — the pointer drag, the keyboard — ends here: it
// resolves to one new id sequence and posts that. Keeping the arithmetic out of
// the component is what makes the behaviour testable without driving a drag.

// DropEdge is which half of a row the pointer is over: the dragged space lands
// above it or below it. Half-height is the whole rule — the row the pointer is
// on is never the answer on its own, because "on b" means above b coming down
// and below b coming up.
export type DropEdge = 'before' | 'after'

/**
 * Move the id at `from` so that it sits at index `to` in the returned list.
 *
 * `to` is a position in the *result*, not an insertion point in the input, which
 * is what makes it the natural argument for the keyboard (`i - 1` / `i + 1`) and
 * what `dropIndex` converts a drop into. It is clamped rather than refused: a
 * nudge past either end is an ordinary keypress on the first or last row, and
 * the right answer is that nothing moves.
 *
 * The input is never mutated, and an out-of-range `from` yields an unchanged
 * copy — a reorder always produces a whole list, so a caller can post the result
 * without checking whether anything happened.
 */
export function reorder(ids: string[], from: number, to: number): string[] {
  const out = ids.slice()
  if (from < 0 || from >= out.length) return out
  const dest = Math.min(Math.max(to, 0), out.length - 1)
  if (dest === from) return out
  const [moved] = out.splice(from, 1)
  out.splice(dest, 0, moved)
  return out
}

/**
 * Convert a drop — dragging the row at `from`, released on the `edge` of the row
 * at `over` — into the destination index `reorder` takes.
 *
 * The two differ whenever the row is dragged downwards: removing it first shifts
 * everything below up by one, so an insertion point of 3 in a list of three is a
 * final index of 2. Dropping a row on either of its own edges resolves to its own
 * index, so releasing where you started is a no-op rather than an off-by-one.
 */
export function dropIndex(from: number, over: number, edge: DropEdge): number {
  const insertAt = edge === 'after' ? over + 1 : over
  return insertAt > from ? insertAt - 1 : insertAt
}
