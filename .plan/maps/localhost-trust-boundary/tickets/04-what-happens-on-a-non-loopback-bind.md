---
type: grilling
blocked_by: [02, 03]
---

# What happens when `-addr` is not loopback?

## Question

`cmd/chartr/main.go:34` defaults to `127.0.0.1:8787` and binds whatever it is
given. `README.md:54` shows `chartr -addr :9000` as an ordinary option. That
wildcard bind puts an unauthenticated API that opens shells and spawns agents in
front of everyone who can reach the port — no browser, no cross-origin trick, none
of the fixes on the companion map touching it.

The fix map's ticket 04 adds a startup warning, deliberately scoped as documentation
because it changes no behaviour. This ticket decides the behaviour.

- **Warn, refuse, or gate?** A warning respects the operator and breaks nobody, and
  is what the fix map ships. A refusal is the only option that actually prevents the
  outcome, and it breaks anyone doing this today. A gate — a credential when the
  bind is wide, an explicit acknowledgement flag — is the middle. Ticket 03's answer
  constrains this: if chartr has a credential mechanism, "require it off loopback"
  is nearly free; if it does not, refusal and warning are the only real choices.
- **Who binds wide today, and for what?** This is the load-bearing unknown and the
  answer should not guess at it. Reaching the cockpit from another machine on a home
  network; a VM or container where the port must be forwarded; a remote dev box over
  SSH. The container case deserves particular care — binding `0.0.0.0` inside a
  container is routine and often *correct*, and a blanket refusal would break a
  legitimate deployment to prevent a misuse. Find out whether chartr in a container
  is a real usage before deciding against it.
- **Is loopback the right test?** A bind to a specific LAN address is narrower than
  `0.0.0.0` but still exposed. A bind to a Tailscale or WireGuard address is exposed
  only to a network the operator already trusts, and treating it like the open
  internet would be wrong in a way operators would resent. Decide whether the rule
  keys on loopback-or-not or on something finer, knowing that finer means a
  classification chartr has to get right on every platform.
- **What does the interface owe here?** If chartr can be running wide, should the
  operator be able to *see* that from the cockpit rather than only from a log line
  they scrolled past at startup? Decide whether this needs a visible indicator. If
  it does, it is frontend work under CLAUDE.md and ADR 0012 and needs its own
  ticket — do not design it here.

**Note what makes this different from the other tickets.** Everything else on this
map defends against someone acting on the operator without their knowledge. This is
the operator doing something deliberate, to their own machine, having read a flag in
a README. The paternalism question is real and should be answered rather than
resolved by security reflex: an operator who wants to bind wide on a network they
control is not making a mistake, and a tool that refuses them has taken something
away. Whatever is decided, say what the operator can still do afterwards.

Done when: a decided rule for non-loopback binds with the argument behind it; the
container and private-network cases each answered rather than lumped in; the
loopback-or-finer question settled; and the existing `-addr` behaviour either
preserved with a stated justification or changed with a stated migration for whoever
is using it today.
