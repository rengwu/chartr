// How the detail pane chooses its side — right or bottom — from the card body's
// dimensions (ticket 07). Three strategies are offered so the feel can be
// compared directly, each optionally wrapped in a dead-band so dragging the card
// through the boundary doesn't flip-flop the dock jarringly.
//
//   width   — bottom when the card is narrower than a side-by-side needs. Simple
//             and predictable; ribbons the map on a tall, mid-wide card.
//   aspect  — bottom when the card is portrait, splitting the long axis. Keeps
//             both panes squarish; slivers the map on a small landscape card and
//             needs an absolute floor it doesn't have.
//   hybrid  — bottom when EITHER the card is too narrow OR it is portrait. The
//             width gate handles the dominant narrow case; the aspect guard
//             catches the tall card the width gate would ribbon.

export type Dock = 'right' | 'bottom'
export type DockMethod = 'width' | 'aspect' | 'hybrid'

// The pane needs a comfortable reading width and the map a usable remainder;
// below this combined width, side-by-side leaves both cramped, so stack instead.
export const WIDTH_THRESHOLD = 600
// h/w above this reads as portrait — splitting vertically beats a right ribbon.
export const ASPECT_RATIO = 1.1
// Dead-bands: the signal must cross the threshold by this much to flip the dock.
export const WIDTH_BAND = 32
export const ASPECT_BAND = 0.12

export function dockByWidth(w: number): Dock {
  return w < WIDTH_THRESHOLD ? 'bottom' : 'right'
}

export function dockByAspect(w: number, h: number): Dock {
  if (w <= 0) return 'right'
  return h > w * ASPECT_RATIO ? 'bottom' : 'right'
}

export function dockHybrid(w: number, h: number): Dock {
  if (w <= 0) return 'right'
  return w < WIDTH_THRESHOLD || h > w * ASPECT_RATIO ? 'bottom' : 'right'
}

/**
 * Decide the dock for one method. With `hysteresis`, `prev` is the dock in force
 * and the decision sticks until the signal clears the threshold by its band — so
 * a resize that lingers near the boundary holds the current side instead of
 * oscillating. Pass `prev = null` (or `hysteresis = false`) for the raw call.
 */
export function decideDock(
  method: DockMethod,
  w: number,
  h: number,
  prev: Dock | null,
  hysteresis: boolean,
): Dock {
  if (w <= 0 || h <= 0) return prev ?? 'right'
  if (!hysteresis || prev === null) {
    return method === 'width' ? dockByWidth(w) : method === 'aspect' ? dockByAspect(w, h) : dockHybrid(w, h)
  }

  const r = h / w
  switch (method) {
    case 'width':
      // Narrow → bottom. Widen past t+band to go right; narrow past t-band to go bottom.
      if (prev === 'bottom') return w >= WIDTH_THRESHOLD + WIDTH_BAND ? 'right' : 'bottom'
      return w < WIDTH_THRESHOLD - WIDTH_BAND ? 'bottom' : 'right'
    case 'aspect':
      if (prev === 'bottom') return r <= ASPECT_RATIO - ASPECT_BAND ? 'right' : 'bottom'
      return r > ASPECT_RATIO + ASPECT_BAND ? 'bottom' : 'right'
    case 'hybrid':
      // Leaving bottom needs BOTH signals clearly on the right side; entering
      // bottom needs EITHER clearly on the bottom side.
      if (prev === 'bottom') {
        const stay = w < WIDTH_THRESHOLD + WIDTH_BAND || r > ASPECT_RATIO - ASPECT_BAND
        return stay ? 'bottom' : 'right'
      }
      const go = w < WIDTH_THRESHOLD - WIDTH_BAND || r > ASPECT_RATIO + ASPECT_BAND
      return go ? 'bottom' : 'right'
  }
}
