import { afterEach, describe, expect, it, vi } from 'vitest'
import {
  hasNativeTitleBar,
  nativePageZoom,
  nativeTitleBarLayout,
  trackPageZoom,
  trackTitleBarButtons,
  type TitleBarButtonRect,
} from './titlebar'

class TestResizeObserver {
  observe() {}
  disconnect() {}
}

function rect(left: number, top: number, width: number, height: number): DOMRect {
  return {
    left,
    top,
    right: left + width,
    bottom: top + height,
    width,
    height,
    x: left,
    y: top,
    toJSON: () => ({}),
  }
}

function stubAnimationFrames() {
  let nextId = 0
  const frames = new Map<number, FrameRequestCallback>()
  vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => {
    const id = ++nextId
    frames.set(id, callback)
    return id
  })
  vi.stubGlobal('cancelAnimationFrame', (id: number) => frames.delete(id))
  return {
    flush() {
      const pending = [...frames.values()]
      frames.clear()
      for (const callback of pending) callback(0)
    },
    get size() {
      return frames.size
    },
  }
}

describe('native page zoom', () => {
  afterEach(() => {
    delete window.__chartrPageZoom
  })

  it.each([
    [0.5, 104, 160],
    [1, 52, 80],
    [1.25, 41.6, 64],
    [2, 40, 40],
    [3, 40, 80 / 3],
  ])(
    'keeps the native strip and traffic-light clearance aligned at %sx',
    (zoom, expectedHeight, expectedClearance) => {
      window.__chartrPageZoom = zoom

      expect(nativePageZoom()).toBe(zoom)
      const layout = nativeTitleBarLayout(52)
      expect(layout.height).toBeCloseTo(expectedHeight)
      expect(layout.trafficLightClearance).toBeCloseTo(expectedClearance)
    },
  )

  it.each([undefined, 0, 0.49, 3.01, Number.NaN, Number.POSITIVE_INFINITY])(
    'falls back to actual size for an invalid native value (%s)',
    (zoom) => {
      window.__chartrPageZoom = zoom
      expect(nativePageZoom()).toBe(1)
      expect(nativeTitleBarLayout(52)).toEqual({
        height: 52,
        trafficLightClearance: 80,
      })
    },
  )

  it('reads the authoritative global on zoom events and removes its listener', () => {
    const seen: number[] = []
    window.__chartrPageZoom = 1.25
    const stop = trackPageZoom((zoom) => seen.push(zoom))

    window.__chartrPageZoom = 2
    window.dispatchEvent(new CustomEvent('chartr:page-zoom', { detail: 0.5 }))
    expect(seen).toEqual([1.25, 2])

    stop()
    window.__chartrPageZoom = 3
    window.dispatchEvent(new CustomEvent('chartr:page-zoom', { detail: 3 }))
    expect(seen).toEqual([1.25, 2])
  })
})

describe('native title-bar button tracking', () => {
  afterEach(() => {
    document.body.replaceChildren()
    delete window.__chartrSetTitleBarButtonRects
    delete window.__chartrPageZoom
    vi.unstubAllGlobals()
  })

  it('reports only the exact clickable boxes intersecting the title strip', async () => {
    const frames = stubAnimationFrames()
    vi.stubGlobal('ResizeObserver', TestResizeObserver)
    window.__chartrPageZoom = 1

    const name = document.createElement('button')
    const folder = document.createElement('button')
    const below = document.createElement('button')
    const plain = document.createElement('span')
    document.body.append(name, folder, below, plain)
    vi.spyOn(name, 'getBoundingClientRect').mockReturnValue(rect(264, 8, 120, 24))
    vi.spyOn(folder, 'getBoundingClientRect').mockReturnValue(rect(388, 8, 24, 24))
    vi.spyOn(below, 'getBoundingClientRect').mockReturnValue(rect(8, 48, 200, 28))
    vi.spyOn(plain, 'getBoundingClientRect').mockReturnValue(rect(420, 0, 300, 40))

    const reports: TitleBarButtonRect[][] = []
    window.__chartrSetTitleBarButtonRects = (next) => reports.push(next)
    const stop = trackTitleBarButtons(40)
    frames.flush()
    await Promise.resolve()

    expect(reports).toEqual([[
      { x: 264, y: 8, width: 120, height: 24 },
      { x: 388, y: 8, width: 24, height: 24 },
    ]])
    stop()
  })

  it('clips in CSS pixels and reports rectangles in native coordinates', () => {
    const frames = stubAnimationFrames()
    vi.stubGlobal('ResizeObserver', TestResizeObserver)
    window.__chartrPageZoom = 1.25

    const crossing = document.createElement('button')
    const below = document.createElement('button')
    document.body.append(crossing, below)
    vi.spyOn(crossing, 'getBoundingClientRect').mockReturnValue(rect(10, 24, 20, 16))
    vi.spyOn(below, 'getBoundingClientRect').mockReturnValue(rect(10, 33, 20, 10))

    const reports: TitleBarButtonRect[][] = []
    window.__chartrSetTitleBarButtonRects = (next) => reports.push(next)
    const stop = trackTitleBarButtons(40)
    frames.flush()

    expect(reports).toEqual([[
      { x: 12.5, y: 30, width: 25, height: 10 },
    ]])
    stop()
  })

  it('re-reports at the new zoom and stops observing after cleanup', () => {
    const frames = stubAnimationFrames()
    vi.stubGlobal('ResizeObserver', TestResizeObserver)
    window.__chartrPageZoom = 1

    const button = document.createElement('button')
    document.body.append(button)
    vi.spyOn(button, 'getBoundingClientRect').mockReturnValue(rect(10, 15, 20, 20))

    const reports: TitleBarButtonRect[][] = []
    window.__chartrSetTitleBarButtonRects = (next) => reports.push(next)
    const stop = trackTitleBarButtons(40)
    frames.flush()

    window.__chartrPageZoom = 2
    window.dispatchEvent(new CustomEvent('chartr:page-zoom', { detail: 2 }))
    expect(frames.size).toBe(1)
    frames.flush()

    expect(reports).toEqual([
      [{ x: 10, y: 15, width: 20, height: 20 }],
      [{ x: 20, y: 30, width: 40, height: 10 }],
    ])

    stop()
    expect(reports.at(-1)).toEqual([])
    window.__chartrPageZoom = 3
    window.dispatchEvent(new CustomEvent('chartr:page-zoom', { detail: 3 }))
    expect(frames.size).toBe(0)
  })
})

// Which host the cockpit booted in, from the two globals the shell injects. The
// wordmark hangs off this: it stays wherever the window is not already naming
// the application above the page.
describe('detecting a window whose title bar is the OS\'s', () => {
  afterEach(() => {
    delete window.__chartrNativePlatform
    delete window.__chartrTitleBar
  })

  it('is true in the shells that never took the top strip', () => {
    window.__chartrNativePlatform = 'linux'
    expect(hasNativeTitleBar()).toBe(true)

    window.__chartrNativePlatform = 'windows'
    expect(hasNativeTitleBar()).toBe(true)
  })

  it('is false on macOS, where the strip is ours to draw', () => {
    window.__chartrNativePlatform = 'darwin'
    window.__chartrTitleBar = 52
    expect(hasNativeTitleBar()).toBe(false)
  })

  it('is false in a plain browser tab, whose chrome is not the window\'s', () => {
    expect(hasNativeTitleBar()).toBe(false)
  })
})
