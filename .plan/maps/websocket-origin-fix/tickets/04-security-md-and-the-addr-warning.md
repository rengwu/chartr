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
