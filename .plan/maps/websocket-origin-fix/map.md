# Websocket origin fix

## Destination

A web page the operator has open in another tab cannot reach the cockpit. Both
websocket handlers verify origin, every route rejects a Host that is not the
address chartr is listening on, and the two remaining low-severity findings from
the report are closed. Done when a cross-origin page can neither drive a terminal
nor read the model snapshot, a rebinding DNS name gets a 4xx on every route, and
`go vet ./...` / `go test ./...` pass with regressions pinning each.

## Notes

**Where this came from.** An external reviewer evaluating chartr for internal use
read the source and reported privately on 2026-08-02, choosing email over a public
issue because the repo has no `SECURITY.md`. They have not published and offered to
hold indefinitely. Every claim in the report was verified against the source before
this map was written; the findings are restated on the tickets with the line numbers
that carry them.

**This map does not wait on the trust-boundary map.** The companion planning map
[`localhost-trust-boundary`](../localhost-trust-boundary/map.md) grills the
assumption that produced these bugs. It is deliberately *parallel*, not a blocker.
The fix here has a known shape — the reviewer did the discovery, and the remedy is
the library's own default — so charting it would buy nothing and spend disclosure
days that were given as a courtesy. Ship 01 and 02; grill afterward. If the
planning map later settles on something stronger (a token, a refusal to bind
non-loopback), it lands on top of this, not instead of it.

**The hole, in one paragraph.** Both `websocket.Accept` calls set
`InsecureSkipVerify: true` (`internal/server/control.go:26`,
`internal/server/terminals.go:242`), which turns off coder/websocket's default
cross-origin rejection. Websockets are not subject to CORS, so binding to loopback
is not a boundary: any page on any origin can open both sockets. `/ws/control`
pushes the whole model snapshot on connect — space IDs, absolute paths, branch
names, terminal IDs, the agent library — and `/ws/terminal/{id}` writes inbound
binary frames straight to the PTY (`terminals.go:272-273`) and replays scrollback
outbound on attach (`terminals.go:257-261`). That is read *and* write access to the
operator's live shells and agent sessions, as them.

**The comment at both call sites is the actual defect.** Both say the same thing —
*"single-operator localhost tool reached through the Vite dev proxy; the
cross-origin Host check would only get in the way."* That is a development-time
convenience that shipped. Ticket 01 must keep the dev proxy working *without*
reopening the hole; the two are not in tension once the dev origin is named
explicitly rather than the check being disabled wholesale.

**Severity is not uniform and the tickets are not equal.** 01 and 02 are the
hotfix. 03 is defence in depth that means little once 01 and 02 land. 04 is policy
and documentation. 05 and 06 are genuine but minor, and neither is reachable by a
remote attacker. Work them in that order; do not let 05 and 06 delay 01.

**Disclosure timing is a human decision, not a ticket.** `.plan/` is committed and
this repo is public (`git@github.com:rengwu/chartr.git`). Pushing this map before
the fix ships publishes a working exploit path against every running instance. The
tickets are written to be worked, not to be safe to publish early — either hold the
commit until 01 and 02 are released, or land the fix first and the map after. This
note exists so the next session does not treat pushing as routine.

**Testing.** Process-boundary as everywhere else (spec, Testing Decisions). Each of
01, 02 and 03 gets a regression that fails against today's code — a test that only
passes after the fix is the only proof the fix is real. `go vet ./...` and
`go test ./...` before anything is called done.

## Decisions so far

<!-- one line per resolved ticket: gist + link. Empty until the first ticket ships. -->

## Not yet specified

- **Whether the API needs authentication at all.** Origin and Host checks stop a
  browser; they stop nothing else on the machine. That question is the
  trust-boundary map's, not this one's — see its ticket 03.
- **What a released fix owes the reporter.** They asked for nothing and offered to
  re-read a patch. Credit, an advisory, and whether to cut a point release rather
  than fold it into the next one are open.

## Out of scope

- **Authentication, tokens, or any credential.** Named here so it is not
  half-introduced under cover of a security fix. It is a design decision with a map
  of its own.
- **Rate limiting, TLS, or anything about a hostile *local* user.** The threat this
  map closes is a remote web page, not a process already running as the operator.
