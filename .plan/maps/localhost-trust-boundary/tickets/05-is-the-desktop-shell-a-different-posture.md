---
type: grilling
blocked_by: [02]
claimed_by: s90a2c59f4097
claimed_at: 2026-08-02T18:30:38Z
---

# Is the desktop shell's posture different from the CLI's?

## Question

chartr ships two front doors and they are not equally exposed, but they share one
server with one set of rules.

The desktop shell binds an ephemeral loopback port — `net.Listen("tcp",
"127.0.0.1:0")` at `cmd/webview/main_webview.go:85`, with the comment at lines 68-70
noting it exists so there is no fixed port to collide on. The reviewer observed that
this narrows the attack usefully but does not close it: JavaScript can scan loopback
over websockets fast enough to find an unknown port, so it is friction rather than
mitigation. The CLI, by contrast, sits on a documented default port and needs no
scan at all.

Decide whether the two should have the same rules:

- **Was the ephemeral port ever a security measure?** The comment says collision
  avoidance. If security was never the intent, it should not be credited as a
  control now — and if it *is* being credited, that should be deliberate and
  written down, with the port-scanning limitation stated beside it.
- **Does the shell have advantages the CLI lacks?** It controls its own webview, so
  it knows exactly which origin should ever connect, and there is no third-party
  browser with other tabs in it. That is a materially stronger position than the
  CLI's, and it may support a tighter default — but only if the webview is genuinely
  the sole client. Check whether an ordinary browser can also point at the shell's
  port while it runs, because if it can, the advantage is smaller than it looks.
- **Is the webview itself a client to be trusted?** It renders the same SPA and can
  navigate. `cmd/webview/external.go` already carries a URL allowlist for opening
  links, which the reviewer specifically praised — so the question of what the
  webview may reach has been answered once already, in one place. Decide whether the
  trust boundary from ticket 02 lands differently inside a webview than inside
  Chrome, and whether the existing allowlist is the right precedent to extend.
- **One posture or two?** Two postures means two code paths to keep correct and a
  standing risk that a fix lands on one and not the other — which is exactly the
  shape of the bug being fixed, where the same wrong option appeared at two call
  sites. One posture is simpler and safer to maintain but forfeits the shell's
  advantages. Decide, and if the answer is two, say what keeps them from drifting.

**Keep the packaging story in view.** The Linux desktop app is an AppImage and the
Mac path is a bundle; a decision that requires the operator to handle a token or a
URL has a different cost in a double-clicked application than in a terminal where a
line was already printed. Ticket 03's answer, if it lands on a credential, meets its
hardest case here.

Done when: the ephemeral port's status as security-or-not is settled and written
down; the webview's standing relative to a browser is decided; there is a stated
answer on one posture or two, with an anti-drift measure if it is two; and the
desktop packaging cost of ticket 03's answer is named.

## Answer

