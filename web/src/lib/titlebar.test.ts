import { afterEach, describe, expect, it, vi } from 'vitest'
import { hasNativeTitleBar, trackTitleBarButtons, type TitleBarButtonRect } from './titlebar'

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

describe('native title-bar button tracking', () => {
  afterEach(() => {
    document.body.replaceChildren()
    delete window.__chartrSetTitleBarButtonRects
    vi.unstubAllGlobals()
  })

  it('reports only the exact clickable boxes intersecting the title strip', async () => {
    const frames: FrameRequestCallback[] = []
    vi.stubGlobal('ResizeObserver', TestResizeObserver)
    vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => {
      frames.push(callback)
      return frames.length
    })
    vi.stubGlobal('cancelAnimationFrame', vi.fn())

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
    frames.shift()?.(0)
    await Promise.resolve()

    expect(reports).toEqual([[
      { x: 264, y: 8, width: 120, height: 24 },
      { x: 388, y: 8, width: 24, height: 24 },
    ]])
    stop()
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
