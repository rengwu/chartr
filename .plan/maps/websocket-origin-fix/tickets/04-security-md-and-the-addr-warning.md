---
type: task
claimed_by: s0f28b121a405
claimed_at: 2026-08-03T07:54:44Z
---

# There is no way to report a vulnerability, and `-addr :9000` is documented without a warning

## Question

Two gaps, both documentation-shaped, both with teeth.

**No `SECURITY.md`.** The reporter who found the websocket hole said plainly that
they emailed because the repo offers no channel — they guessed, and guessed right.
The next person may open a public issue instead, which turns a private report into a
zero-day on every running instance. Add `SECURITY.md` at the repository root (GitHub
surfaces it in the Security tab and in the new-issue flow): where to send a report,
what to expect back and roughly how fast, and that a reporter acting in good faith
will be credited unless they ask otherwise. Keep it short. Do not promise a bounty
or a fixed SLA that will not be honoured.

**`-addr :9000` is documented as an ordinary option.** `README.md:53-54` reads:

```
chartr                 # http://127.0.0.1:8787
chartr -addr :9000
```

with nothing said about what changes. `cmd/chartr/main.go:34` binds whatever it is
given, and there is no authentication anywhere in the server. A wildcard bind
therefore exposes an unauthenticated API that opens shells and spawns agents to the
whole network segment — no browser and no cross-origin trick required, so neither
ticket 01 nor ticket 02 touches it. Of everything in this map, this is the finding
most likely to bite someone who did exactly what the README showed them.

**What to do about it here.** Two parts, and keep them separate:

1. **Documentation, now.** Say in the README, at the flag, that chartr has no
   authentication and that binding to anything other than loopback exposes shell
   and agent-spawn access to everyone who can reach the port. Plain sentence, no
   hedging.
2. **A runtime warning, now.** At startup, if the resolved bind address is not
   loopback, log a clear warning naming what is exposed. A warning is the right
   weight for this ticket because it changes no behaviour and cannot break an
   operator who is deliberately binding wide.

**What not to do here.** Refusing a non-loopback bind, or gating it behind a token
or an explicit `--i-know` flag, is a behaviour change that would break existing
users, and it is genuinely arguable — that decision belongs to the trust-boundary
map's ticket 04, not to this one. Write the warning so that a later refusal can
replace it cleanly, and do not pre-empt the decision.

Tests lead for the warning: starting the server on a non-loopback address emits it;
starting on `127.0.0.1` and on the desktop shell's ephemeral `127.0.0.1:0` does
**not** — a warning every desktop user sees on every launch is a warning nobody
reads.

Done when: `SECURITY.md` exists at the repository root with a working contact; the
README documents what a non-loopback `-addr` exposes; a non-loopback bind warns at
startup and a loopback bind does not; the test above exists; and `go vet ./...` /
`go test ./...` pass.

## Answer

Both halves shipped, in three commits kept separate the way the ticket asks:
`SECURITY.md` (359959a), the documentation at the flag (5485b3b), and the startup
warning with its regressions (ea61a94).

**`SECURITY.md` at the repository root.** Two channels: GitHub's private
vulnerability reporting
(`https://github.com/rengwu/chartr/security/advisories/new`) and
`johngohrw@gmail.com` — the operator chose both when asked. It promises conduct,
not a service level: an acknowledgement "normally within a few days", an honest
answer including "not soon" when that is the truth, fixes on `main` and in the next
release with no back-porting, and credit unless the reporter declines. No bounty,
no SLA. A closing section names the two things that are by design — no
authentication, and that chartr runs the operator's agents as the operator — so
they are not re-reported as findings, while an *escape* from either still is, and
says the browser is the boundary chartr does defend.

**Documentation.** `README.md` and `docs/getting-started.md` both documented
`-addr :9000` as an ordinary option; both now say at the flag that chartr has no
authentication, that reaching the port is the whole access check, and that a
non-loopback bind hands shell and agent-spawn access to everyone who can reach it.
The getting-started edit is beyond the Done-when's letter and deliberate: it is the
same flag documented the same way two files apart, and fixing one of them leaves
the trap in the guide a new operator is actually reading.

**The warning.** `internal/server/exposure.go` is a predicate (`exposedBind`) and a
message (`exposureWarning`), acted on at one call site in `Serve` immediately after
the origin patterns are set. `Serve` rather than `cmd/chartr/main.go` for two
reasons: it is the last place before a handler can run that knows the address the
listener *actually* bound — `:9000` and `0.0.0.0:9000` are one wildcard bind
spelled two ways, and only the listener resolves it — and it is the seam both
binaries share, so the desktop shell is covered by the same line. The split into
predicate + message is the "replace cleanly" clause: a later refusal (trust-boundary
ticket 04) becomes a `return` at that one call site, and nothing here pre-empts the
decision. Non-loopback is warned, loopback and `localhost` are silent, and a
listener that is not `host:port` at all (a unix socket) is silent because nothing on
a network can reach it. A host that does not parse as an IP warns: `net.Listen`
resolves names so chartr never produces one, and the warning is the safe way to be
wrong.

**Done-when, clause by clause.** `SECURITY.md` exists with a contact the operator
picked — with one caveat below. The README documents the exposure. A non-loopback
bind warns and a loopback bind does not, pinned by
`internal/server/exposure_test.go`: `TestANonLoopbackBindWarnsAboutWhatItExposes`
starts a real wildcard-bound chartr and asserts the line names what is exposed (not
merely that something is), and `TestALoopbackBindDoesNotWarn` covers both quiet
spellings the product produces — the desktop shell's ephemeral `127.0.0.1:0` and
`-addr localhost:0`. Each asserts on the log only after a `/api/health` round-trip,
which is what proves `Serve` reached the point that logs, so the negative case is
deterministic rather than a race the rig usually wins. `go vet ./...` and
`go test ./...` pass (server package 56s); the new tests also pass under `-race` and
with `-tags chartrdev`.

**Rig.** `chartrtest` grew two things rather than a new seam: `WithBindAddress`, and
`Logged()` over the process's `log` output. The capture is an `io.MultiWriter`, so a
failing test still shows what the server logged; `BaseURL` now dials `127.0.0.1` when
the bind was the wildcard, since `0.0.0.0:PORT` is an address to listen on, not one
to connect to. `Option` changed from `func(*server.Options)` to a rig-level struct so
the bind address can travel with the server options; all existing options and callers
are unchanged in behaviour.

**Deliberately not done.** No refusal, no token, no `--i-know` flag — that is the
trust-boundary map's ticket 04, and this warning is written to be replaced by it. No
change to `cmd/chartr/main.go`: it binds what it is given and the check belongs where
the resolved address is known. No live wildcard bind was run by hand on the
operator's machine — opening a real LAN port to confirm what the test already pins at
the same seam is not worth the exposure.

**One thing a human must do.** GitHub's private vulnerability reporting has to be
enabled in the repository settings (Settings → Code security → Private vulnerability
reporting) or the advisory link in `SECURITY.md` 404s for whoever follows it. The
email works either way, so the file is not broken without it, but half the contact is
inert until that switch is flipped. Flagged rather than assumed.

**Unchanged and still true:** the map's disclosure note. Nothing here was pushed.
