---
type: task
---

# Both websockets accept any origin, so any page can drive a terminal

## Question

`internal/server/control.go:26` and `internal/server/terminals.go:242` each pass
the same option to coder/websocket:

```go
c, err := websocket.Accept(w, r, &websocket.AcceptOptions{
    // Single-operator localhost tool reached through the Vite dev proxy; the
    // cross-origin Host check would only get in the way (as on the control
    // socket).
    InsecureSkipVerify: true,
})
```

`Accept` rejects cross-origin handshakes by default. This turns that off. Because
websockets are not subject to CORS, no preflight stands between an arbitrary page
and these sockets, and **binding to loopback is not a boundary** — `127.0.0.1` is
reachable from any origin the operator happens to have open.

What that buys an attacker, concretely:

- **`/ws/terminal/{id}` is arbitrary keystroke injection.** The read loop writes
  inbound binary frames straight to the PTY (`terminals.go:272-273`). A page sends
  one binary frame and it is typed into the operator's live shell — or into a
  running agent session — as them. No response reading is required, so it works
  blind.
- **`/ws/control` is the reconnaissance that makes the above trivial.** It pushes
  the whole model snapshot on connect: space IDs, absolute repository paths, branch
  and dirty state, the agent library, config layer paths, and **the live terminal
  IDs**. The reporter noted that IDs are sequential (`internal/terminal/manager.go:171-172`,
  `fmt.Sprintf("t%d", m.seq)`) and therefore guessable; they do not even need
  guessing, because this socket hands them over.
- **Both directions leak.** Websockets are not constrained by the same-origin
  policy for *reading* once the handshake completes, so scrollback replay on attach
  (`terminals.go:257-261`) plus the live PTY stream is straightforward exfiltration
  of whatever is on the operator's terminals.

The desktop shell binds an ephemeral port (`cmd/webview/main_webview.go:85`,
`127.0.0.1:0`), which narrows the target but does not remove it — loopback port
scanning over websockets is fast enough to treat as friction rather than
mitigation. The CLI's default `127.0.0.1:8787` needs no scan at all.

**The fix.** Replace `InsecureSkipVerify: true` with `OriginPatterns` scoped to the
address chartr is actually listening on, at both call sites. The bound address is
known — `run` has it (`cmd/chartr/main.go:49-61`) and the webview resolves its
ephemeral port after `net.Listen` — so plumb it to the server rather than hardcoding
a pattern. Same-origin requests carry no `Origin` header restriction problem; the
patterns exist for the one case below.

**Keep the dev proxy working without reopening the hole.** The comment is not wrong
that Vite's proxy origin would otherwise be refused — it is wrong that the remedy is
disabling the check. Name the dev origin explicitly instead: an extra pattern for
the Vite host, admitted only in a development build (a build tag, or a flag that is
off by default and documented as development-only). What must not survive is a
shipped binary that accepts every origin. Decide which mechanism and say why in the
answer; either is acceptable, an unconditional extra pattern is not.

Tests lead, and must fail against today's code: a handshake carrying a foreign
`Origin` header is rejected on **both** `/ws/control` and `/ws/terminal/{id}`; a
handshake carrying the bound address as its origin is accepted and still streams;
and — the regression that matters — a rejected terminal handshake results in
**nothing** written to the PTY. Assert the last one against the terminal, not just
the HTTP status, so a future refactor that accepts-then-closes cannot pass.

Done when: neither `Accept` call sets `InsecureSkipVerify`; both scope origins to
the bound address; the dev proxy path still works and is gated so it cannot ship
enabled; the tests above exist and fail without the fix; and `go vet ./...` /
`go test ./...` pass.
