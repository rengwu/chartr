---
type: grilling
blocked_by: [02]
claimed_by: se0219db4c903
claimed_at: 2026-08-02T18:23:34Z
---

# Does the API need authentication at all?

## Question

There is no authentication anywhere in the server. `s.mux` is served raw
(`internal/server/server.go:267`); every route answers anyone who can reach the
port. Once the origin and Host fixes land, a *browser* can no longer be turned
against the cockpit — but nothing else changes. Decide whether that is sufficient
or whether chartr needs a credential.

Given ticket 02's boundary, settle:

- **What does origin-plus-Host actually buy, and what does it leave?** It stops the
  remote web page, which was the whole reported attack. It stops nothing that can
  already make a request from the machine: another local process, a malicious
  dependency in a project the operator has open, an agent chartr itself spawned. Say
  whether those are in the trust set — ticket 02 should have answered it — and if
  any is out, whether a credential is the instrument that addresses it or whether
  something else is.
- **The uncomfortable one: chartr spawns agents that run arbitrary code and can
  make HTTP requests.** An agent in a session can reach the cockpit API and spawn
  more sessions, open shells, or register spaces. Is that a threat or a feature? It
  is at least *foreseeable*, and it is the case where "a local process is already
  trusted" is least convincing, because the local process is one chartr started on
  behalf of a ticket whose content may have come from outside. Decide it explicitly.
- **What would a credential cost?** Be concrete rather than hand-waving "friction":
  where the token lives, how the browser gets it on first load, what happens on the
  desktop shell where there is no URL to paste, what happens when the operator
  bookmarks the page, and what breaks in the Vite dev flow. Ticket 01's survey has
  Jupyter's and code-server's answers to exactly this. If the cost is genuinely too
  high, that is a finding — but it has to be a measured cost, not a felt one.
- **Is there a middle position?** A credential required only when the bind is not
  loopback; a credential on state-changing routes only; a confirmation step on the
  routes that execute rather than on the API as a whole. Name the ones considered
  and why they were rejected, so ticket 04 does not have to rediscover them.

**The standing preference gets its real test here.** Trust at the gate is a coherent
position and may well be the right one for a single-operator local tool — a login
screen on a program you launched yourself is close to theatre. But it now has a
counter-example: the last time this project decided a check would "only get in the
way," the result was remote keystroke injection into the operator's shell. Defend
the preference against that, or narrow it. A "no authentication" answer is
acceptable and possibly correct; an *unargued* one is not.

Done when: there is a decided position on authentication with the argument behind
it; the spawned-agent case is answered explicitly rather than left implied; the
middle positions considered are named with their rejection reasons; and if the
answer is no credential, it states what would change that — the specific
circumstance under which this gets reopened.

## Answer

