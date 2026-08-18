import type { AnimationConfig, FlipParams } from 'svelte/animate'
import { cubicOut } from 'svelte/easing'

/**
 * FLIP for sortable rows that may also change height.
 *
 * Svelte's built-in `flip` scales an element when its dimensions change. That
 * is useful for morphing a box, but a sortable row's contents should not squash
 * when a child is added or removed. This variant animates position only: a real
 * reorder still slides into place, while a height-only update produces no
 * transform and leaves the row's heading anchored.
 */
export function reorderFlip(
  node: Element,
  { from, to }: { from: DOMRect; to: DOMRect },
  params: FlipParams = {},
): AnimationConfig {
  const { delay = 0, duration = 120, easing = cubicOut } = params
  const computedTransform = getComputedStyle(node).transform
  const transform = computedTransform && computedTransform !== 'none' ? computedTransform : ''
  const dx = from.left - to.left
  const dy = from.top - to.top
  const distance = Math.hypot(dx, dy)

  return {
    delay,
    // A session opening or closing changes only this wrapper's height. Do not
    // even schedule an invisible animation for that update; motion is reserved
    // for an actual change of position.
    duration:
      distance === 0 ? 0 : typeof duration === 'function' ? duration(distance) : duration,
    easing,
    css: (_t, u) =>
      `transform: ${transform}${transform ? ' ' : ''}translate(${u * dx}px, ${u * dy}px);`,
  }
}
