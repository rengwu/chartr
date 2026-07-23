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

## Answer

Shipped. The open-target decision is one small module beside `titlebar.ts` —
`web/src/lib/external.ts`, `openExternal(url, win = window)` — because this is the
native-shell seam, not the prefs resolve: it mirrors the `__chartrTitleBar`
contract exactly (the shell injects a global; its presence *is* the capability),
so a plain browser tab and a pre-hook shell both take the `window.open(url,
'_blank', 'noopener,noreferrer')` fallback with no branching anywhere else. The
island lazily imports the bundled `@xterm/addon-web-links` (own chunk,
`addon-web-links-*.js`) with that function as its click handler — where a click
*goes* is never decided inside the renderer (ADR 0010). No pref gates it; every
terminal gets links.

The shell side is `cmd/webview/external.go`, bound in `main_webview.go` as
`w.Bind("__chartrOpenExternal", openExternalURL)` before `Navigate`, handing the
URL to the platform opener (`open` / `xdg-open` / `explorer` — the same ladder
`internal/server/configsurface.go` uses) and reaping the child in the background.
The file is deliberately **untagged**, so the guard compiles and is tested by the
default cgo-free build exactly as the lock is (ADR 0013).

One thing the ticket did not name and the code now holds: a URL reaching this
hook traces back to text an *agent* printed into a terminal, and `open` will
happily launch an application for a `file:` path or a registered custom scheme.
Both sides therefore hold the same http(s)-only allowlist — `checkExternalURL` in
Go (table-tested over `file:`, `javascript:`, `vscode://`, `mailto:`, schemeless,
hostless) and `isOpenable` in TS — since either alone would be the only thing
between terminal output and a launched app. The hook's returned promise is
swallowed client-side so a refusal is not an unhandled rejection.

Tests lead as asked: `external.test.ts` is the pure decision unit — hook present →
hook called, hook absent → `window.open`, a non-callable global still falls back,
non-http blocked from both paths.

**Not driven live.** The Chrome extension was not connected this session, so the
click was never exercised in a real page, and a click inside the native webview is
not scriptable either. What was verified: `check`/`build`/`vitest` (119), `go vet`
+ `go test ./...`, the addon lands as its own bundled chunk, and the tagged
`CGO_ENABLED=1 -tags webview` shell builds and starts with the binding installed.

Gist: a bundled web-links addon whose clicks route through one shell-hook-or-tab
decision, plus the `__chartrOpenExternal` webview binding behind an http(s)-only
allowlist on both sides.
