// Opening a link outside the cockpit.
//
// The cockpit runs in two hosts — a plain browser tab and the native webview
// shell (ADR 0013) — and "open this URL" means something different in each. A
// browser tab can open another tab; the shell's window is the whole application,
// so opening a link *inside* it would navigate away from the cockpit with no way
// back. The shell therefore hands the page a global that reaches the operator's
// real browser, and this module is the one place that decides between the two.
//
// The contract is the `__chartrTitleBar` one (see titlebar.ts): the shell injects
// a global before the page loads, and its mere presence is the capability. A
// plain browser tab never sees it and takes the fallback, so the fallback — not
// the hook — is the behaviour everywhere the shell is not, including before the
// macOS shell addition ships.
declare global {
  interface Window {
    // Bound by the native shell. Takes the URL and hands it to the OS; the
    // shell's binding returns a promise, which we neither await nor need.
    __chartrOpenExternal?: (url: string) => unknown
  }
}

// Where a link went, returned so the decision is observable from a test without
// a real window or a real shell.
//
//   - `shell`   — handed to the native shell's hook, which opens the system browser.
//   - `browser` — opened in a new browser tab, the no-shell fallback.
//   - `blocked` — not a scheme we are willing to hand to either.
export type OpenOutcome = 'shell' | 'browser' | 'blocked'

// Only ever hand out http(s). The web-links addon only matches those anyway, so
// this guard is not what makes links work — it is what keeps this function from
// becoming a way for terminal *output* to launch something on the operator's
// machine. `open`/`xdg-open` on the shell side will happily act on a `file:` or
// an app's custom scheme, and terminal output is untrusted text from whatever
// the agent printed, so the narrow allowlist lives on both sides of the seam.
function isOpenable(url: string): boolean {
  try {
    const scheme = new URL(url).protocol
    return scheme === 'http:' || scheme === 'https:'
  } catch {
    return false
  }
}

/**
 * Open a URL outside the cockpit, preferring the native shell's hook and falling
 * back to a new browser tab. Returns which path it took.
 *
 * `win` is injectable so the decision is a pure unit: pass a stub carrying (or
 * omitting) `__chartrOpenExternal` and assert the outcome.
 */
export function openExternal(url: string, win: Window = window): OpenOutcome {
  if (!isOpenable(url)) return 'blocked'

  const hook = win.__chartrOpenExternal
  if (typeof hook === 'function') {
    // The shell's binding answers with a promise that rejects when the OS opener
    // refused. Nothing here can retry and the operator has no action to take, so
    // it is swallowed rather than left as an unhandled rejection in the console.
    void Promise.resolve(hook(url)).catch(() => {})
    return 'shell'
  }

  // `noopener` is the standard hygiene for a link out of the app: the new tab
  // gets no `window.opener` handle back into the cockpit.
  win.open(url, '_blank', 'noopener,noreferrer')
  return 'browser'
}
