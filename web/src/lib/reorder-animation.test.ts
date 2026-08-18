import { describe, expect, it } from 'vitest'
import { reorderFlip } from './reorder-animation'

function rect(left: number, top: number, width: number, height: number): DOMRect {
  return new DOMRect(left, top, width, height)
}

describe('reorderFlip', () => {
  it('does not move or scale a row whose height alone changed', () => {
    const node = document.createElement('div')
    const animation = reorderFlip(
      node,
      { from: rect(20, 40, 240, 48), to: rect(20, 40, 240, 80) },
      { duration: 120 },
    )

    expect(animation.duration).toBe(0)
    expect(animation.css?.(0, 1)).toBe('transform: translate(0px, 0px);')
    expect(animation.css?.(0.5, 0.5)).not.toContain('scale')
  })

  it('slides a reordered row from its old position without resizing it', () => {
    const node = document.createElement('div')
    const animation = reorderFlip(
      node,
      { from: rect(20, 40, 240, 80), to: rect(20, 120, 240, 48) },
      { duration: 120 },
    )

    expect(animation.css?.(0, 1)).toBe('transform: translate(0px, -80px);')
    expect(animation.css?.(1, 0)).toBe('transform: translate(0px, 0px);')
    expect(animation.css?.(0.5, 0.5)).not.toContain('scale')
  })

  it('keeps the existing transform while the position animation runs', () => {
    const node = document.createElement('div')
    node.style.transform = 'translateX(2px)'
    document.body.append(node)

    const animation = reorderFlip(
      node,
      { from: rect(20, 40, 240, 48), to: rect(28, 40, 240, 48) },
      { duration: 120 },
    )

    expect(animation.css?.(0, 1)).toBe(
      'transform: translateX(2px) translate(-8px, 0px);',
    )
    node.remove()
  })
})
