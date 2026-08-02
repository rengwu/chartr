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