> **Partly superseded — read the [amendment](#amendment--2026-08-03-signal-the-window-never-hand-over-a-url) at the end of this file first.** The
> verdict below — one posture, a shell-specific bootstrap adapter — stands and is
> reaffirmed. Every mention of a "one-use" nonce or handoff is superseded by ticket
> 03's reusable capability, and the second-launch fallback below is replaced: a second
> launch signals the running window and never receives a URL.

**One admission posture, with a shell-specific bootstrap adapter.** The desktop
shell and CLI must use the same server-ingress policy settled by tickets 02 and 03:
an exact admitted Origin and Host for browser-shaped callers, plus the same fresh
per-process capability for every protected HTTP and WebSocket route. The shell may
prove possession more conveniently because it creates its own client, but it does
not get a weaker gate or a second definition of who is trusted.

### The ephemeral port is not a security control

`127.0.0.1:0` exists to avoid collisions and permit independently rooted shell
instances, exactly as its comment says. It usefully raises the discovery cost over
the CLI's documented port, but discovery cost is neither client identity nor
authorization. Browser JavaScript can scan loopback, and local processes can inspect
listeners. More decisively, the current shell deliberately writes the selected URL
to `.chartr/shell.lock` and may print it when a second launch cannot raise the first
window. The port was not designed to be secret and must not become a secret by
policy.

Keep the ephemeral bind for its operational benefit, but credit it only as incidental
friction and attack-surface reduction. No admission decision, test, or security claim
may depend on an attacker failing to find it.

### The webview is a controlled delivery channel, not a different principal

The shell's webview is the sole **intended** client, but not the sole possible one.
`srv.Serve(ctx, ln)` exposes the same ordinary HTTP server used by the CLI, and an
ordinary browser that learns the port can request it while the shell runs. Origin
cannot distinguish those containers: a page loaded from
`http://127.0.0.1:<ephemeral>` has the same origin whether WebKit/WebView2 or Chrome
renders it. User-Agent checks, injected marker globals, or “the shell chose this
port” would be forgeable client hints, not admission proofs.

The webview nevertheless has one material advantage: the native launcher controls
its first navigation. It can navigate to ticket 03's one-use bootstrap URL, receive
the `HttpOnly`, `SameSite=Strict` capability cookie, and be redirected to the clean
cockpit URL without asking the operator to copy a token. A browser that merely
discovers and opens the clean shell URL must see only the unauthenticated denial or
inert-static surface. If a future browser fallback is wanted, the shell must mint a
separate one-use handoff deliberately; knowledge of the port is not that handoff.

Once bootstrapped, the embedded document is trusted on exactly the terms ticket 02
sets for a browser tab: it is an admitted chartr-origin document, not a trusted
webview process wholesale. Navigation to another origin ends that admission, and a
compromise of the admitted document remains a client compromise. The shell should
therefore initial-navigate only to its own bootstrap/cockpit origin and keep external
pages out of the cockpit window.

The existing `openExternalURL` allowlist is the right **native-boundary precedent**:
JavaScript-to-native calls must expose a narrow operation and validate attacker-
influenced values again in native code; external navigation belongs in the real
browser and only absolute HTTP(S) URLs may cross that hook. It is not a replacement
for server admission, and duplicating its URL allowlist as an authentication rule
would confuse outbound navigation policy with inbound authority.

### Alternatives put under pressure

- **A weaker desktop posture based on captive webview plus unknown port** is
  rejected. The webview is not captive at the TCP boundary, the port is discoverable
  and intentionally recorded, and this would reinstate the port boundary ticket 02
  rejected. The next native-shell call site could then repeat the reported bug.
- **A separately tighter desktop protocol that refuses every ordinary browser** is
  also rejected for now. The common capability already prevents an unbootstrapped
  browser or local process from crossing the gate. Distinguishing WebView from Chrome
  would require another native credential or transport, another server mode, and a
  separate recovery/test story while adding little protection against compromise of
  the admitted SPA or the operator's OS account. It would also break the explicit
  browser-fallback possibility unless that path grew a third handoff.
- **One undifferentiated launch experience** is rejected even though policy is one.
  Making a double-clicked AppImage or macOS bundle print or prompt for a token would
  impose CLI mechanics where the shell has a safer automatic channel. Shared policy
  does not require identical bootstrap presentation.

### Packaging cost and anti-drift rule

Ticket 03's credential should be invisible in the packaged desktop experience. On
every launch the process generates the capability and one-use nonce in memory, then
the native shell navigates its webview through the common bootstrap exchange. The
operator handles no token, no terminal is required, and the shell lock retains only
the clean URL. A server restart naturally performs a new bootstrap; persisted
webview cookies confer nothing against the newly generated capability.

The cost is implementation and recovery plumbing: both AppImage and macOS bundle
launch paths must construct the bootstrap navigation before showing protected state,
and the current “print the running URL” fallback cannot imply that a fresh browser
will be admitted. Failure to raise the existing window should direct the operator
back to it, or explicitly create the separate one-use browser handoff ticket 03
requires. It must never put the bearer capability in argv, the lock, logs, or an
agent-visible environment.

Keep this from drifting by having exactly one server admission middleware and one
bootstrap exchange owned by `internal/server`; both launchers configure those shared
primitives rather than selecting an auth mode. Launcher differences stop at delivery:
the CLI prints/provides its one-use bootstrap, while the shell passes its one-use URL
to `Navigate`. Conformance tests for protected HTTP, both WebSockets, stale/consumed
bootstrap nonces, and clean-URL denial must exercise the shared server independently
of launcher. There should be no `desktopSkipAuth`, trusted-User-Agent, or webview-only
handler branch to forget in a future fix.

### Verdict and revisit trigger

The shell has a tighter bootstrap channel, not a different trust boundary. Accept the
small maintenance cost of two launch adapters in exchange for one admission policy
and a zero-paste desktop experience; accept that an ordinary browser can reach the
listener, while the server serves it only inert/denial content until deliberately
bootstrapped.

Reopen this decision if the shell moves to an OS-native transport that can prove
operator intent more strongly than a bearer capability, if the embedded engine
cannot reliably carry the common cookie/Origin/Host checks, or if evidence shows the
one-use navigation leaks into browser-visible, persistent, or child-process state.
Also reopen it if browser fallback becomes a supported desktop requirement rather
than a recovery possibility, because its handoff and UX will then need an explicit
contract. Port unpredictability or a webview brand string alone never qualifies as
evidence for a separate posture.

## Amendment — 2026-08-03: signal the window, never hand over a URL

**Supersedes** every reference above to a "one-use" bootstrap or nonce — ticket 03's
amendment replaced it with one reusable process-lifetime capability — and replaces
the *Packaging cost and anti-drift rule* section's treatment of the second-launch
fallback. The verdict itself is unchanged and is reaffirmed: **one admission posture,
with a shell-specific bootstrap adapter.** The ephemeral port is still not a security
control, the webview is still a controlled delivery channel rather than a different
principal, and `openExternalURL` remains the native-boundary precedent, not an
admission rule.

### First navigation is the whole desktop bootstrap

The shell holds the capability in memory already — it constructs the `Server`. Its
delivery adapter is one line at `cmd/webview/main_webview.go:163`: navigate to the
cockpit URL carrying `?k=<capability>` instead of the clean URL. The middleware sets
the cookie and redirects, and the operator sees a window that was simply already
signed in. Nothing is printed, pasted, or displayed; the capability never touches
argv, `.chartr/shell.lock`, logs, or an agent-visible environment. This is what the
answer above meant by a zero-paste desktop experience, and the reusable capability
makes it a parameter on an existing `Navigate` call rather than a handoff mechanism.

### A second launch signals the running process; it never receives a URL

The answer above left the second-launch fallback unresolved, and the obvious repairs
are all wrong in the same way. `raiseInstance` reports false on every non-macOS
platform (`cmd/webview/menu_other.go:28`), so today's fallback prints the running
instance's URL (`main_webview.go:102`). Under authentication that URL leads to a
denial page — an instruction that visibly does not work, whose only apparent remedy
is quitting the running shell. That makes restart the recovery path for an ordinary
double-click, which ticket 03's amendment rules out.

**The second process must not obtain the capability, and it does not need to.** The
lock already records the live shell's PID (`cmd/webview/lock.go:31`); that is a
sufficient channel. A second launch **sends `SIGUSR1` to the recorded PID and exits**,
and the running shell raises its own window from its signal handler. `raiseInstance`
becomes "signal, then trust the running process" on the platforms that report false
today, with the raise itself a `gtk_window_present` or the platform equivalent
dispatched onto the existing UI thread; macOS keeps the native path it already has.

This is deliberately not an HTTP route. An unauthenticated `/activate` endpoint would
add the only unauthenticated authority-bearing surface in the product and would be
reachable by any page that finds the port, whereas signal delivery is restricted to
the same OS user by the kernel — the same user who, as ticket 02 concedes, can
already drive the window manager directly. So the residual abuse case is not merely
bounded, it is strictly smaller than what the adversary already has, and it needs no
rate limiting to say so.

The second-launch failure message must stop printing the running instance's URL. When
the raise cannot be attempted or does not take, the honest message names the running
window and its PID and directs the operator to it. Quitting is never presented as the
remedy.

### Webview cookie loss is deliberately not solved

A webview's cookie jar has no operator-facing clear button and no second profile, so
the browser loss case that ticket 03's `Enter`-to-reprint recovery answers has no real
desktop counterpart. Building a desktop recovery path now would be speculative
plumbing on a failure nobody has seen.

Named so the omission is a decision rather than an oversight: if it ever happens, the
fix is to bind one more native function beside `__chartrOpenExternal` that
re-`Navigate`s the window through the bootstrap URL the shell still holds in memory,
and have the SPA call it on a `401`. That is a few lines and needs no new secret, no
server change, and no restart — which is exactly why it does not need building in
advance. **Trigger:** any reproducible report of a webview losing its cockpit cookie
while its process is still alive.

### Additionally rejected

- **Printing the running URL to a second launch** — rejected. It is an instruction
  that no longer works, and its implied remedy is quitting a live cockpit.
- **Passing the capability or bootstrap URL to the second process** — rejected. A
  second process is not a trusted client, the lock must never become a credential
  store, and the operator's answer is the window that already exists.
- **An unauthenticated HTTP activation route** — rejected. It would create the
  product's only unauthenticated authority-bearing surface, reachable from any page
  that discovers the port, to replace a signal that is already same-user-only.
- **Rate limiting the activation channel** — rejected as unnecessary once activation
  is a signal. Anyone who can send it can already raise or close windows directly.
- **A desktop-only credential, transport, or auth mode** — still rejected, as above.
  The delivery adapter differs; the posture does not. There must be no
  `desktopSkipAuth`, trusted-User-Agent, or webview-only handler branch.

### Amended anti-drift rule

Unchanged in substance and now cheaper to hold: one admission middleware in
`internal/server` owns the gate *and* the `?k=` bootstrap branch, and both launchers
configure that shared primitive rather than selecting a mode. Launcher differences
stop at delivery — the CLI prints its bootstrap URL and reprints it on `Enter`, the
shell passes it to `Navigate`. Conformance tests exercise protected HTTP, both
WebSockets, bootstrap redirect and cookie issuance, and clean-URL denial against the
shared server, independently of launcher. The second-launch signal path is a shell
test and touches no server code, which is itself the point.

### Amended revisit trigger

The triggers above stand, with "one-use navigation" read as "bootstrap navigation".
Add: reopen if a supported platform cannot deliver or handle the activation signal,
or if a desktop browser fallback becomes a supported requirement — in which case it
needs a deliberate delivery decision, because knowledge of the port is not one and
the second process still may not carry the capability.

## Amendment — 2026-08-03: no activation signal; the second launch says so and exits

**Supersedes** the *A second launch signals the running process; it never receives a
URL* section of the amendment above. Everything else stands: one admission posture,
the `?k=` first navigation, the ephemeral port as non-control, the webview as a
delivery channel, no desktop auth mode, and the deliberate omission of webview
cookie-loss recovery.

**Operator decision, taken during ticket 06's grill.** The amendment above was right
about what to *reject* and wrong about what to build in its place.

### Why this is being cut

The `SIGUSR1` design costs native window-raising code on exactly the two platforms
where it is hardest, to save one alt-tab:

- **macOS already has its native raise** (`raiseInstance`, `menu_darwin.go`) and is
  untouched either way. So every line of new work lands on the tiered platforms.
- **Linux needs a signal handler plus a `gtk_window_present` marshalled onto the
  existing UI thread** — cgo, from a signal context, on the AppImage that ADR 0011's
  amendment made a release gate.
- **Windows has no `SIGUSR1` at all.** Go does not define it there, and the shell is a
  real (best-effort) tier that builds and smoke-tests in CI. The amendment above
  specified a mechanism one supported platform cannot express, which its own revisit
  trigger anticipated — "reopen if a supported platform cannot deliver or handle the
  activation signal" — without noticing the trigger was already true when written.

### What replaces it

**A second launch prints that the first is running, names its PID, and exits
non-zero.** No signal, no handler, no per-platform raise, no new code beyond changing
one message. macOS keeps `raiseInstance` and its behaviour is unchanged.

The message must still obey the amendment above's real finding, which is the part
worth keeping: **it must not print the running instance's URL**, because under
authentication that is an instruction that visibly does not work, and its only
apparent remedy is quitting a live cockpit. It names the window and the PID and stops
there. Quitting is never presented as the remedy.

### What this knowingly gives up

On Linux and Windows, double-clicking chartr while it is running tells the operator it
is running instead of pulling the window forward. They alt-tab. That is a real but
small regression against a design that was never built, on the two platforms where the
desktop shell is explicitly best-effort (ADR 0011).

### Amended revisit trigger

The triggers above stand, minus the activation-signal one, which this removes by
removing the signal. Add: **build the raise when someone asks for it on a platform
they actually use.** If that happens, the shape is per-platform native activation
behind the existing `raiseInstance` seam — not an HTTP route, which the amendment
above rejected for reasons that still hold, and not a mechanism specified before it is
known which platform needs it.
