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

## Not yet specified

- **Everything the tickets ask.** This map opens with all of it as fog: the
  boundary itself (02), authentication (03), non-loopback binds (04), the desktop
  shell's posture (05), and the rule new routes must satisfy (06).
- **Whether any of this reaches the user interface.** If a decision here implies the
  operator must *see* something — a bound-address indicator, a warning banner on a
  wide bind — that is frontend work under CLAUDE.md and ADR 0012, and it is not
  scoped until a ticket produces the need.

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
