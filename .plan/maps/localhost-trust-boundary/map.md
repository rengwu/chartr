# Localhost trust boundary

## Destination

chartr has a written trust model: a stated answer to *who can reach this and what
are they allowed to do*, arrived at by argument rather than inherited from a
convenience. Every question the security report raised has a decided position —
authentication or not, what happens on a non-loopback bind, whether the desktop
shell differs from the CLI — and there is a standing rule a new route can be checked
against before it ships. Done when a session adding a route can tell from the map
what that route must satisfy, and when the assumption that produced the websocket
hole has either been defended in writing or replaced.

## Notes

**Why this map exists.** A reviewer evaluating chartr reported six issues privately
on 2026-08-02. Five are ordinary bugs and are being fixed on the companion map
[`websocket-origin-fix`](../websocket-origin-fix/map.md). The sixth thing is not on
that map because it is not a bug: all of them descend from a single assumption that
was never written down, never tested, and is stated out loud in the code that
carries the worst of them —

> *"The cockpit is a single-operator tool bound to localhost and reached through
> the Vite dev proxy in development, so the cross-origin Host check would only get
> in the way here."* — `internal/server/control.go:23-25`

That sentence is the map's subject. It is not obviously wrong; it is undefended.
Fixing the two websockets closes the hole it opened without touching the reasoning
that will open the next one.

**This map does not block the fix, and the fix does not pre-empt this map.** The
companion map ships first, on purpose — the remedy there is known and there is
disclosure time to spend. Nothing settled here should be waited on before releasing
it. Equally, nothing the fix does forecloses a stronger answer landing later on top.

**A hard rule for this map: the fog is real, so nothing here is decided by
default.** The temptation is to reason from "it's localhost, it's fine" — which is
the premise under review. Every ticket must reach its answer through an argument
that survives the browser being a hostile client, because that is what the report
demonstrated it is.

**The standing preference, named so it can be argued with rather than absorbed.**
This project leans toward trust at the gate: minimal enforcement, few guards,
configuration believed rather than policed. That instinct is *what wrote the comment
above.* It may still be right — a single-operator local tool that adds a login
screen has lost something real — but it now has a counter-example with a working
exploit against it, and it must win the argument on tickets 03 and 04 rather than be
inherited from them. A session that finds itself concluding "no enforcement needed"
should be able to say what evidence would have changed its mind.

**Measured, not assumed.** Ticket 01 is research for the same reason the
agent-state-detection map opened with recordings rather than intuition: several
mature tools solved exactly this problem, some of them after their own incident, and
their answers are cheap to read and expensive to re-derive.

**What a settled answer looks like.** Not a policy document. A short section in the
map's Decisions plus, where it has teeth, an ADR — the trust boundary is
architectural in the same way ADR 0010's chrome/island split is, and a future
session changing it should have to argue with a numbered decision.

## Decisions so far

<!-- one line per resolved ticket: gist + link. Empty until the first ticket ships. -->

