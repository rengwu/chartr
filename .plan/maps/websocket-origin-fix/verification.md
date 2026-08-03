# Verification — Chris Abernethy's report (Tradeverifyd)

Validated 2026-08-03 against `37f79bb` (working tree clean). Every finding from
the emailed report is checked below against the code, the test suite, and a live
binary — not against the tests alone.

**Result: all seven findings resolved.** Two follow-ups worth knowing about, both
recorded at the bottom; neither reopens the reported issue.

## How this was validated

- Read the implementation: `internal/server/{origins,gate,hostgate,exposure,filemodes}.go`,
  `server.go:279-330`, `Makefile:330-500`.
- `go vet ./...` — clean. `go test ./...` — all packages pass (`internal/server` 57s).
- Built `./cmd/chartr` and probed a live server on a free loopback port with curl:
  raw websocket handshakes, forged `Host` headers, CORS-simple POSTs.
- Confirmed the desktop shell shares the path: `cmd/webview/main_webview.go:79,126`
  goes through the same `server.New` / `srv.Serve`, so it inherits the same gate.

## Findings

### 1. `InsecureSkipVerify: true` on both websocket handlers — FIXED

Both `websocket.Accept` sites now pass `OriginPatterns: s.origins`
(`control.go:29`, `terminals.go:245`), derived from the address the listener
actually bound (`origins.go:33`, set in `Serve` before the listener is served).

The Vite dev origin that `InsecureSkipVerify` was really buying is now behind the
`chartrdev` build tag (`origins_dev.go` / `origins_shipped.go`), so it is *not
compiled into* a shipped binary — no flag or env var can widen a release.

Live probe, server bound `127.0.0.1:8834`:

| Origin sent | `/ws/control` | `/ws/terminal/t1` |
|---|---|---|
| `http://evil.example` | 403 | 403 |
| `http://127.0.0.1:9999` (wrong port) | 403 | — |
| `http://127.0.0.1.evil.com` (suffix trick) | 403 | — |
| `http://127.0.0.1:8834` | 101 | 101 |
| `http://localhost:8834` | 101 | — |

Refusal body: `request Origin "evil.example" is not authorized for Host "127.0.0.1:8834"`.

The terminal handler refuses **before** `Attach` (`terminals.go:238-250`), so a
turned-away handshake writes nothing to the PTY and replays no scrollback — the
exact exploit path in the report. Covered by
`TestRefusedTerminalHandshakeWritesNothingToThePTY`.

### 2. No Host-header validation → DNS rebinding — FIXED

`gate(ln.Addr(), s.mux)` wraps the **entire** mux (`server.go:306`), so the check
runs before routing and a route added later cannot miss it. `hostRule`
(`hostgate.go`) derives the admitted set from the bound address and deliberately
never resolves a name — resolving would defeat the check, since the rebinding
name *does* point at 127.0.0.1.

Live probe:

| Request | Result |
|---|---|
| `GET /api/health`, correct Host | 200 |
| `GET /api/health`, `Host: localhost:8834` | 200 |
| `GET /api/health`, `Host: evil.example:8834` | **403** |
| `POST /api/spaces`, `Host: evil.example:8834`, valid JSON | **403** |
| `GET /` (SPA route), `Host: evil.example:8834` | **403** |
| `GET /api/health`, `Host: localhost:9999` (right name, wrong port) | **403** |
| `/ws/terminal/t1`, forged Host | **403** at the gate |

Refusals are plain `403`, never a redirect — a redirect to the right name would
hand the attacker's page a working URL.

### 3. Handlers decode JSON regardless of Content-Type — FIXED

`mustDeclareJSON` / `declaresJSON` (`gate.go:64-84`), in the same gate. Every
POST must declare `application/json`, body or not, because a bodyless POST is
still a side effect worth causing blind; anything carrying a body must declare it
whatever the method. Bodyless `DELETE`/`PUT` are deliberately untouched so the
method split that already keeps `PUT /api/config/agents/{name}` out of reach
keeps working.

Live probe — both routes named in the report:

| Request | Result |
|---|---|
| `POST /api/spaces`, `Content-Type: text/plain` | **415** |
| `POST /api/spaces/{id}/terminals`, no Content-Type | **415** |
| `POST /api/spaces`, `application/json; charset=utf-8` | 400 (reaches the handler) |

### 4. No SECURITY.md — FIXED

`SECURITY.md` added: private reporting via GitHub advisories or email, an honest
one-maintainer conduct promise, credit by default, and a "what is already known"
section stating that no-auth and running-agents-as-you are by design — while
naming the browser boundary as the thing that *is* defended.

### 5. `-addr` exposes an unauthenticated API without warning — FIXED

`exposedBind` / `exposureWarning` (`exposure.go`), asked of the *resolved*
listener address so `:9000` and `0.0.0.0:9000` are treated alike and the desktop
shell's `127.0.0.1:0` stays quiet. Nothing is refused — an operator who meant it
keeps working.

Live check, `-addr 0.0.0.0:8836`:

