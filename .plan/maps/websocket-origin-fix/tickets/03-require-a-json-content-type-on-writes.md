---
type: task
claimed_by: s3a9b9a7a72d4
claimed_at: 2026-08-03T08:18:43Z
---

# Handlers decode JSON regardless of Content-Type, so a POST needs no preflight

## Question

Every handler that takes a body decodes it unconditionally — `handleRegister`
(`internal/server/spaces.go:72`), `handleSpawn` (`internal/server/spawn.go:70`),
`handleLaunch` (`internal/server/terminals.go:61-63`), and the rest all call
`json.NewDecoder(r.Body).Decode(...)` without looking at `Content-Type`.

A cross-origin `POST` with `Content-Type: text/plain` is a **CORS-simple request**:
no preflight is sent, so the browser delivers it and the server acts on it. The
attacker cannot read the response, but every one of these routes is a side effect
worth causing blind. The `POST` routes reachable this way include:

- `POST /api/spaces` — registers an arbitrary path and **runs `git init` in it** if
  it is not already a repository (`internal/registry/registry.go:288-294`).
- `POST /api/spaces/{id}/terminals` — opens a shell.
- `POST /api/spaces/{id}/launch` — spawns a real agent CLI with an
  attacker-controlled `context` string written into the payload the agent is then
  told to read (`terminals.go:94-126`). The skill must resolve as `on-ramp`, so this
  is not arbitrary command execution — it is **prompt injection into an agent
  session**, which for this product is close enough to matter.
- `POST /api/spaces/{id}/maps/{slug}/tickets/{num}/spawn`, the session
  resume/respawn/release routes, `/api/config/open`, `/api/config/create`, and
  `/api/spaces/pick`.

Space IDs are not secret: `sha256(abspath)[:12]` (`registry.go:515-516`), derivable
by anyone who can guess a path.

**What is already safe, and why it matters here.** `PUT /api/config/agents/{name}`
and the `DELETE` routes (`server.go:155,185-186,215`) are *not* reachable this way —
those methods force a preflight the server will not answer. That is load-bearing:
`PUT /api/config/agents/{name}` registers an agent command, which would be direct
arbitrary execution. The method split is doing real work by accident, and this
ticket should not disturb it.

**This is defence in depth, not the fix.** Once tickets 01 and 02 land, a
cross-origin page cannot usefully reach these routes anyway, and a rebinding
attacker is same-origin and can set any Content-Type they like. Requiring a JSON
content type therefore closes nothing on its own — it is the cheap second lock that
holds if one of the other two is ever misconfigured or regressed. Work it after 01
and 02, and do not describe it as the remedy.

**The fix.** Reject a body-bearing request whose `Content-Type` is not
`application/json` (parse with `mime.ParseMediaType` so a `; charset=utf-8`
parameter is accepted, and match the media type case-insensitively) with a 415.
Apply it in one place rather than per-handler — the Host wrapper from ticket 02 is
the obvious seam, gated to requests that carry a body — so a route added later
inherits it. Check that the frontend actually sets the header on every call it
makes before landing this; a route the SPA posts to bare will break, and the point
is to find that now rather than in a release.

Tests lead: a `text/plain` POST to `/api/spaces` is refused with 415 and **no
directory is registered and no `git init` runs** — assert the effect, not just the
status. A companion asserts an `application/json; charset=utf-8` POST still
succeeds, and one pins that a `GET` with no body is unaffected.

Done when: state-changing routes refuse a non-JSON content type in one shared
place; the SPA still works against every route it calls; the tests above exist and
fail without the fix; and `go vet ./...` / `go test ./...` pass.

## Answer

**A write must declare a JSON body, checked once above the mux.** Commit
`b90ffb9`.

Ticket 02's wrapper is now `gate` (`internal/server/gate.go`) and holds the
checks that must hold for *every* route: the Host must name the address we
bound, then a write must declare `application/json`. `hostgate.go` keeps the
host rule it always had; nothing about it changed but where the wrapper lives.
An undeclared write is refused with a plain 415 before it is routed, so a route
added later inherits the check rather than remembering it.

**The rule, and why it is not exactly "requests that carry a body".** The
ticket's fix paragraph says to gate body-bearing requests; its next paragraph
says to check the SPA first because *"a route the SPA posts to bare will
break"*. Under the body-only reading nothing posted bare would break, so those
two only agree if a bodyless write is in scope — and it should be. What
`mustDeclareJSON` requires is:

