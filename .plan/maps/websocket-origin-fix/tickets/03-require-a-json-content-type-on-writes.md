---
type: task
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