```
chartr: WARNING: listening on [::]:8836, which is not loopback. chartr has no
authentication: anyone who can reach this port can open shells, run commands and
spawn agents on this machine as you, and read any file you can. Bind 127.0.0.1
(the default) unless you meant to expose it.
```

A loopback bind printed nothing. README documents the exposure at the flag.

### 6. Session payloads 0644 under 0755 — FIXED for new writes

`filemodes.go` adds `writeOwnerFile` (0600, with an explicit `chmod` because
`os.WriteFile` only applies its mode on create) and `ownerDirMode` (0700).
Applied to session payloads (`spawn.go:387-412`), the registry, the agent
library and the config root; the registry additionally repairs an older
world-readable file on its next save.

Deliberately **not** applied to what chartr writes into the operator's own
repository — the `.gitignore` run marker, claimed tickets under `.plan/`, the
installed adapter — which stay ordinary 0644/0755 repo files.

See the follow-up below: this is forward-only.

### 7. `make appimage` fetches tools with no checksum — FIXED

Four SHA-256s pinned in the Makefile (linuxdeploy and appimagetool ×
x86_64/aarch64), with `verify_tool` run **after download and before `chmod +x`**,
on the cached path as well as the freshly fetched one. A mismatch deletes the
file and fails the target — never a warning. The bump procedure is documented
inline, including cross-checking against GitHub's server-side digest.

## Follow-ups

Neither reopens a reported finding. Both are worth mentioning if you reply to him.

### A. Payloads already on disk are still world-readable

The permission fix is forward-only, scoped out on purpose in ticket 05 ("Not
migrated: payloads already on disk" — walking the data root is a migration, not
that ticket). In practice on this machine:

- 83 of 83 `~/.config/chartr/sessions/*/payload.md` are `0644`
- all 84 session directories are `0755`, as is `~/.config/chartr` itself
  (`MkdirAll` will not chmod a directory that already exists)
- `user.toml` (the agent library) is still `0644` until its next write
- the running `chartr.app` instance started 2026-07-27, so it is still writing
  0644 payloads until it is restarted on a build carrying the fix

One-time close: `chmod -R go-rwx ~/.config/chartr`, plus restarting the desktop app.

### B. The `[::1]` origin pattern is inert

coder/websocket matches `OriginPatterns` with `path.Match`
(`accept.go:243-264`), which reads `[::1]` as a character class — so the pattern
`http://[::1]:PORT` can never match a literal `[::1]` origin. Confirmed: sending
`Origin: http://[::1]:8834` to a server bound `127.0.0.1:8834` returns 403.

This is **not** a hole — it fails closed, and every real browser path it was
meant to cover is already admitted by the library's own Origin-equals-Host rule
(a browser at `http://[::1]:PORT` sends that as both Origin and Host). But
`origins.go`'s comment claims a loopback bind admits "the three loopback
spellings" when only two of the patterns can ever fire. Either drop the `::1`
pattern or correct the comment.

### C. Scheme is not enforced on the same-host fast path (informational)

The library authorizes when `Origin`'s host equals `Host`, ignoring scheme, so
`Origin: https://127.0.0.1:PORT` is admitted (probe returned 101). Not
exploitable: an attacker would need to serve TLS on the port chartr already
holds. Library behaviour, not something this repo introduced.

## Reproducing

```sh
go build -o /tmp/chartr ./cmd/chartr
/tmp/chartr -addr 127.0.0.1:8834 &

ws() { curl -s --max-time 3 -o /dev/null -w "%{http_code}\n" \
  -H "Connection: Upgrade" -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Version: 13" -H "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==" "$@"; }

ws -H 'Origin: http://evil.example' http://127.0.0.1:8834/ws/control        # 403
ws -H 'Origin: http://127.0.0.1:8834' http://127.0.0.1:8834/ws/control      # 101
curl -s -o /dev/null -w '%{http_code}\n' -H 'Host: evil.example:8834' \
  http://127.0.0.1:8834/api/health                                          # 403
curl -s -o /dev/null -w '%{http_code}\n' -H 'Content-Type: text/plain' \
  -d '{"path":"/tmp/x"}' http://127.0.0.1:8834/api/spaces                   # 415
```

Note: registering a space through the API writes to the real
`~/.config/chartr/spaces.toml`. The test space used for this validation was
deregistered afterwards and the registry verified clean.

## Regression coverage

The behaviour above is pinned by tests, so a later change cannot quietly undo it:

- `origin_gate_test.go` — foreign origin refused on both sockets; refused
  terminal handshake writes nothing to the PTY; bound origin still streams;
  loopback alias admitted
- `origin_gate_dev_test.go` — the Vite origin is refused without the dev tag
- `hostgate_test.go` — foreign Host refused on API, SPA and both socket routes;
  every loopback spelling still reaches every route; right name / wrong port refused
- `contenttype_test.go` — simple-content-type POST registers nothing; bare POST
  opens no shell; parameterised JSON accepted; bodyless GET/DELETE unaffected
- `filemode_test.go` — payload, registry, agent library and config root are
  owner-only; the run marker stays a repository file; an older world-readable
  registry is repaired on save
- `exposure_test.go` — non-loopback bind warns and says what it exposes;
  loopback bind stays quiet
