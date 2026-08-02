---
type: task
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
