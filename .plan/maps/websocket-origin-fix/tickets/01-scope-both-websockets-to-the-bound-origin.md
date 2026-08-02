---
type: task
claimed_by: sf2e933acc091
claimed_at: 2026-08-02T20:25:51Z
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

## Answer

**Both sockets are scoped to the bound address, and the dev proxy is a build
tag.** Commit `36e3b1e`.

`InsecureSkipVerify: true` is gone from both call sites; each now passes
`OriginPatterns: s.origins` (`internal/server/control.go:22`,
`internal/server/terminals.go:238`).

**Where the address comes from.** `Serve` derives the patterns from
`ln.Addr()` and stores them on the `Server` before the listener is served
(`internal/server/server.go`). `New` cannot have them: the shell binds
`127.0.0.1:0` and learns its port only after `net.Listen`, so the bound address
does not exist at construction. `Serve` is the last moment before a handler can
run and the first at which the address is known, and because every handler runs
on a goroutine started after that write, the ordering needs no lock (confirmed
under `go test -race`). Both entry points get this for free — `cmd/chartr` and
`cmd/webview` already hand their listener to `Serve`, so neither changed.

**What the patterns are** (`internal/server/origins.go`). A loopback or
wildcard bind yields the three loopback spellings of the port —
`http://127.0.0.1:PORT`, `http://[::1]:PORT`, `http://localhost:PORT`; a bind to
one specific non-loopback address names that address alone. Every pattern carries
scheme, host and port in full, so there is no wildcard anywhere and nothing
matches by prefix. Two things worth stating for the next reader:

- The aliases are not a widening. A browser sets `Origin` itself and a page
  cannot forge it, so an `Origin` of `localhost:PORT` means the page was served
  by whatever owns that port — which, on the same loopback stack, is chartr.
- coder/websocket authorizes any handshake whose `Origin` equals the request
  `Host` *regardless* of `OriginPatterns` (`accept.go`, `authenticateOrigin`).
  That covers every ordinary browse, so the patterns are doing work only where
  the two differ: the loopback alias above, and the dev proxy. It also means a
  wildcard bind is still reachable under names chartr cannot enumerate — closing
  that is ticket 02's `Host` check, not something `OriginPatterns` can do.

**The dev gate is a build tag, not a flag.** `devOriginPatterns` returns nothing
in `origins_shipped.go` (`//go:build !chartrdev`) and the Vite origins in
`origins_dev.go` (`//go:build chartrdev`). A flag defaulting to off is still
*present* in the released binary, and anything that can pass an argument — a
launcher, a desktop entry, a talked-through support step — can turn it back on;
a tag means the extra origin is not compiled in at all, so no argument and no
environment variable widens a shipped chartr. `make dev-backend` now runs
`go run -tags chartrdev ./cmd/chartr` and is the only thing that sets it.
`CHARTR_DEV_ORIGIN` overrides the default `localhost:5173` pair for a Vite that
took another port, and is readable only in that build.

The proxy genuinely does need naming: Vite forwards with `changeOrigin: true`
(`web/vite.config.ts`), so the request arrives carrying the backend's `Host` and
the browser's own `Origin` — they differ, and the Origin-equals-Host rule cannot
admit it. That one origin is the whole of what `InsecureSkipVerify` was buying.

**Against each Done-when clause.**

- *Neither `Accept` call sets `InsecureSkipVerify`* — grep finds it nowhere in
  `internal/` or `cmd/` but in one comment recording what was removed.
- *Both scope origins to the bound address* — same `s.origins`, one derivation.
- *The dev proxy still works and is gated so it cannot ship enabled* — proved
  from both sides: `TestViteDevOriginAdmittedUnderTheDevTag` (`chartrdev`,
  control socket *and* a terminal that streams) and
  `TestViteDevOriginRefusedWithoutTheDevTag` (every other build, including the
  one `make test` and CI run).
- *The tests exist and fail without the fix* — verified by re-adding
  `InsecureSkipVerify: true` and re-running. Four fail:
  `TestControlSocketRefusesForeignOrigin`,
  `TestTerminalSocketRefusesForeignOrigin`,
  `TestViteDevOriginRefusedWithoutTheDevTag`, and
  `TestRefusedTerminalHandshakeWritesNothingToThePTY` — the last reporting the
  injected marker actually present in the terminal's replayed scrollback, i.e.
  the reported exploit reproduced, not merely a status code.
- *`go vet ./...` / `go test ./...` pass* — both, and also under
  `-tags chartrdev`, and `go test -race ./internal/server/`. Frontend `check`,
  `build` and `vitest` (217 tests) are green too, since one comment in
  `vite.config.ts` changed; no amber in the built CSS.

**The PTY assertion, and why it is hand-rolled.**
`AttemptCrossOriginTerminalWrite` (`internal/chartrtest/rig.go`) writes the
handshake over a raw TCP connection and pipelines a masked binary frame
immediately after it, without waiting for the response. It is deliberately not a
`websocket.Dial`: a client library will not write a frame onto a handshake it
already knows was refused, so a test built on one could not distinguish an
outright rejection from a server that accepts and then closes — which is the
refactor the ticket asked to pin. The test then attaches legitimately, runs its
own command, and asserts the injected marker is absent from the replayed
scrollback. The ordering is exact rather than a race: the PTY consumes input in
arrival order, so once our later command's *output* has come back, anything the
probe managed to inject would already have echoed.

**Deliberately not done.**

- **No `Host` check.** A DNS-rebinding name still satisfies Origin-equals-Host
  and would be admitted here. That is ticket 02 and I did not reach into it.
- **No authentication of any kind**, per the map's Out of scope.
- **`docs/` untouched.** There is no development-workflow document to amend —
  the Makefile target's comment is where `dev-backend` is documented, and it
  now explains the tag; `web/vite.config.ts` carries a pointer for a developer
  who meets a 403 and looks there first.

**One flag for a human.** The map's disclosure note applies to this commit: it
sits on `main` unpushed, and pushing `.plan/` before a release publishes the
exploit path. Nothing here pushes.