- **02 — the boundary is client admission, not the process or the port**: the original "single operator, bound to localhost, therefore safe" assumption is **replaced**, not defended. For a browser client the security identity is the exact origin of the page, with `Host` protecting that decision from DNS rebinding rather than substituting for it. **In** the trust set: the one operator, through a chartr UI document served from an origin this instance admits. **Out:** every other tab in that browser, other local processes *including agents chartr spawned*, other OS users, every network peer. Authority **and** disclosure are protected in their own right — scrollback, paths, branch state and snapshots are assets, so a push-only socket is not harmless and a read-only route is not outside the boundary. The Vite dev origin is an *admitted second principal* added per run, never a disabled gate. Knowingly accepted limitation: chartr cannot isolate code that already controls the operator's OS account. [ticket](tickets/02-what-is-the-trust-boundary.md)
- **03 — one per-process capability, required at one gate, valid for the process lifetime**: authentication of an admitted client, not an account system — no users, passwords, roles or login screen. A 32-byte capability is generated at server start, held **only in memory**, and required at a single ingress middleware for every route that reveals cockpit data or exercises authority, both WebSockets included; browsers must pass Origin and Host *as well*. Spawned agents are answered explicitly: they are outside the boundary and never receive it. **Amended 2026-08-03** after the first answer's consumed one-use nonce made restarting the accidental recovery path for a lost cookie: the nonce is deleted, one *reusable* capability replaces two secrets, and bootstrap collapses into a branch of the same middleware — `?k=<capability>` on any `GET` sets an `HttpOnly`/`SameSite=Strict` cookie and `302`s to the clean URL, so cookies then carry both sockets and no bootstrap endpoint, nonce state or Vite proxy wiring exists. **No expiry, no rotation, no consumption**: the capability is valid exactly as long as its process, so nothing can go stale independently of the thing it admits. A lost browser cookie is recovered by re-opening the bootstrap URL, reprinted on `Enter` by the running CLI reading its own stdin. Knowingly accepted: the URL lives in terminal scrollback for the process lifetime (Jupyter's posture) — but never in files, `~/.config`, the shell lock, argv, child environments or logs, because an agent can `cat` a file and cannot trivially read process memory. **Restart is not a recovery path.** [ticket](tickets/03-does-the-api-need-authentication.md)
- **05 — one posture, two delivery adapters; the shell signals its window rather than handing over a URL**: the ephemeral `127.0.0.1:0` bind is collision avoidance and is credited only as incidental friction — no admission decision may depend on an attacker failing to find the port. The webview is a controlled *delivery channel*, not a different principal: an ordinary browser can reach the same listener and Origin cannot tell the containers apart, so User-Agent or marker-global checks would be forgeable hints. Its one real advantage is that the launcher controls the first navigation, which after the 03 amendment makes the desktop bootstrap a `?k=` parameter on the existing `Navigate` call — zero paste, nothing printed, nothing in `.chartr/shell.lock`. **Amended 2026-08-03** for the second-launch case the first answer left open (`raiseInstance` is false on every non-macOS platform, so today's fallback prints the running URL, which authentication turns into a dead instruction whose only apparent remedy is quitting a live cockpit): a second launch now **sends `SIGUSR1` to the PID in the lock and exits**, and the running shell raises its own window. Not an HTTP route — signal delivery is same-user-only by the kernel, so the residual abuse case is strictly smaller than what that user already has and needs no rate limiting. The second process never receives the capability, the URL, or a reason to quit anything. Webview cookie-loss recovery is deliberately **not** built, with the two-line fix and its trigger written down. [ticket](tickets/05-is-the-desktop-shell-a-different-posture.md)

## Not yet specified

- **The remaining tickets.** Non-loopback binds (04) and the rule new routes must
  satisfy (06) are still fog; both now carry the constraints 03 and 05 settled, so
  they decide what is left rather than re-deriving the credential.
- **The denial page, which 03 has now produced a need for.** An operator whose cookie
  is gone lands on it, and it is the surface that tells them to press `Enter` in the
  chartr terminal — so it is load-bearing for the recovery flow, not a stub 403. It
  is frontend work under CLAUDE.md and ADR 0012 and needs its own ticket; note that it
  is served on the *unauthenticated* surface, so it cannot use anything the cockpit
  fetches after admission. A bound-address indicator may join it if 04 produces the
  need.

## Out of scope

- **The five bug fixes.** They belong to the companion map and are being worked
  there. A ticket here that finds itself specifying a patch has drifted.
- **Multi-user or multi-operator chartr.** The product is one operator at one
  machine (spec). Nothing here is an opening move toward accounts, roles, or
  sharing, and a ticket that concludes otherwise should say so loudly rather than
  quietly widen the product.
- **TLS and remote access.** chartr over a network to a remote machine is a
  different product with a different threat model. Ruled out here so ticket 04 can
  answer the `-addr` question without being drawn into it.
