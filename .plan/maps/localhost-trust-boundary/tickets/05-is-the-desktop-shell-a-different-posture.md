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
