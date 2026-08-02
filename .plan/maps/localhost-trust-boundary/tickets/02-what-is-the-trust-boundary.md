---
type: grilling
blocked_by: [01]
claimed_by: s03dce6d82955
claimed_at: 2026-08-02T18:18:35Z
---

# What is chartr's trust boundary?

## Question

Say, in writing, who is allowed to reach the cockpit and what they are allowed to
do. chartr has never had an answer to this. It has had an *assumption*, stated at
`internal/server/control.go:23-25` and repeated at `terminals.go:239-241`: a
single-operator tool bound to localhost, therefore safe. The security report showed
what that assumption misses — the browser is a client chartr does not control, and
"bound to localhost" says nothing about which *page* is connecting.

Settle the frame the rest of the map decides against:

- **Who is in the trust set?** The operator, plainly. Their browser? Every tab in
  it? Other processes running as them? Other users on the machine? Name each and say
  in or out. The websocket bug was possible because "the operator's browser" was
  treated as one thing when it is really "every page the operator has open."
- **What is the asset being protected?** Be specific, because it drives everything
  else. Shell and agent execution as the operator is the obvious one. But the
  control snapshot also discloses absolute repository paths, branch and dirty state,
  the agent library and config layer paths, and terminal scrollback carries whatever
  the operator's shells have printed — which for an agent cockpit routinely includes
  source, diffs, and anything an agent was handed. Decide whether disclosure is a
  protected asset in its own right or only a stepping stone to execution.
- **Is the boundary the process, the port, or the origin?** These give different
  answers and the code currently implies all three at once. A local process running
  as the operator can reach the port no matter what — so if the boundary is the
  port, chartr can never defend it. If the boundary is the *origin*, then the
  browser is the threat surface and origin/Host checks are the whole defence. Say
  which, because ticket 03's authentication question is nearly decided by this one.
- **What does chartr owe an operator who does something unusual?** Binding wide,
  running on a shared machine, running two instances. Enforcement, a warning, or
  nothing — and the principle behind the choice, not a case-by-case list.

**Grill the standing preference here rather than around it.** This project leans
toward trust at the gate — minimal enforcement, configuration believed rather than
policed. That instinct produced the comment under review. Do not overturn it by
reflex and do not inherit it by reflex either: state where the gate is, and what it
is entitled to assume about everything past it. If the conclusion is that the
existing preference was right and only the *implementation* was wrong, that is a
legitimate answer — but it has to be argued against the working exploit, and the
answer must say what evidence would have changed it.

**The dev-proxy tension is in scope and is the concrete test case.** Whatever
boundary this ticket draws has to accommodate a Vite dev server on a different
origin without the shipped binary being weakened. Ticket 01's survey should show how
others resolved it. A boundary that cannot answer this is not finished, because it
is precisely where the last one failed.

Done when: the trust set, the protected assets, and the location of the boundary are
each stated in a form the later tickets can build on; the dev-proxy case is
accommodated by the stated boundary rather than excepted from it; and the answer
says plainly whether the original assumption is being defended or replaced.

## Answer

**chartr's boundary is client admission at the cockpit server, not the process or
the listening port.** For a browser client, the security identity is the exact
origin (scheme, host, and port) of the page making the request. `Host` protects
that decision from DNS rebinding; it is not a substitute for it. Loopback and an
ephemeral or fixed port reduce who can reach the gate, but reaching the port does
not put a caller inside the trust set.

### Trust set

- **In:** the single human operator, and a chartr UI document the operator opened
  from an origin this particular server instance admits. The document is trusted
  as that origin, not because it happens to run in the operator's browser.
- **Out:** every other tab and page in that browser, including a page the operator
  intentionally visited. The browser application and browser profile are not
  trusted wholesale. Extensions and injected script are likewise not promoted
  merely because they execute in the same browser; compromise of the admitted
  chartr origin itself remains a compromise of the client.
- **Out by default:** other local processes, including processes running as the
  operator, dependencies in an opened repository, and agents chartr spawned.
  Running with the operator's OS authority may let such a process obtain many of
  the same effects directly, but it does not grant the separate authority to
  inspect or steer every live chartr space, terminal, and session. Ticket 03 must
  decide how much of this exclusion can usefully be enforced; this boundary does
  not turn an enforcement limitation into trust by definition.
- **Out:** other OS users on the machine and every network peer. Loopback does not
  distinguish local users, and a non-loopback bind changes reachability, not this
  trust set. Multi-operator access remains a different product.

