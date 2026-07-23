import { describe, expect, it, vi } from 'vitest'
import { openExternal } from './external'

// The open-target decision is the whole unit here: hook present → hook called,
// hook absent → a new browser tab. Neither the web-links addon (inside the
// island) nor the shell's binding (native, needs a real window) is testable
// here, so the seam between them is what the tests pin.

// A stand-in for `window` carrying only what the decision reads. `openExternal`
// takes it as a parameter precisely so a test never needs a real one.
function fakeWindow(hook?: (url: string) => unknown) {
  return {
    __chartrOpenExternal: hook,
    open: vi.fn(),
  } as unknown as Window & { open: ReturnType<typeof vi.fn> }
}

describe('openExternal', () => {
  it('calls the shell hook when the native shell exposes one', () => {
    const hook = vi.fn()
    const win = fakeWindow(hook)

    expect(openExternal('https://example.com/docs', win)).toBe('shell')
    expect(hook).toHaveBeenCalledWith('https://example.com/docs')
    expect(win.open).not.toHaveBeenCalled()
  })

  it('opens a new browser tab when the hook is absent', () => {
    const win = fakeWindow()

    expect(openExternal('https://example.com/docs', win)).toBe('browser')
    expect(win.open).toHaveBeenCalledWith(
      'https://example.com/docs',
      '_blank',
      'noopener,noreferrer',
    )
  })

  it('falls back when the global is present but not callable', () => {
    const win = fakeWindow('nonsense' as unknown as () => void)

    expect(openExternal('http://localhost:8787/', win)).toBe('browser')
    expect(win.open).toHaveBeenCalled()
  })

  it('opens plain http as well as https', () => {
    const hook = vi.fn()
    expect(openExternal('http://127.0.0.1:3000/x', fakeWindow(hook))).toBe('shell')
    expect(hook).toHaveBeenCalledWith('http://127.0.0.1:3000/x')
  })

  it('blocks a non-http scheme rather than handing it to either path', () => {
    for (const url of ['file:///etc/passwd', 'javascript:alert(1)', 'not a url']) {
      const hook = vi.fn()
      const win = fakeWindow(hook)
      expect(openExternal(url, win)).toBe('blocked')
      expect(hook).not.toHaveBeenCalled()
      expect(win.open).not.toHaveBeenCalled()
    }
  })
})
