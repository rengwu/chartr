# Handoff — localhost authentication without restarts

**Date:** 2026-08-03 · **Repo:** `chartr` (directory still named
`wayfinder-harness`)

## Purpose of the next session

Continue the localhost trust-boundary planning conversation, specifically revising
the authentication and desktop-shell decisions so chartr can remain a genuinely
long-running application. Authentication loss, browser-cookie loss, a second
desktop launch, or ordinary recovery must never require or encourage a server
restart. Do not implement yet; settle and record the corrected decision first.

## Authoritative context — read rather than re-derive

- The map and its destination: [`../maps/localhost-trust-boundary/map.md`](../maps/localhost-trust-boundary/map.md)
- The settled trust set and protected assets: [`../maps/localhost-trust-boundary/tickets/02-what-is-the-trust-boundary.md`](../maps/localhost-trust-boundary/tickets/02-what-is-the-trust-boundary.md)
- The current authentication answer, which needs reconsideration: [`../maps/localhost-trust-boundary/tickets/03-does-the-api-need-authentication.md`](../maps/localhost-trust-boundary/tickets/03-does-the-api-need-authentication.md)
- The desktop posture answer from this session, which needs a corresponding
  amendment: [`../maps/localhost-trust-boundary/tickets/05-is-the-desktop-shell-a-different-posture.md`](../maps/localhost-trust-boundary/tickets/05-is-the-desktop-shell-a-different-posture.md)
- Comparable-tool evidence, especially Jupyter's token bootstrap:
  [`../maps/localhost-trust-boundary/tickets/01-how-comparable-tools-draw-this-line.md`](../maps/localhost-trust-boundary/tickets/01-how-comparable-tools-draw-this-line.md)

Relevant commits on `main`:

- `307e28b` — trust boundary
- `953636b` — per-instance cockpit authentication
- `76ad8a8` — one desktop/CLI admission posture

## New, explicit product constraint

The operator stated that it is **paramount that chartr remain a long-running app
without requiring any restarts**. Treat this as a hard requirement, not a UX
preference. In particular:

- capability expiry or periodic rotation must not interrupt a running instance;
- losing a browser cookie must be recoverable against that same process;
- reopening/double-clicking the desktop app must not make quitting the existing
  process the practical recovery path;
- an authentication failure must never trigger an automatic restart; and
- live terminals, sessions, and in-memory cockpit state must survive every
  authentication recovery flow.

## Why the current answers need reopening

Ticket 03 selected a distinct, consumed one-use bootstrap nonce. It does not state
how a running CLI instance issues another nonce after the browser cookie is lost.
Restarting would therefore become the accidental recovery mechanism.

Ticket 05 correctly chose one admission policy and an automatic desktop bootstrap,
but it described browser fallback as a future possibility. That understates current
behavior: `cmd/webview/main_webview.go` prints the running clean URL when a second
launch cannot raise the first window, and non-macOS `raiseInstance` always reports
false (`cmd/webview/menu_other.go`). Authentication would make that URL insufficient
unless a restart-free handoff replaces the current fallback.

These are planning defects, not implementation bugs yet. Do not quietly work around
them in code; amend/reopen the decisions explicitly.

## Direction reached in conversation

The user asked for the simplest secure design. The latest proposed verdict was:

1. Retain exact Host and Origin enforcement for all protected HTTP and WebSocket
   routes.
2. Retain one high-entropy capability generated per server process because ticket
   02 deliberately excludes spawned agents and arbitrary local processes from the
   trust set. Host/Origin alone fixes the reported hostile-page exploit but does
   not enforce that part of the boundary.
3. Make that capability valid for the entire server-process lifetime. Do not expire
   or rotate it merely because time passed.
4. Replace the distinct consumed nonce with a reusable bootstrap URL for that
   process lifetime. It exchanges the capability for an `HttpOnly` cookie and
   redirects immediately to a clean URL.
5. The CLI prints/provides the bootstrap URL; the desktop shell passes it directly
   to its own WebView. Either client can repeat the bootstrap without restarting
   if its cookie is lost.
6. A second desktop launch must activate the existing process without receiving
   its capability. The running process may raise its own window or open an
   authenticated browser itself. The second process gets only success/failure,
   never a token or bootstrap URL.

The user selected a **process-lifetime capability** and an **opaque desktop browser
handoff** when grilled. “Opaque” means the credential mechanics are invisible; the
operator may still visibly cause a browser/window to open. No token is copied,
pasted, or displayed in the desktop flow, persisted in the shell lock, returned to
the second process, or inherited by agents.

## Pressure still required before writing the amendment

The simplified reusable URL knowingly makes a bearer secret live as long as the
process. Grill where it may safely appear. The conversation leaned toward allowing
the CLI's owning terminal to display it (similar to Jupyter), while forbidding it
from project files, the shell lock, agent environments, ordinary logs, browser
bookmarks (the redirected clean URL remains bookmarkable), and child argv. Confirm
that this exposure is an acceptable enforcement of ticket 02 rather than silently
weakening that ticket.

The desktop activation channel also has a bounded abuse case: a local process might
be able to ask the running process to open/focus a trusted chartr window, even though
it cannot obtain the credential. Decide whether rate-limited window activation is
an acceptable availability nuisance. Do not solve this by returning the bootstrap
secret to the caller or by requiring a restart.

Finally, distinguish these two policies clearly:

- **one admission posture** remains the decision;
- **different delivery adapters** are allowed: terminal output for CLI, in-process
  navigation/activation for desktop.

The verdict must name rejected alternatives and a revisit trigger. The most
important rejected alternatives are no credential at all, time-based credential
rotation, consumed-only bootstrap with no same-process recovery, and restart as
recovery.

## Repository state and scope

The ticket 05 answer is committed at `76ad8a8`; the worktree was clean before this
handoff file was created. No authentication implementation exists yet, and none was
requested in this conversation. Stay within the planning map: revise the answers
and later map Decisions/ADR as the chartr lifecycle requires, then leave
implementation decomposition to a subsequent map.

## Suggested skills for the next session

*Apply only if available in that agent's environment.*

- **`grill`** — pressure-test the reusable process-lifetime bootstrap and activation
  trade-offs, then amend tickets 03 and 05 with a decisive restart-free verdict.
- **`domain-modeling`** — after the verdict, capture the distinction between
  admission policy, bootstrap capability, and launcher delivery adapter in the
  architectural vocabulary/ADR.
- **`to-tickets`** — only after the planning map is settled, decompose the shared
  ingress gate, bootstrap flow, CLI delivery, and desktop activation into
  implementation tickets.

## Files worth opening first

- `cmd/chartr/main.go` — current CLI prints one clean listener URL and serves until
  signalled.
- `cmd/webview/main_webview.go` — same server, ephemeral loopback listener, initial
  `Navigate`, and second-launch fallback.
- `cmd/webview/lock.go` — currently stores PID plus clean URL; it must never become a
  credential store.
- `cmd/webview/menu_other.go` and `cmd/webview/menu_darwin.go` — why second-launch
  activation differs by platform.
- `internal/server/server.go` — the future single admission/bootstrap ownership
  point; there must not be desktop-specific authentication middleware.