> **Partly superseded — read the [amendment](#amendment--2026-08-03-one-reusable-capability-no-consumed-nonce) at the end of this file first.** The
> credential decision below stands. Its *bootstrap* — a consumed one-use nonce
> distinct from the capability — does not: it made restarting the accidental recovery
> path for a lost cookie. Every mention of a nonce, a one-use handoff, or a
> per-restart bootstrap cost below is superseded.

**Yes. Every server instance must generate a high-entropy, per-process bearer
capability and require it at the common server-ingress gate for every route that
reveals cockpit data or exercises cockpit authority, including both WebSockets.**
This is authentication of an admitted client, not an account system: there are no
users, passwords, roles, or login screen. The unauthenticated surface may contain
only the bootstrap exchange, a denial page, and inert static UI assets; it must not
contain health, configuration, snapshot, payload, or other cockpit state.

The credential supplements rather than replaces the browser controls. An admitted
browser client must present the capability **and** pass the exact Origin and Host
decisions from ticket 02. A non-browser client has no useful Origin proof, so it
must present the capability in an authorization header. Reaching loopback, knowing
the port, sending no `Origin`, or forging an allowed `Origin` proves nothing by
itself.

### What the controls buy

Origin plus Host stops the reported browser attack: an untrusted page cannot turn
the operator's browser into an admitted client, and DNS rebinding cannot substitute
an attacker-selected host for the cockpit origin. It does not distinguish the
operator from `curl`, another OS user, a repository dependency, or an agent process
that can open a TCP connection. The capability supplies that missing proof.

This does not pretend to sandbox arbitrary code with full control of the operator's
OS account. Same-user code may be able to inspect process memory, read browser state,
or otherwise steal a bearer capability. The useful improvement is still material:
the server no longer intentionally grants cross-space cockpit authority and
disclosure to every process that discovers a port. Lower-privilege local users and
agents confined to a workspace cannot cross the gate merely by making HTTP
requests. That is the narrower promise ticket 02 requires.

### Bootstrap and measured cost

- **Lifetime and storage.** Generate a fresh capability at each server start and
  retain it only in server memory. Do not persist it in project files, user config,
  the shell lock, or an environment variable inherited by child processes. A
  persistent config token would make bookmarks convenient but would also give the
  very agents and repository dependencies under review a stable file to target.
- **Browser CLI flow.** The launch message provides a one-time bootstrap URL whose
  nonce is distinct from the API bearer capability. A successful exchange consumes
  the nonce, places the capability in an `HttpOnly`, `SameSite=Strict` cookie, and
  redirects immediately to a clean URL, following the Jupyter and OpenVSCode
  pattern from ticket 01. The bearer capability must not appear in browser history,
  logs, referrers, or the browser process command line; if chartr later opens the
  browser automatically, it should use a permission-restricted redirect file or
  an equivalent one-use handoff. The clean URL can be bookmarked and reused while
  that instance and cookie live. After a server restart, a stale bookmark receives
  a useful unauthorized page directing the operator to the newly printed bootstrap
  URL. One fresh bootstrap per launch is the knowingly accepted CLI cost.
- **Desktop shell.** The shell navigates its WebView through the same bootstrap
  exchange before loading the cockpit, so the operator pastes nothing and policy
  does not fork. Its existing lock continues to store only the clean loopback URL,
  never the capability. If a second launch cannot raise the existing window, it
  must not print a credentialless browser URL that implies access will work;
  providing browser fallback would need a separate one-use handoff. Ticket 05 may
  choose a tighter native handoff, but not a weaker admission policy.
- **Vite development.** Development keeps the same credential gate and explicitly
  proxies a bootstrap endpoint as well as API and WebSocket traffic. The developer
  opens a per-run bootstrap URL on the exact admitted Vite origin; the proxy carries
  the exchange to the Go server, and the resulting cookie belongs to the Vite
  origin used for later proxied requests. A backend restart therefore requires a
  fresh dev bootstrap. This adds launch wiring and invalidates the current habit of
  opening an uncredentialed Vite URL, but does not require weakening the shipped
  binary or embedding a stable secret in frontend source.

The capability must never be added to agent payloads, agent environments, spawned
command lines, control snapshots, or ordinary logs. Tests and intentional
non-browser integrations may receive it explicitly through a test-only or
operator-controlled handoff; possession must be deliberate rather than inherited
from locality.

### Spawned agents are outside, not automation clients

An agent calling the cockpit API is a threat under the current product contract,
not a feature. A ticket can contain externally sourced instructions, and the agent
already has the authority intentionally delegated to its session and workspace.
Letting it discover the local port and then inspect other spaces, inject terminal
input, mutate global configuration, or spawn sessions is an unannounced privilege
expansion. chartr therefore does not give spawned agents the cockpit capability.

This does not prohibit a future automation interface. Such an interface would need
an explicitly delegated, least-privilege capability scoped to named operations or a
named space; it must not be implemented by leaking the operator's cockpit-wide
bearer token. That is a new design question, not an exception to this answer.

### Rejected alternatives and middle positions

- **No credential on loopback:** rejected because it makes the port the boundary
  and directly contradicts ticket 02's exclusion of local processes and other OS
  users. Origin and Host solve the hostile-browser case only; the research survey's
  no-auth examples do not protect assets comparable to chartr's combined terminal,
  agent, configuration, and cross-space disclosure surface as completely as the
  Jupyter/editor examples do.
- **Credential only on non-loopback binds:** rejected because reachability changes
  on a wide bind but the trust set does not. It would leave the explicitly named
  loopback adversaries admitted. A wide bind still needs this capability; whether
  it must also be refused or placed behind another secure transport remains ticket
  04's question.
- **Credential only on state-changing routes:** rejected because ticket 02 protects
  disclosure independently. Snapshots, paths, payloads, and scrollback are assets,
  and read endpoints can supply the information needed for a later attack. Splitting
  read and write also creates a route-classification trap every new handler can get
  wrong.
- **Confirmation only for execution:** rejected as admission. It neither protects
  reads nor identifies who requested the prompt, and a local caller could create
  prompt fatigue or race the operator. Confirmation can later be defense in depth
  for an unusually destructive action, not a replacement for the ingress gate.
- **Password, persistent API key, or OS-account identity:** rejected for this
  single-operator product. A password adds a login and recovery lifecycle without
  identifying anyone beyond the one launched instance; a persistent key creates a
  stable secret on disk; and TCP plus browser clients offer no portable proof of
  which same-user process expresses operator intent. A per-process capability gets
  the useful admission property at lower product and operational cost.
- **Route-specific tokens or handler checks:** rejected in favor of the settled
  trust-at-the-gate preference. Authentication belongs in one middleware boundary
  shared by HTTP and WebSocket routes. Individual handlers may enforce semantic
  invariants, but must not reinvent admission.

### Verdict and revisit trigger

The original no-authentication assumption is narrowed, not defended: loopback is a
reachability default, Origin and Host are browser proofs, and a per-instance bearer
capability is the common proof of possession. The accepted trade-offs are a fresh
bootstrap after each restart, extra Vite launch wiring, an authenticated test-client
handoff, and the inability of a clean bookmark alone to recover access to a new
instance. These costs are smaller than silently treating every local process as the
operator, and the cookie/bootstrap flow avoids turning them into a recurring login.

Reopen the mechanism if evidence shows the capability is routinely exposed to
spawned agents despite the handoff constraints, if a supported client cannot carry
the cookie/header proof, or if implementation cannot keep the secret out of
persistent or child-visible state. Reopen the product decision if chartr adds an
intentional automation API, multi-operator use, or remote access. A future OS-native
channel that can prove operator intent more tightly may replace the bearer handoff,
but it must preserve the same trust set; mere inconvenience is not a reason to
fall back to port-as-identity.

## Amendment — 2026-08-03: one reusable capability, no consumed nonce

**Supersedes** the *Bootstrap and measured cost* section's browser-CLI, desktop-shell
and Vite bullets, and the sentence in *Verdict and revisit trigger* naming "a fresh
bootstrap after each restart" as the accepted cost. Everything else in the answer
above stands: the credential itself, the single ingress gate, both WebSockets,
Origin and Host on top for browser callers, the memory-only rule, the exclusion of
spawned agents, and every rejected alternative already listed.

### Why this needed reopening

The original answer specified **two secrets** — a consumed one-use bootstrap nonce
distinct from the bearer capability — and never said how a *running* instance issues
a second nonce. Restarting therefore became the accidental recovery mechanism for a
lost cookie. That is disqualifying: chartr is a long-running cockpit holding live
terminals, sessions and in-memory state across weeks of work, and losing that to a
cleared cookie is a worse outcome than the disclosure the credential defends against.

The correction is a subtraction, not a new mechanism. A nonce that immediately mints a
cookie of exactly equal power is not stronger than the capability it exchanges for —
it only adds consumption state, a second secret to keep straight, and a recovery
cliff. Collapsing the two into one reusable process-lifetime capability removes the
cliff and, with it, four downstream complications the original answer had to invent:
a separate bootstrap endpoint with nonce state, a "separate one-use browser handoff"
for the desktop (ticket 05), an activation channel for a second desktop launch, and
bootstrap-specific Vite proxy wiring.

### The capability is valid for the process lifetime

One 32-byte random capability, generated at server start, held only in server memory,
with **no expiry, no rotation and no consumption**. It is valid exactly as long as the
process that minted it, and is invalid the instant that process ends. This is the
whole restart-free property: there is no credential state that can go stale
independently of the thing it admits, so no timer, sweep or renewal path exists to
interrupt a running instance. Time passing is not a security event here — the
capability's blast radius is a process the operator is actively using, and expiring it
on a schedule would impose a recurring re-authentication on the operator while doing
nothing to an attacker who already holds it.

### Bootstrap is a redirect, not an endpoint

The gate accepts the capability three ways, all in one middleware:

1. a `k` query parameter on any `GET`, which is the bootstrap;
2. the `HttpOnly` cookie that bootstrap sets, which is how browsers and both
   WebSocket handshakes carry it thereafter;
3. an `Authorization: Bearer` header, for tests and deliberate non-browser clients.

Bootstrap is a branch in the same middleware rather than a route of its own: a `GET`
carrying a valid `k` sets `chartr=<capability>` as `HttpOnly`, `SameSite=Strict`,
`Path=/`, with a long `Max-Age` so an ordinary browser restart is not a recovery
event, and `302`s to the same path without `k` under `Referrer-Policy: no-referrer`.
A `302` replaces the history entry, so the clean URL is what the operator bookmarks
and what the SPA ever sees in `location`. An invalid or absent capability gets the
denial page described below. Because cookies ride the WebSocket handshake to a
same-origin listener, the control and terminal sockets need no separate token
mechanism — which is what keeps the "one gate" claim true of the code and not just of
the prose.

The bootstrap URL is **reusable for the process lifetime**. Presenting it twice is
not an error, does not consume anything, and yields the same cookie.

### Recovery without restart

Two loss cases exist and each has a same-process answer:

- **A browser cookie is lost** — cleared, a second browser, a private window. The
  operator re-opens the bootstrap URL. If they no longer have it, the CLI **reprints
  it on demand**: the process reads its own stdin and prints the bootstrap URL again
  on `Enter`, and the denial page says so in as many words. This adds no file, no
  endpoint, no second secret and no unauthenticated surface, and the channel is
  restricted to whoever controls the terminal the cockpit is running in — which is a
  better proof of operator intent than anything the network side can offer. Detached
  launches simply lose the reprint, which is acceptable: a launch with no terminal
  has the URL in its log.
- **The desktop webview's cookie is lost.** Deliberately not solved; see ticket 05.

Both recoveries are the same process answering again, so live terminals, sessions and
in-memory cockpit state are untouched by construction. **Restart is not a recovery
path and must never be documented as one.**

### The exposure this knowingly accepts

A reusable capability lives in the CLI's terminal scrollback for the life of the
process. This is Jupyter's posture and it is accepted deliberately rather than
engineered around: the terminal is the operator's own channel, and ticket 02 already
concedes that code with full control of the operator's OS account is outside what
chartr can defend.

What the original answer forbade still holds, and holds for exactly this reason. The
capability must not reach project files, user config, `.chartr/shell.lock`, argv,
environment variables inherited by children, ordinary logs, control snapshots, or
agent payloads. The distinction is not ceremonial: an agent instructed to read a file
succeeds trivially, while one that must read another process's memory needs a debugger
and, on macOS, entitlements it does not have. Memory-plus-terminal keeps that gap;
a token file closes it. The clean URL after redirect stays bookmarkable and carries
nothing.

### Additionally rejected

- **A consumed one-use nonce distinct from the capability** — rejected as the defect
  being corrected. It buys no strength over the capability it mints, and its
  consumption is precisely what made restart the recovery path.
- **Expiry or periodic rotation of the capability** — rejected. It interrupts a
  running instance on a timer, which is the failure mode this amendment exists to
  remove, and it does not dislodge an attacker who has already stolen it.
- **A persisted capability or bootstrap-URL file, even at `0600`** — rejected. It
  would make recovery trivial, but hands the excluded party in ticket 02 — spawned
  agents and repository dependencies running as the operator — a stable file to read.
  This is the one convenience whose cost is a real narrowing of the trust set.
- **Restart as the recovery mechanism** — rejected outright, and named here so no
  later ticket can adopt it by silence. Losing live terminals and sessions to a
  cookie problem is a worse outcome than the one being defended against.
- **A separate bootstrap endpoint with server-side nonce state** — rejected as
  unnecessary once the capability is reusable; a query parameter and a `302` in the
  existing middleware do the same work with no state to keep.
- **Bootstrap-specific Vite proxy wiring** — rejected as unnecessary. Development
  adds the one exact Vite origin to the admitted set (ticket 02) and the developer
  opens the ordinary `?k=` URL through the proxy; the proxy forwards `Set-Cookie`
  already, so the cookie belongs to the Vite origin used for later proxied requests
  without a bootstrap-aware proxy rule. A backend restart still means a fresh dev
  bootstrap.

### Amended revisit trigger

The triggers above stand. Add: reopen if evidence shows the terminal-scrollback
exposure is materially exploited, if an operator's realistic workflow produces cookie
loss often enough that `Enter`-to-reprint is felt as a recurring login rather than a
rare recovery, or if any future launcher cannot present the capability without
persisting it. A revisit may make the capability *harder* to obtain; it may not
reintroduce a credential whose loss requires restarting the process.
