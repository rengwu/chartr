---
type: task
blocked_by: [02]
claimed_by: s4a8d13e7f713
claimed_at: 2026-07-31T02:47:27Z
---

# The OS notification

## Question

Make the clock's event reach the operator when nothing is open. The chartr server
fires an operating system notification itself, so it arrives with the cockpit's
browser tab closed, with no permission grant and with no webview — the case that
motivated the whole effort.

**Three platform paths, all `exec`.** macOS shells out to `osascript`, Linux to
`notify-send`, Windows to a PowerShell toast. None of them links a library, so the
cgo-free single supported artifact (ADR 0011) is unaffected and nothing changes
about how chartr builds or cross-compiles.

**Best-effort is the contract, and it is load-bearing.** A missing binary, a
non-zero exit, a machine with no notification daemon, a headless box: each logs
once and is otherwise ignored. The feature degrades; the cockpit does not. Log
once per process rather than once per notification — an operator on a machine that
can never notify should not have their log filled by a working feature.

**One interface, platform chosen once at construction.** The notifier is a small
interface with the platform selection made where the server is built, so tests
substitute a stub and never shell out. This is what keeps the suite honest on
every OS, including the ones CI does not drive daily.

**Content.** The title names the space. The body names the ticket where the tab is
a session, the reason in the operator's words — "finished", "needs you",
"crashed", "exited" — and how long it ran. A tab that is not a session says what
it can without inventing a ticket. Clicking the notification does nothing:
routing a click back into a specific cockpit view needs a deep-link scheme that
does not exist, and it is out of scope.

**Arguments are passed, never interpolated into a shell string.** A space path or
a ticket title can contain quotes, backticks or newlines, and every one of these
platform paths is a command line. Build argument vectors and let the OS do the
quoting.

Tests lead, at the process boundary in `internal/server` with a stub notifier
substituted: a tab running a stub agent that works past a short configured *n*
produces exactly one notification carrying the right space, ticket and reason; a
tab that works briefly produces none; `enabled = false` produces none; a notifier
that returns an error leaves the server healthy and the model snapshot unchanged.
Add a unit test that a space path containing quotes and a newline produces a
correct argument vector on each platform's builder, since that is the one part of
this that a stub cannot cover.

Done when: a long stub run fires exactly one OS notification naming its space,
ticket, reason and duration; a short one fires none; a failing or absent platform
notifier degrades silently and logs once; no test shells out to a real notifier on
any platform; `go vet ./...` and `go test ./...` pass.
