import { describe, it, expect } from 'vitest'
import {
  decideDock,
  dockByWidth,
  dockByAspect,
  dockHybrid,
  WIDTH_THRESHOLD,
  ASPECT_RATIO,
  WIDTH_BAND,
  ASPECT_BAND,
} from './dock'

describe('dock strategies (no hysteresis)', () => {
  it('width: bottom only below the width threshold', () => {
    expect(dockByWidth(WIDTH_THRESHOLD - 1)).toBe('bottom')
    expect(dockByWidth(WIDTH_THRESHOLD + 1)).toBe('right')
  })

  it('aspect: bottom only when portrait', () => {
    expect(dockByAspect(800, 400)).toBe('right') // landscape
    expect(dockByAspect(400, 900)).toBe('bottom') // portrait
  })

  it('the strategies disagree exactly where each is known to fail', () => {
    // Tall, mid-wide: width ribbons the map (right), aspect/hybrid stack it.
    expect(dockByWidth(700)).toBe('right')
    expect(dockByAspect(700, 1200)).toBe('bottom')
    expect(dockHybrid(700, 1200)).toBe('bottom')

    // Small landscape: aspect slivers the map (right), width/hybrid stack it.
    expect(dockByAspect(500, 300)).toBe('right')
    expect(dockByWidth(500)).toBe('bottom')
    expect(dockHybrid(500, 300)).toBe('bottom')

    // Large landscape: all agree on right.
    expect(dockByWidth(900)).toBe('right')
    expect(dockByAspect(900, 700)).toBe('right')
    expect(dockHybrid(900, 700)).toBe('right')
  })
})

describe('hysteresis dead-band', () => {
  it('width: holds the current dock within the band, flips outside it', () => {
    const t = WIDTH_THRESHOLD
    // Sitting just inside the band keeps whatever was in force.
    expect(decideDock('width', t + WIDTH_BAND - 5, 800, 'bottom', true)).toBe('bottom')
    expect(decideDock('width', t - WIDTH_BAND + 5, 800, 'right', true)).toBe('right')
    // Clearing the band flips.
    expect(decideDock('width', t + WIDTH_BAND + 5, 800, 'bottom', true)).toBe('right')
    expect(decideDock('width', t - WIDTH_BAND - 5, 800, 'right', true)).toBe('bottom')
  })

  it('aspect: holds within the ratio band', () => {
    const w = 600
    const near = (r: number) => w * r
    expect(decideDock('aspect', w, near(ASPECT_RATIO + ASPECT_BAND - 0.02), 'bottom', true)).toBe('bottom')
    expect(decideDock('aspect', w, near(ASPECT_RATIO - ASPECT_BAND + 0.02), 'right', true)).toBe('right')
    expect(decideDock('aspect', w, near(ASPECT_RATIO + ASPECT_BAND + 0.05), 'right', true)).toBe('bottom')
  })

  it('hybrid: leaving bottom needs both signals clear; entering needs either', () => {
    const wideEnough = WIDTH_THRESHOLD + WIDTH_BAND + 20
    const landscape = wideEnough * (ASPECT_RATIO - ASPECT_BAND - 0.1)
    // Both clearly on the right side → leaves bottom.
    expect(decideDock('hybrid', wideEnough, landscape, 'bottom', true)).toBe('right')
    // Wide enough but still portrait → one signal holds it bottom.
    expect(decideDock('hybrid', wideEnough, wideEnough * (ASPECT_RATIO + 0.2), 'bottom', true)).toBe('bottom')
  })

  it('with hysteresis off, matches the raw strategy regardless of prev', () => {
    expect(decideDock('hybrid', 700, 1200, 'right', false)).toBe(dockHybrid(700, 1200))
    expect(decideDock('width', 500, 300, 'right', false)).toBe(dockByWidth(500))
  })

  it('degenerate sizes keep the previous dock', () => {
    expect(decideDock('hybrid', 0, 0, 'bottom', true)).toBe('bottom')
    expect(decideDock('width', -5, 100, 'right', true)).toBe('right')
  })
})
