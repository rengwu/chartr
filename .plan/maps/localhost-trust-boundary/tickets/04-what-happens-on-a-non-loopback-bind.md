---
type: grilling
blocked_by: [02, 03]
claimed_by: s0b56ec50e0a7
claimed_at: 2026-08-02T19:20:03Z
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

**Constraints inherited from 03 and 05 — read these before answering.** Ticket 03 and
its 2026-08-03 amendment settled the credential, and this ticket does not get to
re-litigate its shape. Four things are already decided and would be defects to
reintroduce here:

- **The credential is unconditional.** The second bullet above ("a credential when the
  bind is wide") is stale — it was written before 03 landed. A per-process capability
  is required on *every* bind including loopback, so "require it off loopback" is not
  the middle position; it is already the floor. This ticket decides what a wide bind
  needs **in addition**, or whether it is refused, not whether the capability applies.
- **One posture, one gate.** A wide bind must not select a second auth mode, a
  stricter credential, a separate middleware, or a "remote" server variant. That is
  the drift shape ticket 05 spent its whole answer rejecting, and it is how the
  original two-`websocket.Accept` bug happened. Anything a wide bind needs is either
  a property of the one gate or a refusal at startup.
- **No expiry, no rotation, no restart-as-recovery.** Exposure does not make a timer
  a good idea; ticket 03's amendment rejected expiry outright, and the reasoning does
  not weaken off loopback. Whatever this ticket decides must leave a running instance
  running — an operator must never lose live terminals and sessions to an
  authentication event.
- **The capability stays out of persistent state.** A wide bind is exactly where a
  token file starts to look convenient. It is rejected in 03 for reasons that get
  stronger, not weaker, with more reachable callers.

**One genuinely new question this ticket owns.** The bootstrap URL now carries the
capability in a `?k=` parameter, and the CLI prints it at startup. On a wide bind the
address chartr prints is either meaningless (`0.0.0.0`) or a specific interface, and
the URL is a bearer secret in a context where more than one machine may be able to
reach the listener. Decide what is printed, over what transport that URL is expected
to travel, and whether an operator relaying it to another machine is a supported flow
or a documented hazard. This is downstream of the credential's shape, not a reason to
change it.

Done when: a decided rule for non-loopback binds with the argument behind it; the
container and private-network cases each answered rather than lumped in; the
loopback-or-finer question settled; the existing `-addr` behaviour either preserved
with a stated justification or changed with a stated migration for whoever is using
it today; and the bootstrap URL's presentation on a wide bind decided without
introducing a second posture, an expiry, or a persisted credential.

## Answer

**chartr refuses a non-loopback `-addr` at startup unless the operator also passes
an explicit flag that names the consequence — `-expose-cleartext` — and the flag
changes nothing whatsoever after the listener is created.** The default becomes
loopback-only; the escape hatch is one deliberate token, not a warning; the
capability, the gate, the Origin/Host checks and the bootstrap flow from 03 and 05
are untouched. This is a behaviour change to a documented flag and it ships in the
same release as the capability, as one migration.

The rule keys on **loopback or not** and nothing finer. What is printed on an
exposed bind is the ordinary loopback bootstrap URL, plus a plain statement of the
interfaces the listener answers on **without** the capability appended. Relaying
the bootstrap URL to a second machine is a **documented hazard, not a supported
flow.**

### What actually changes on a wide bind, once 03 has landed

Ticket 03 removes most of what makes today's `-addr :9000` alarming. A network peer
who reaches the port meets the same 32-byte capability everyone else meets, and
reachability is not admission (02). So the honest list of what a wide bind still
changes is short — and the first item is decisive:

1. **The disclosure half of the boundary loses its only control.** chartr has no
   TLS and it is ruled out of scope for this map, so an exposed listener carries the
   cockpit in cleartext. Ticket 02 protects disclosure *in its own right*: terminal
   scrollback, keystrokes, repository paths, branch state, diffs, prompts, and
   whatever a command echoed. An on-path party reads all of it **passively, off the
   WebSocket frames, without ever presenting the capability.** The credential
   protects admission; it protects confidentiality not at all. On loopback that gap
   does not exist because there is no path to be on. Off loopback it is total.
2. **Active on-path injection compromises the admitted origin.** Cleartext HTTP means
   an attacker can rewrite the SPA. 02 already states that compromise of the admitted
   chartr origin is compromise of the client, so this is not a partial loss — it
   hands over the capability and the cockpit together.
3. **The pre-auth surface becomes remotely reachable.** The denial page, the static
   assets and the gate itself. Small — but every future pre-auth bug is remote
   instead of local, and there is no rate limiting anywhere.
4. **`Host` stops being well-defined.** A wildcard listener has no single address it
   is "listening on"; see the flagged constraint at the end.

Item 1 is the whole argument. The reflexive framing — "a wide bind is fine now,
because auth" — is wrong in exactly the way this map warns against: it treats
authentication as the boundary when 02 says the boundary protects authority *and*
disclosure. On an exposed cleartext listener chartr can enforce one of those two and
has no mechanism, present or in scope, for the other.

02's principle then decides it without any further balancing: **preserve the trust
set, require an explicit compensating gate, or refuse the configuration.** There is
no compensating gate available for disclosure — TLS is the compensating gate and it
is out of scope. So the configuration is refused. That is not security reflex; it is
the settled principle applied to the one case where chartr genuinely has no control
to offer.

### Why an operator-supplied flag rather than a flat refusal

Because refusal-with-no-escape is a claim chartr cannot honestly make, and the
paternalism objection in this ticket is right on the merits.

- **The operator knows something chartr cannot.** chartr sees a bind address. It
  cannot see whether that address is on a home LAN with two known devices, a
  WireGuard interface, or conference Wi-Fi. The difference between those is the
  entire risk, and only the human at the keyboard has it.
- **A flat refusal does not prevent the outcome, it relocates it.** `socat
  TCP-LISTEN:9000,fork TCP:127.0.0.1:8787` reproduces the exposure exactly, outside
  chartr, with no startup line, no documentation, and nothing chartr can say about
  it. Harm reduction favours the version that happens inside the tool that can
  describe it.
- **The flag is not a warning.** A warning ships regardless of consent; this is a
  precondition. The launch fails without it. That is what makes it satisfy 02's "a
  warning can explain the consequence but is not itself a boundary": nothing here
  relies on the operator reading anything.
- **The flag is not a compensating gate either, and must not be described as one.**
  It restores no control. It is the operator declaring that this particular network
  is outside chartr's threat model. Naming it `-expose-cleartext` rather than
  `-allow-remote` or `-insecure` is deliberate: it states the property being given
  up, so the person typing it learns the thing they need to know at the moment they
  need it. A bare `-force` or `-yes` shape is rejected — it teaches nothing and reads
  as a formality.

The counter I take most seriously is OpenVSCode Server's, from ticket 01: its
official Docker image ships `--without-connection-token` in the entrypoint, so the
escape hatch became the default for everyone who used the blessed artifact. That
failure mode is real and the defence is narrow but sufficient today — chartr ships no
image, no compose file and no systemd unit, so there is no upstream artifact that can
bake the flag in on an operator's behalf. This is written down as a standing
constraint rather than a hope: **if chartr ever publishes a container image or
service unit, that artifact must not contain this flag.** See the revisit trigger.

### The rule, precisely

- **Test:** the requested `-addr`, evaluated **before** `net.Listen`, so a refused
  launch never opens a port. Loopback means every address the listener would answer
  on is loopback: a literal IP for which `netip.Addr.IsLoopback()` holds (so all of
  `127.0.0.0/8` and `::1`), or a name that resolves only to such addresses. An empty
  host (`:9000`), `0.0.0.0` and `::` are wildcards and are **not** loopback. Failure
  to resolve fails closed.
- **Without the flag:** refuse. Exit non-zero with a message that names the address,
  says in one sentence that chartr has no TLS so the cockpit and its capability would
  cross the network in cleartext, and prints all three remedies verbatim — the
  loopback form of what they probably meant, the tunnel, and the flag. Nobody should
  have to search for the way forward.
- **With the flag:** bind exactly as asked. The capability is unchanged, the gate is
  unchanged, both WebSockets are unchanged, Origin and Host stay enforced, no timer
  and no persisted secret appears. Print one additional startup line stating the
  exposure.
- **The flag on a loopback address is itself an error.** A flag that claims exposure
  where there is none is a misconfiguration, and refusing it is what stops
  `alias chartr='chartr -expose-cleartext'` from surviving in a shell profile and
  silently arming a later `-addr` change.
- **`-addr` with a different *port* on loopback is untouched.** `127.0.0.1:9000` was
  always fine and stays fine.
- **The desktop shell is out of it by construction.** `cmd/webview` hard-codes
  `127.0.0.1:0` and has no `-addr` flag (ticket 05). It can never be exposed, sees no
  refusal, and **must not gain an `-addr` flag** — that would be the posture
  divergence 05 spent its answer rejecting.

**The compliance test for "one posture, one gate": the flag must change zero code
paths after the listener exists.** No second middleware, no stricter credential, no
"remote" server variant, no bind-conditional branch inside a handler. If an
implementation finds itself writing `if exposed` anywhere below `main`, it has
drifted. That is checkable and the implementation ticket should be judged against it.

### Loopback or finer: settled on loopback

Finer is rejected, and not narrowly.

- **A wildcard bind carries no network information at all.** `0.0.0.0` is every
  interface at once — present and future, including one a VPN brings up ten minutes
  later. There is no address to classify.
- **Classifying the *peer* instead is a second gate.** Keying on the remote address
  of each connection means a per-request ACL running alongside the capability check:
  precisely the second admission mechanism 03 and 05 forbid, and an IP allowlist is
  reachability, not identity — 02's rejection of the port boundary generalises to it
  without modification.
- **The private/VPN ranges do not mean what the rule would need them to mean.**
  RFC1918 is the address space of every café and coworking network, not a trust
  signal. Tailscale's `100.64/10` is shared CGNAT space that carrier NAT also uses. A
  WireGuard interface can be routable. chartr would be inferring "the operator trusts
  this network" from a number that does not carry it.
- **A wrong guess fails silently and completely.** A rule that admits `192.168/16`
  unprompted gives the café case a clean launch with no message at all — the worst
  possible outcome, arrived at by trying to be helpful.

So: loopback or not, no exceptions, no ranges, no interface-type detection. The
operator supplies the network judgement that chartr cannot compute, by typing the
flag. That is the same division of labour as the rest of the rule.

### The container case, answered on its own

**Under this rule chartr in a container works, with `-addr 0.0.0.0:8787
-expose-cleartext`, and no special case exists for it.** Keeping the flag rather
than refusing outright is partly *for* this case: a blanket refusal would break the
one deployment where a wildcard bind is genuinely correct.

But the ticket asked whether chartr in a container is a real usage, and the evidence
in this repository says it is not one chartr supports today:

- no `Dockerfile`, no compose file, no image published anywhere in the repo, and no
  mention of containers in `README.md` or `docs/getting-started.md`; the shipped
  artifacts are a `.dmg`, an AppImage and a CLI binary;
- and three shipped features assume the server process sits on the operator's own
  desktop, not merely on their own machine. `internal/server/folderpicker.go:19`
  says so in as many words — "chartr always serves on loopback, so a dialog the
  server raises lands on the operator's own desktop". In a container that native
  chooser raises into nothing. `handleOpenLayer` runs the `$VISUAL`/`$EDITOR`/OS-opener
  ladder server-side (`internal/server/configsurface.go:151`), and `env.HydratePATH()`
  adopts the operator's login-shell `PATH` so their agent CLIs are findable.

So the container answer is: **not blessed, not broken.** It works through the same
one flag as everything else, and an operator running it there is accepting a
degraded product — the picker and the editor-open action do not do what their names
say — on top of the cleartext exposure. If containerised chartr ever becomes a
target, that is a product decision with its own map, and the first thing it must
settle is those three features, not the bind.

### The private-network case, answered on its own

A phone or tablet on a home LAN, a spare laptop across the room, a tailnet: allowed,
through the same flag, and this is the case the flag mostly exists for. But the
refusal message and the docs must lead with the alternatives, because each of them is
strictly better than what the flag gives:

- `ssh -L 8787:127.0.0.1:8787 <host>` — chartr stays on loopback, the transport is
  encrypted and authenticated, and nothing about chartr changes;
- `tailscale serve 8787` — proxies a loopback port over the tailnet with TLS and
  tailnet identity in front of it;
- any local reverse proxy terminating TLS in front of `127.0.0.1:8787`.

All three keep chartr's own posture at the loopback default and add the confidentiality
chartr cannot supply. The flag is for the operator who cannot use any of them and has
judged the network acceptable. **What the operator can still do after this decision:
everything they can do today** — including the plain wide bind, at the cost of one
token that tells them what they are trading.

### The existing `-addr` behaviour: changed, with a migration

Changed. And the migration is smaller than it looks, because the documented example
is almost certainly not doing what its readers think:

```
chartr -addr :9000       # serve somewhere else   ← docs/getting-started.md:51
chartr -addr :9000                                ← README.md:54
```

`docs/getting-started.md:51` labels it "serve somewhere else". That is a *port*
change, and the wildcard bind is an accident of Go's address syntax — the shortest
way to write "port 9000" happens to also mean "every interface". Whoever is using
this today mostly wanted the port.

- **Wanted a different port** (expected majority): `-addr 127.0.0.1:9000`. Nothing
  else changes.
- **Wanted exposure**: add `-expose-cleartext`. Nothing else changes.
- **Both README and getting-started examples must be corrected to `127.0.0.1:9000`**,
  so the docs stop teaching a wildcard bind while demonstrating a port change.

**Sequencing.** The companion map's ticket 04 ships its startup warning as written and
this does not block it — that ticket already anticipated being replaced cleanly. This
refusal lands with 03's capability, in the same release, so operators absorb one
breaking change and read one release note rather than two.

### What is printed on an exposed bind

This ticket's own question. Today `main.go:69` prints `ln.Addr()`, which for a
wildcard is `http://0.0.0.0:9000` — not a URL that works from anywhere, and useless
as a `Host`. Decided:

1. **The bootstrap URL is always printed on the loopback authority**:
   `http://127.0.0.1:<port>/?k=<capability>`. It is the one form guaranteed to work,
   it is the form the operator's own browser should use, and `Enter`-to-reprint (03)
   reprints exactly it. `http://0.0.0.0:<port>` is never printed as though it were a
   URL.
2. **Exposure gets its own line, and the capability is not on it.** Echo back the
   address the operator asked for and say what it means — `exposed on :9000 —
   reachable from the network, in cleartext`. The capability appears exactly once in
   the output, attached to the local URL.

   chartr deliberately does **not** enumerate the interface addresses the listener
   answers on. Walking `net.Interfaces()` and filtering link-local and IPv6 forms is
   real cross-platform code bought for a cosmetic line, and the operator who wants
   their LAN address has `ifconfig`. Echoing the requested address carries the same
   information — that this launch is exposed, and on which port — at no cost.

The reason for (2) is narrow and I will not oversell it: an operator who wants the
combined URL will concatenate the two in five seconds, and that is fine — they did it
deliberately. What chartr declines to do is *produce* a ready-made
`http://192.168.1.20:9000/?k=<secret>` string, because that string is one paste away
from Slack, a screenshot, or a ticket, and 03's whole storage discipline exists to
keep this secret out of places it can be read later. Printing it pre-assembled is
chartr inviting the leak; making the operator assemble it is chartr staying out of it.

3. **Transport expectation: the operator's own terminal, and nowhere else.** That is
   what 03 accepted as the exposure surface, and this ticket does not widen it.
   Relaying the bootstrap URL to a second machine is a **documented hazard, not a
   supported flow.** No QR code, no pairing code, no second-device enrolment, no
   short-lived join token — every one of those is a second secret or an expiry, both
   of which 03 rejected, and inventing one here would smuggle in the second posture
   this ticket is forbidden to create. If second-device access ever becomes a product
   goal, the missing piece is a transport (TLS), not a weaker credential.

### The interface: no indicator, and here is the trigger that would earn one

The map asked whether 04 produces the need for a bound-address indicator in the
cockpit. **It does not — not with this rule.** The case for one was written against a
warn-only world, where an operator could be exposed without having decided to be.
Under refuse-plus-flag the exposure is never accidental: it exists only because the
operator typed `-expose-cleartext` on this launch. An indicator would spend a
snapshot field, chrome work and a whole ticket telling them something they typed.

The residual argument — a chartr process lives for weeks (03's own premise), so
"typed the flag on Monday, screen-sharing on Friday" is real — is not nothing, but it
is not enough to build against now. YAGNI: build it when someone is bitten.

**Trigger:** if operators do run exposed for long stretches, or if the flag turns out
to be common enough that a launch's posture is genuinely forgotten, add it — as one
token-styled line of fact in the chrome, present only when the listener is
non-loopback, riding the post-admission snapshot (so it discloses nothing to anyone
not already admitted). It would be frontend work under CLAUDE.md and ADR 0012, chrome
side of the ADR 0010 split, monochrome. Nobody should design it before then.

### Rejected alternatives

- **Warn only, keep binding whatever is asked** (the standing preference's answer, and
  what the fix map ships as an interim). Rejected because a warning is not a boundary
  (02) and, more concretely, because the thing it warns about has no control behind
  it: on a cleartext listener chartr's disclosure protection is zero, not weakened.
  This is the option that would have needed the capability to protect confidentiality,
  and it does not. What would have changed my mind: chartr having TLS, or the exposed
  assets being authority-only rather than a live terminal stream.
- **Flat refusal with no escape hatch.** Rejected. It cannot actually prevent the
  outcome — one `socat` line reproduces it with no warning and no docs — and it
  removes a legitimate use (container, home LAN with no SSH or tailnet available) to
  prevent a misuse the operator is better placed to judge than chartr is. It also
  makes chartr's refusal a lie about its own reach.
- **Finer classification: allow private/VPN ranges, refuse public.** Rejected above at
  length: a wildcard bind has no address to classify, per-peer classification is the
  forbidden second gate, RFC1918 is not a trust signal, and the failure mode of a
  wrong guess is silent and total.
- **A stricter credential, a second auth mode, or a "remote" server variant when the
  bind is wide.** Rejected on the inherited constraint, and independently: it is the
  drift shape that produced the two-`websocket.Accept` bug, and it would make the gate
  a thing with modes rather than a thing.
- **Expiry, rotation, or a shorter-lived capability off loopback.** Rejected. 03
  settled it, exposure does not weaken the reasoning, and a timer would not dislodge
  an on-path attacker who is reading the frames in cleartext anyway.
- **A token file, a `--token-file`, or any persisted credential to make second-device
  access convenient.** Rejected. This is exactly where it looks most attractive and
  the cost is highest: a stable file is readable by the spawned agents and repository
  dependencies 02 puts outside the boundary.
- **Auto-loosening the browser gates on a wide bind**, as Jupyter's
  `allow_remote_access` and Ollama's skipped Host middleware do (ticket 01). Rejected
  outright: 02 forbids configuration silently enlarging the trust set as a side effect
  of changing an address, and that is precisely what those two do. Where a remote
  hostname must be admitted, it is named explicitly — the additive-exception shape
  ticket 01 found in five of six tools.
- **`-addr` accepting a hostname allowlist, a `--trusted-origin`, or any new
  admission config as part of this ticket.** Deferred, not rejected: see the flagged
  constraint below. It belongs to whoever implements the Host rule, not here.

### Flagged doubts — read these, they are not rhetorical

- **A leaked capability has no remedy but restarting the process, and this is the
  first case where that bites.** 03 rejected restart as a *recovery* path for a lost
  cookie and rejected expiry; it did not consider restart as *incident response*.
  Those are different questions, and I am not reopening 03 — on loopback a leaked
  capability implies same-user code, which 02 already concedes. Off loopback, leaking
  it to another machine is realistic and the only cure is killing live terminals and
  sessions. Named here so the trade is explicit rather than discovered later: **if
  exposed use becomes common, the reopen is a live operator-initiated remint that
  keeps the process running, not an expiry.**
- **`Host` is ill-defined under a wildcard listener, and the companion map's ticket 02
  ("reject a Host we are not listening on") assumes it is not.** A wildcard listener
  answers on every interface, so the set is enumerable but changes as interfaces come
  and go, and a request arriving by *name* (`laptop.local`) is not in it at all. The
  flag must not turn Host enforcement off. Whoever implements the Host rule must
  decide what it means under `-expose-cleartext`; if a name needs admitting, it is
  named explicitly by the operator (Vite's `allowedHosts` shape), never inferred and
  never disabled. Flagging it rather than deciding it: it is the fix map's rule, and
  designing it here would be the drift this map warns about.
- **This rule belongs in the trust-boundary ADR as one clause, not its own ADR.** One
  boundary, one numbered decision (ticket 06 owns the ADR). A separate ADR for "the
  bind rule" would imply the bind is its own posture, which is the thing this answer
  denies.

### Revisit trigger

Reopen if any of these becomes true:

- **chartr ships a container image, compose file or service unit.** That artifact must
  not contain `-expose-cleartext`; if the deployment cannot work without it, the
  container question is a product decision, not a packaging detail, and OpenVSCode's
  image is the worked example of getting it wrong.
- **chartr acquires TLS, or remote access becomes a supported product.** The refusal
  exists only because no compensating gate for disclosure is available; supply one and
  the whole argument is different — the flag should then gate cleartext specifically,
  not exposure.
- **Operators are routinely passing the flag.** That is evidence the default is wrong
  for real usage, and the answer is to make the supported path (tunnel, or TLS) easier
  — not to relax the default.
- **Evidence that the flag is being pasted into shell profiles, dotfiles or scripts as
  a matter of course.** The consent it represents is per-launch and deliberate; if it
  becomes ambient, it has stopped meaning anything and needs a different shape.
- **A second-device flow becomes a product goal.** Then the bootstrap-URL relay
  hazard above must be replaced by a designed transport. It may make the capability
  harder to obtain; per 03 it may not reintroduce a credential whose loss requires
  restarting the process.
