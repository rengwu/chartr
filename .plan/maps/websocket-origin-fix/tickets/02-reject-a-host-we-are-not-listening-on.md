---
type: task
claimed_by: s8a33b2220cbe
claimed_at: 2026-08-03T04:35:06Z
---

# No route checks Host, so DNS rebinding grants the whole API

## Question

Nothing in the server validates the `Host` header. `s.mux` is handed to
`http.Server` raw (`internal/server/server.go:267`) with no wrapper, and no handler
inspects Host itself. Every route — the API, both websockets, and the SPA
(`server.go:133-231`) — answers to whatever name resolved to the listening socket.

That is the DNS rebinding path. An attacker's page on `evil.example` is served from
a name whose DNS record they control with a short TTL; they re-point it at
`127.0.0.1` and the browser now treats requests to `http://evil.example:8787/` as
**same-origin**. Every same-origin protection disappears at once: reads are allowed,
so responses come back, and the entire HTTP API is available for read and write —
including the routes ticket 03 discusses and the ones a preflight would otherwise
have protected.

This is **independent of ticket 01 and is not fixed by it**. Origin patterns scoped
to the bound address do not help when the browser believes the attacker's origin
*is* the bound address. Both tickets are needed; either alone leaves a full path in.

**The fix.** Wrap the mux in a handler that rejects any request whose `Host` is not
the address chartr is listening on, before routing. The comparison has to be made
carefully rather than by string equality:

- Compare host and port separately (`net.SplitHostPort`), tolerating a missing port
  for the default.
- A loopback bind must accept `127.0.0.1`, `[::1]` and `localhost`, and nothing
  else. These are the names the operator's own browser will actually send, and the
  set is small and closed — an allowlist, not a pattern match.
- A wildcard bind (`:9000`, `0.0.0.0:9000`) has no single correct name, which is
  exactly the configuration ticket 04 warns about. Decide and state the rule: the
  defensible one is to accept the machine's own addresses and refuse names that do
  not resolve to them, but if that proves unworkable, say so in the answer and
  narrow it rather than silently accepting everything.
- Reject with a plain 403 and no body worth reading. Do not redirect.

The wrapper is the natural home for anything else that must hold for *every* route,
which the trust-boundary map's ticket 06 will have opinions about. Write it so a
second check can be added beside the first without restructuring, but do not add
speculative hooks now.

Tests lead, and must fail against today's code: a request carrying a foreign Host
is refused on an API route, on the SPA route, and on **both** websocket routes —
the last matters because a rebinding attacker reaching `/ws/terminal/{id}` gets
everything ticket 01 describes regardless of origin patterns. A companion asserts
that `127.0.0.1`, `localhost` and `[::1]` all still work against a loopback bind,
so the fix does not break the operator's own browser or the desktop shell's
ephemeral-port URL.

Done when: every route rejects a Host that is not the bound address; the three
loopback spellings still work; the wildcard-bind rule is decided and written down
in the answer; the tests above exist and fail without the fix; and `go vet ./...` /
`go test ./...` pass.

## Answer

**Every route goes through a host gate before it is routed.** Commit `7eb895f`.

`Serve` now hands `http.Server` `hostGate(ln.Addr(), s.mux)` rather than the mux
raw (`internal/server/server.go:285`). The gate is one wrapper
(`internal/server/hostgate.go`): a `hostRule` derived once from the bound
address, and a per-request `allows(r.Host)` that answers 403 with `http.Error`
and nothing worth reading, never a redirect. Because it wraps the mux, it covers
the API, the SPA, and both websockets by construction — there is no route it can
be forgotten on, which is the property the ticket was after.

**The comparison.** `net.SplitHostPort` splits the header; a Host with no port
is read as 80, the only scheme chartr serves. The port must equal the bound port.
The host is normalized once — lowercased, trailing root dot stripped, IPv6
brackets removed — so `LOCALHOST.` and `[::1]` are not third and fourth answers,
and normalization widens nothing (`evil.example.` normalizes to `evil.example`,
refused either way). Names are matched verbatim against a closed list;
addresses are compared with `net.IP.Equal`, so `::ffff:127.0.0.1` is not a way
around `127.0.0.1`.

**The rule, by bind:**

- **Loopback** (`127.0.0.1:PORT`, the CLI default and the shell's ephemeral
  port) — exactly `127.0.0.1`, `[::1]` and `localhost` on that port, and nothing
  else. A closed allowlist of what the operator's own browser actually sends.
- **One specific non-loopback address** — that address alone.
- **Wildcard** (`:9000`, `0.0.0.0:9000`) — the three loopback spellings, plus
  any address the machine currently holds, as a literal. No other name, ever.