- **every POST**, body or none. It is the one state-changing method a browser
  sends cross-origin with no preflight, and the bodyless POSTs are the ones
  worth causing blind: `/api/spaces/pick` raises the operator's folder chooser,
  `/api/spaces/{id}/terminals` opens a real shell, `/api/spaces/{id}/tracker-adapter`
  writes a file into their repository. A bare `fetch(url, {method: 'POST'})`
  carries neither body nor content type, so gating on the body would have left
  every one of those exactly as open as it was.
- **anything carrying a body**, whatever the method — a body is bytes a handler
  will decode, and an undeclared one has no business being decoded.

Left alone, deliberately: a bodyless GET (both websocket handshakes are one, and
asking a handshake for a content type would take the cockpit down), and a
bodyless DELETE or PUT — nothing to decode, and neither is a method a browser
sends cross-origin without the preflight this server never answers. That method
split is what keeps `PUT /api/config/agents/{name}` out of reach, and the ticket
said not to disturb it; `TestABodylessDeleteNeedsNoContentType` pins that it
still works without a header, so a later tightening cannot break it silently.

The header is parsed with `mime.ParseMediaType`, so `application/json;
charset=utf-8` is the same declaration as a bare one, and the media type is
matched case-insensitively.

**The SPA change the ticket asked me to look for.** `web/src/lib/actions.ts` is
the only client in the tree (nothing in `cmd/`, the webview shell, or the
scripts speaks to the API), and `send` set the header only when it had a body —
so seven actions posted bare: pick-folder, open terminal, seen, release (both
spellings), and install/dismiss tracker adapter. It now declares JSON on every
POST, body or none. That is the whole
frontend diff; `check`, `build` and `vitest` (217) pass, and the built CSS has
no amber.

**Against each Done-when clause.**

- *State-changing routes refuse a non-JSON content type in one shared place* —
  `gate`, above the mux, before routing.
- *The SPA still works against every route it calls* — every call goes through
  `send`, which now declares JSON on each POST; the rest are GETs and bodyless
  DELETEs, which are untouched. The dev proxy is unaffected: Vite forwards the
  header as sent.
- *The tests exist and fail without the fix* — `internal/server/contenttype_test.go`.
  `TestASimpleContentTypePostRegistersNothing` posts `{"path": …}` to
  `/api/spaces` under each of the three CORS-simple content types and asserts
  the **effect**: no `.git` in the folder it named and no space in the snapshot.
  `TestABarePostOpensNoShell` does the same for the bodyless case against
  `/api/spaces/{id}/terminals` — no shell in the snapshot — and then opens one
  declared, so the refusal is the content type's doing and not a broken route.
  Verified by short-circuiting the check and re-running: both fail, the
  registration one reporting `git init` actually ran and the space actually
  registered. The three positive tests
  (`TestJSONWithAParameterOrOddCasingIsAccepted`,
  `TestAGetWithNoBodyIsUnaffected`, `TestABodylessDeleteNeedsNoContentType`)
  pass either way, which is what says they are not vacuous.
- *`go vet ./...` / `go test ./...` pass* — both, and also under
  `-tags chartrdev`.

`PostWithContentType` (`internal/chartrtest/rig.go`) is the one new seam: it
posts a body verbatim under a content type of the caller's choosing, and an
empty one omits the header entirely — which is how the bare-POST case is put on
the wire at all, since `http.Post` always sets one.

**Deliberately not done.**

- **Nothing per-handler.** No handler learned to check its own content type, and
  none should; the point is that the check cannot be forgotten on a new route.
- **No 415 body worth reading.** Plain `http.Error`, like the gate's 403. The
  SPA falls back to its status-line message, which is correct — a cockpit that
  reached this refusal has a bug, not something to show the operator.
- **DELETE and bodyless PUT still need no header**, per the ticket's warning
  about the method split. If a future ticket wants *every* write to declare
  JSON, it is one line in `mustDeclareJSON` plus the rig's `Delete`.
- **No second every-route check beyond these two**, and no speculative hook for
  the trust-boundary map's ticket 06.

**One flag for a human**, unchanged from tickets 01 and 02: the map's disclosure
note applies to this commit too. It sits on `main` unpushed, and pushing
`.plan/` before a release publishes the exploit path. Nothing here pushes.