The model does not claim to sandbox a fully compromised OS account. Same-user code
may read repositories, signal processes, inspect process state, or steal whatever
capability ticket 03 might introduce. The useful promise is narrower: chartr does
not intentionally hand cockpit-wide authority to a caller merely because it is
local. That distinction matters for a hostile web page, a lower-privilege local
user, and a compromised agent whose direct filesystem view may be narrower than
the cockpit's cross-space view.

### Protected assets

Both **authority and disclosure are protected in their own right**:

- shell keystrokes, terminal creation and termination, agent/session spawning and
  lifecycle actions, and configuration or registry mutations are operator
  authority; availability of those terminals and sessions is part of it;
- control snapshots, repository and configuration paths, branch and dirty state,
  agent-library contents, ticket payloads, and terminal scrollback are confidential
  cockpit data. Disclosure is harm even when it is not followed by execution: it
  can expose source, diffs, prompts, secrets printed by a command, and the shape of
  work elsewhere on the machine.

A push-only route is therefore not harmless, and a read-only route is not outside
the boundary. Every HTTP and WebSocket route crosses the same admission boundary;
what a route is allowed to reveal or change is a later, route-level question.

### The development origin is an admitted caller, not an exception

Production admits the cockpit's own effective origin. Development may **add the
one exact Vite origin for that run** while retaining the ordinary Origin and Host
gates; it must not disable either gate globally or make the shipped binary trust
all origins. The two origins are different principals deliberately joined by
development configuration. This is the recurring shape in blocker 01's Jupyter,
Vite, code-server, GoTTY, and Ollama findings, and it avoids repeating the false
choice embodied by `InsecureSkipVerify`: either the proxy works or the boundary
exists.

An origin allowlist constrains browsers only. A client that sends no meaningful
`Origin` has not thereby proved it is trusted; ticket 03 owns the credential or
other treatment for that case. Likewise, trusting a development `Origin` does not
make an arbitrary request `Host` valid.

### What configuration may assume

The standing preference for **trust at the gate survives, but the old gate does
not**. Once a caller has crossed one consistently enforced admission boundary,
handlers should not grow route-by-route pseudo-authentication. Configuration may
name an additional trusted caller explicitly. It may not silently enlarge the
trust set as a side effect of changing an address, running on a shared machine, or
starting another instance.

The principle for unusual operation is: **preserve the trust set, require an
explicit compensating gate, or refuse the configuration.** A warning can explain
the consequence but is not itself a boundary. Thus a wide bind must not turn
network peers into trusted operators; a shared-machine launch must not treat all
local users as the operator; and two instances are two origins whose authority is
not interchangeable. Tickets 03 and 04 still decide the mechanism and migration,
and ticket 05 decides whether the desktop shell can prove the same admission more
tightly without becoming a divergent policy.

### Rejected frames and verdict

- **Process boundary:** rejected because the browser process contains mutually
  distrustful pages, and same-user processes are not equivalent to operator
  intent. Treating the whole process or OS account as trusted recreates the
  reported exploit by definition.
- **Port boundary:** rejected because a port supplies reachability, not identity;
  fixed versus ephemeral and loopback versus wide only change the effort and set
  of possible callers.
- **Origin as the entire boundary:** rejected because it describes browser callers
  well and non-browser callers not at all. It is the browser proof used at the
  server-wide admission boundary, not a complete authentication policy.
- **Warn and trust configuration:** rejected as a boundary model. It can be an
  operator-experience choice only after another control preserves who is admitted.

**Verdict:** replace the original assumption. “Single operator, bound to localhost”
remains a product and reachability fact, but it is not authorization. chartr trusts
one operator through explicitly admitted clients; all routes protect both control
and disclosure at a uniform server-ingress boundary. The knowingly accepted cost
is explicit development-origin configuration and, depending on tickets 03–05, a
small launch/bootstrap cost. The knowingly accepted limitation is that chartr
cannot provide strong isolation from arbitrary code already controlling the
operator's OS account.

Reopen this decision if chartr becomes multi-operator or supports remote access,
if a supported client cannot present either a trustworthy browser origin or the
proof ticket 03 selects, or if browser/desktop embedding makes origin cease to
identify the admitted UI. A new way for same-user code to cross materially greater
OS privilege through chartr would also require strengthening the boundary rather
than merely revisiting route policy.
