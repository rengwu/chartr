---
type: research
claimed_by: s3d41a500e84d
claimed_at: 2026-08-02T18:09:05Z
---

# How comparable local-server tools draw this line

## Question

chartr is one of a large family: a program that binds a local HTTP port, serves a
browser UI, and exposes something dangerous behind it. Several of that family have
been through exactly the incident chartr just avoided, and their answers — including
the ones they tried and abandoned — are the cheapest evidence available for tickets
02 through 06.

Find out what each of these actually does *today*, and where the history is visible,
what they did before and why they changed:

- **Jupyter Notebook / Lab.** The reference point for "local server, arbitrary code
  execution, browser UI." What it does about origin and Host checking, and its token
  model — where the token lives, how the browser gets it, what the user experience
  costs. Jupyter has been through public vulnerabilities here; find what forced each
  change.
- **Vite** — directly relevant, because the dev proxy is the stated reason chartr
  disabled its check. Vite added host checking after a rebinding vulnerability of
  its own; find how `server.host`, `server.allowedHosts` and its origin handling
  landed, and what the default is now. This is the closest thing to a
  ready-made answer for ticket 01 of the fix map's dev-origin problem.
- **code-server / openvscode-server.** Remote-capable by design, so its posture is
  necessarily stronger — but note *what* it requires and at what point (password,
  proxy, tunnel), because it shows what the price of authentication actually is.
- **ttyd and gotty.** The closest functional analogues: a terminal over a
  websocket. What they do about origin, what their defaults are, and whether their
  documentation warns about exposure. gotty in particular has a documented history
  worth reading.
- **Ollama.** Ships origin checking for a local API with no auth, which is close to
  the position chartr may land on; find what its allowed-origin list contains and
  why.

For each: the default posture, what is enforced versus documented, how development
origins are admitted without weakening the shipped default, what happens on a
non-loopback bind, and — most useful of all — any public incident that moved them.

**What this ticket is not.** It is not a recommendation and must not contain one.
Tickets 02–04 do the deciding; this one gives them something to decide against.
Where a tool's answer looks wrong for chartr, record it anyway with the reason it
does not transfer, rather than filtering the survey down to the appealing options.

**Watch for the shape that matters most.** Nearly every tool here has to solve
"admit the dev origin without shipping a hole" — the exact tension the comment at
`control.go:23-25` resolved the wrong way. If a pattern recurs across three or more
of them, that is the finding, and it should be called out at the top of the answer
rather than buried per-tool.

Done when: each tool above has its current posture recorded with a citation, the
incidents that shaped them are named where they exist, the recurring dev-origin
pattern is called out explicitly, and nothing in the answer recommends a course for
chartr.
