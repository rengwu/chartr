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
