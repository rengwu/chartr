---
type: task
blocked_by: [01]
---

# Make failure visible without a terminal

## Question

Give a bundled launch somewhere to put a fatal startup error. Today the shell
writes to standard error and exits, which a Finder launch discards — so every
failure mode becomes a silent bounce in the Dock. The message that must not be
swallowed is ADR 0013's deliberate one: *the native runtime is missing, use the
supported binary*. That error exists specifically so a missing dependency reads
as a missing dependency rather than a broken product, and under a bundle it
currently reaches nobody.

Two surfaces, both active **only when bundled** — a terminal launch keeps
standard error exactly as it is:

- **A native alert** naming what failed, so a failed launch is a message rather
  than nothing. Cocoa, sitting with the existing platform code behind the
  `webview` and `darwin` build tags.
- **A log file** under the operator's own logs directory, so a bug report can
  carry evidence instead of a description.

Use the bundle detection ticket 01 established; do not add a second way to ask
the same question.

**What is tested and what is not.** The alert is not unit-tested — ADR 0013
already accepts that for surfaces needing a real display, and this is one. What
*is* tested is the tag-free decision of **which sink applies**: bundled selects
the alert-and-log path, a terminal launch selects standard error. Keep that
decision on the tag-free side so the cgo-free suite reaches it.

## Done when

`go test ./...` passes at `CGO_ENABLED=0` with a test covering the sink choice in
both directions. On a real Mac, a shell binary forced into a fatal startup
failure while bundled raises an alert naming the failure, and the same text lands
in the log file; the same binary run from a terminal still prints to standard
error and exits non-zero, unchanged. The missing-native-runtime error in
particular is confirmed to reach the alert. `go vet ./...`, the frontend `check` /
`build` / `vitest`, and the no-amber check are green.
