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

## Answer

The clock's event now reaches the operator with nothing open. `internal/notify` is
the whole of the notifier: a one-method `Notifier`, a pure `Compose` that writes
what the operator reads, and three `exec` platform paths chosen once by
`notify.Platform()` where the server is built.

**What shipped.**

- `notify.Compose` is the content, pure and table-tested away from any platform,
  server or PTY. The title is the space's name — the sidebar's name, resolved from
  the registry in `server.announceRun`, since the event carries only a space id. The
  body is `<map> #NN <phrase> · ran <duration>`, with the four reasons in the
  operator's words (`finished`, `needs you`, `crashed`, `exited`) and a duration
  written the way someone glancing at a notification reads it (`45s`, `4m 12s`,
  `1h 3m`). A tab that is not a session says `A terminal finished · ran 2m` rather
  than inventing a ticket, and a space chartr can no longer name still reports,
  titled `chartr`.
- **Every path builds an argument vector.** macOS passes the title and body as
  trailing argv into an AppleScript `on run argv` handler, so the script osascript
  parses is a constant; Linux passes them positionally to `notify-send` after `--`.
  Windows is the one platform where there is no argument vector to hide them in —
  `powershell -Command` takes a script and concatenates anything after it — so the
  two strings ride as **base64** blobs the script decodes, an alphabet that cannot
  contain a quote, a backtick, a newline or a `$`. That is the honest reading of
  "never interpolated": operator text never reaches a parser as syntax anywhere.
- **Best-effort, and it is the server's to swallow.** `onRunFinished` marks the dot,
  pushes the snapshot, and only then announces — off the sampler goroutine, because
  the notifier execs and neither the cockpit's push nor every tab's status sampling
  may wait on a notification daemon. A missing binary, a non-zero exit, an
  unsupported OS: each returns an error that changes nothing and is logged through a
  `sync.Once`, so a machine that can never notify says so once per process.
- `server.Options.Notifier` defaults to the platform notifier and is substituted by
  `chartrtest.WithNotifier`. Nothing downstream of `New` knows which OS it is on.

**Done-when, clause by clause.** *One notification naming space, ticket, reason and
duration* — `TestFinishedSessionFiresOneOSNotification` spawns a real session on the
stub agent against a real ticket, and asserts the title is the space name and the
body names `widget #01`, `finished` and a duration; it then holds still for several
more sampler ticks to prove *exactly* one, not one per sample. *A short one fires
none* — `TestRunUnderThresholdFiresNoNotification`. *`enabled = false` fires none* —
`TestNotificationsDisabledFireNone`. Both negatives wait for the tab to read idle
with no dot, which is the sample the clock decided on, so a zero there is a run that
never emitted rather than one still in flight. *A failing notifier degrades and logs
once* — `TestFailingNotifierLeavesTheCockpitWorking` (health still 200, the dot the
same event raised still in the snapshot) and `TestFailingNotifierLogsOncePerProcess`,
which fails two consecutive runs and finds exactly one line. *No test shells out on
any platform* — the stub is substituted at construction, and the arg-vector unit test
calls the three builders as pure functions, so a Linux CI still proves the macOS and
Windows vectors. `go vet ./...` and `go test ./...` pass, `-race` included; no `web/`
file was touched, so the frontend bar is untouched.

**Ticket 01's flag, answered.** A drop does **not** flush the clock; a run still only
ends on a sample. Every drop is a tab the operator is standing at: a session that
died on its own *pins* rather than drops and reports as `dead` through the ordinary
path, while what drops is an ad-hoc shell they exited, a session they killed, or an
on-ramp tab whose agent quit — notifying them about the thing they just did is worse
than silence. The accepted cost, recorded at `manager.onExit`: an on-ramp tab whose
agent exits on its own after a long run reports nothing.

**Deliberately not done.** No click handling (out of scope — there is no deep-link
scheme). No ticket *title* in the body: `RunFinished` carries the slug and number,
which is the identity the tab already wears, and resolving a title would mean
rescanning `.plan/` on the sampler's path for a nicer noun. Nothing retries, batches
or queues a failed notification. One thing to know rather than a doubt: the macOS
path was fired by hand once against a hostile space name (quotes, `$HOME`, a
backtick) and delivered with no expansion — that is not in the suite and never will
be, by the map's rule.
