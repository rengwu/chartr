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
