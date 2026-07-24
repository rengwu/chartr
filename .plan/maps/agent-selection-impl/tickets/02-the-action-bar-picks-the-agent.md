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

## Answer

The detail pane's spawn buttons now name the agent they will run and let the
operator pick or override it, while the rule that decides *which* agent lives in a
testable pure module rather than inside a component. Implemented in three parts:

- **`web/src/lib/agentchoice.ts` — the selection rule as a pure function.**
  `chooseAgent(agents, lastAgent)` returns one of `ready`, `unchosen`, or `empty`.
  `ready` requires a remembered name that resolves to a registered agent whose
  binary is on PATH; a missing name, a stale name, and an agent whose binary is
  absent all read as `unchosen`. `empty` is reserved for a completely empty
  library. There is no automatic first choice — a library of one agent with
  nothing remembered still opens the picker, so adding a second agent never
  silently changes behaviour. `agentchoice.test.ts` covers ready, unchosen under
  all three conditions, and empty.
- **`web/src/lib/actions.ts` and prop drilling — the frontend always names an
  agent.** `spawnSession` now sends `agent` in the request body (defaulting to the
  empty string in the still-deferred empty-library case). The global agent library
  and the space's `lastAgent` are threaded `App → SpacePane → MapCard →
  DetailPane`, so the pane has both inputs it needs without reaching outside its
  props.
- **`web/src/lib/DetailPane.svelte` and the new dropdown primitive — split
  controls on the action bar.** Each role button is now an inline-flex pair: the
  primary button spawns with the remembered agent and names it on the label when
  `agentChoice` is `ready`; when `unchosen` it opens the agent picker instead. The
  caret button is a `DropdownMenu.Trigger` styled from the same `buttonVariants`
  so the two halves match. The picker lists every registered agent; an agent whose
  binary is missing is disabled and shows its `missing` reason on the row. A
  one-off choice from the picker spawns with that agent, and ticket 01's
  server-side `SetLastAgent` makes it the space's new remembered choice with no
  second confirming action. The empty-library case is intentionally left to fall
  through to the existing server refusal path — ticket 04 owns that surface.

A new `dropdown-menu` primitive was vendored through `npx shadcn-svelte@latest add
dropdown-menu`. Its lucide icons were swapped for Phosphor (`CaretRight`, `Check`,
`Minus`), the unused `@lucide/svelte` dependency was removed from
`web/package.json`, and `npm install` cleaned the lockfile. No raw colours were
introduced: the menu uses `--popover`, `--accent`, `--muted-foreground`, and
`--destructive` only.

Against Done-when: `go vet ./...` and `go test ./...` pass; frontend `check`
reports 0 errors, `build` succeeds, and `vitest` passes 91 tests including the 6
new `agentchoice` tests. The built CSS contains no amber. The server was rebuilt
and starts cleanly, returning the cockpit on the loopback address.

ADR 0012 is followed throughout: tokens, vendored primitives, and Phosphor icons
only. ADR 0002 holds — the frontend still knows nothing about what any CLI's
flags mean; it only names and sends the chosen agent. No ADR is changed.

Scope note for review: the action bar is intentionally more crowded with up to
four agent-named split buttons; collapsing the roles is deferred work per the
map's Out of scope. The empty-library state is also ticket 04's responsibility.

