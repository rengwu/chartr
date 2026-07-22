---
type: task
blocked_by: [01]
---

# The action bar picks and names the agent

## Question

Give the operator the choice, at the moment of spawning, on the surface where
spawning is deliberate — and make the button say what it will run. After this
ticket the frontend always sends an explicit `agent`, so the binding path is
unused in practice even though it still exists.

**The selection rule lands as a pure module first.** Write
`web/src/lib/agentchoice.ts` — "which agent does this space spawn with" — as a
function over the library and the space's `lastAgent`, returning one of three
states: **ready** (an agent to spawn with), **unchosen** (nothing remembered, or
a remembered name that no longer resolves, or one whose binary is absent), and
**empty** (no agents registered at all). This is where the rules from the spec
live, and it is the only part of this ticket with tests: prior art is
`args.ts` / `args.test.ts` and `attention.ts`. There is no Svelte component
testing in this repo and this ticket does not introduce any — logic worth
asserting goes in the module, not the component.

**The action bar becomes a split control.** In `web/src/lib/DetailPane.svelte`,
each role button gains the agent it will use and a secondary affordance that
opens the list. In the **ready** state the primary click spawns straight away and
the button names the agent. In the **unchosen** state the primary click opens the
list instead of spawning — there is no automatic first choice, and the rule does
not vary with how many agents are registered. The **empty** state is ticket 04's
work; leave it to fall through to the existing error path here rather than
half-building it.

**A successful spawn writes the choice**, which ticket 01 already does
server-side — so an override needs no separate confirming action and simply
becomes what this space uses next time. The picker lists every registered agent;
one whose binary is absent renders **disabled with its reason visible on the
row**, not hidden behind a hover title. The spawn-time refusal from ticket 01
stays as the backstop for a binary that vanishes after the list renders.

**Use a primitive, not a hand-rolled menu.** No `dropdown-menu` is vendored yet.
Add it through the documented path — `cd web && npx shadcn-svelte@latest add
dropdown-menu`, then swap lucide icons for Phosphor, prune unused deps, and
re-check for raw colour (`docs/design-system.md` → *Adding a primitive*; ADR
0012). Tokens for every colour, no chroma in the chrome.

Note the bar gets more crowded: up to four role buttons now each carry an agent
name. That is known, accepted, and explicitly out of scope for this map.

Done when: `go vet ./...` / `go test ./...` and the frontend `check` / `build` /
`vitest` are green with no amber in the built CSS; `agentchoice.test.ts` covers
ready, unchosen (nothing remembered / stale name / absent binary) and empty; and
in the running cockpit the first spawn in a space opens the picker, a later spawn
goes one click and names its agent on the button, choosing a different agent from
the list spawns that one and makes it the space's new default, and an agent whose
binary is missing is unselectable with the reason readable on its row.