**The wildcard rule, and the half of the ticket's suggestion I did not take.**
The ticket proposed accepting the machine's own addresses *and refusing names
that do not resolve to them*. The first half is what I built. The second half is
not merely unworkable, it is self-defeating: resolving the Host header is exactly
what rebinding subverts. `evil.example` resolves to `127.0.0.1` **because the
attacker pointed it there**, so a lookup would confirm it as one of our own
addresses and admit the very request the check exists to stop. So the gate never
resolves a name — it refuses one it does not already know. That is the narrowing
the ticket asked me to state rather than a silent acceptance, and it has a real
cost worth naming: **a machine bound to `0.0.0.0` and reached at its own hostname
(`http://mybox.local:9000/`) is now refused, and the operator must use the IP.**
That is a behaviour change for a configuration ticket 04 already warns about; I
think refusing is right there, but it is the kind of call a human may want to
revisit when 04 writes the warning, so it is flagged rather than buried.

Own addresses are looked up at request time (`net.InterfaceAddrs`) rather than
cached at startup, so a laptop that joins a network or drops a VPN does not
answer to a stale set — and the lookup is only ever reached by a request the
static set already turned down, so it is off the ordinary path.

**A listener with no host:port** (a unix socket) leaves the check off, and that
is correct rather than a compromise: rebinding is an attack on a *name* resolving
to a TCP port, and a socket no browser can dial has neither. Nothing in chartr
builds one today; `originPatterns` has the same branch for the same reason.

**Against each Done-when clause.**

- *Every route rejects a Host that is not the bound address* — the wrapper sits
  above the mux, and there are tests on an API route
  (`TestForeignHostRefusedOnAPIRoutes`), the SPA route
  (`TestForeignHostRefusedOnTheSPARoute`), and **both** websockets
  (`TestForeignHostRefusedOnBothWebsocketRoutes`).
- *The three loopback spellings still work* —
  `TestEveryLoopbackSpellingStillReachesEveryRoute` drives `127.0.0.1`,
  `localhost` and `[::1]` against `/api/health` and `/ws/control`, and
  `TestTheOperatorsBrowserAtLocalhostStillStreams` does the same end to end: a
  real dial to `ws://localhost:PORT` against a `127.0.0.1` bind, reading a
  snapshot. `TestTheRightNameOnTheWrongPortIsRefused` pins the other side, that
  the port is half the address.
- *The wildcard rule is decided and written down* — above, and in
  `hostRuleFor`'s doc comment where the next reader will meet it.
- *The tests exist and fail without the fix* — verified by unwiring the gate
  (`Handler: s.mux`) and re-running. Four fail; the two positive tests pass
  either way, which is what says they are not vacuous. The websocket failures
  read `= 101, want 403` — the socket genuinely opened under the attacker's
  name, which is the reported hole reproduced rather than a status code.
- *`go vet ./...` / `go test ./...` pass* — both, and also under
  `-tags chartrdev`, and `go test -race` over the origin and host tests.

**The rebinding pairing is in the test, not just the name.** `HandshakeWithHost`
(`internal/chartrtest/rig.go`) presents `Host: attacker.example:PORT` *and*
`Origin: http://attacker.example:PORT` — what a rebound browser really sends,
since it believes that name is this address. coder/websocket authorizes any
handshake whose Origin equals its Host regardless of `OriginPatterns`, so ticket
01 is provably not what turns these away; only the Host check is. It is a raw
TCP handshake because net/http drops a Host set through a header map, and
`DialOptions` has no way to reach `Request.Host`. Ticket 01's
`AttemptCrossOriginTerminalWrite` now shares that one handshake writer — a pure
refactor, its behaviour and its test unchanged.

Beyond the suite, the real binary was checked by hand: bound to `127.0.0.1:8912`,
the three loopback spellings and `LOCALHOST.:8912` answer 200, while
`evil.example:8912` and `127.0.0.1:9999` answer 403.

**Deliberately not done.**

- **No test for the wildcard bind.** Binding `0.0.0.0` in a test raises the macOS
  firewall dialog on an unsigned test binary, which is not a thing to add to
  `go test ./...`. The wildcard path is therefore reasoned and documented, not
  pinned — the one gap in this ticket's coverage, and stated here rather than
  left to be discovered. If it is worth pinning, it wants a rig option to bind a
  given address and a decision about CI, which is more than this ticket.
- **Nothing about the dev proxy.** Vite forwards with `changeOrigin: true`, which
  rewrites Host to the *target* — the bound address — so the proxy passes the
  gate with no dev-only widening. The `chartrdev` tag stays what ticket 01 made
  it: an origin gate, not a host one.
- **No second every-route check.** The wrapper is written so one can be added
  beside the first, per the ticket, but no speculative hook was added for the
  trust-boundary map's ticket 06.
- **No documentation change.** The `-addr` warning is ticket 04's, and the
  wildcard rule above is what it will need.

**One flag for a human**, unchanged from ticket 01: the map's disclosure note
applies to this commit too. It sits on `main` unpushed, and pushing `.plan/`
before a release publishes the exploit path. Nothing here pushes.
