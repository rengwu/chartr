---
type: task
blocked_by: [01]
---

# Clickable links + shell hook

## Question

An operator clicks a URL in terminal output and it opens — in the system browser
when running the macOS shell, in a new browser tab otherwise.

- Load the web-links addon (lazily, bundled) so URLs in output are clickable.
- On click, open via a shell-provided external-open hook when present — mirroring
  the existing `__chartrTitleBar` native-shell global pattern — and fall back to
  opening a new browser tab (`window.open(url, '_blank')`) when the hook is
  absent.
- Add the small macOS webview-shell addition that exposes that hook
  (`__chartrOpenExternal`) so a clicked link reaches the system browser (ADR
  0013). Until/without the shell hook, the browser-tab fallback is the behaviour.

Tests lead: a pure unit over the open-target decision — hook present → hook called;
hook absent → `window.open` fallback. The addon wiring and the native side are
verified at runtime.

Done when: a URL in output is clickable and opens in the system browser under the
macOS shell (via the new hook) and in a new tab elsewhere; the decision unit is
green; frontend + Go checks pass.
