---
type: task
blocked_by: [01]
---

# The sidebar skill picker

## Question

Turn the space card's `+ Idea ▾` control into the skill launcher the Destination
describes, driving the `/launch` endpoint (01). **The control is always a
dropdown** — a single `skills ▾` trigger the operator opens and picks a skill from
every time; there is no split primary-action button and no remembered skill. Build
it on the vendored `DropdownMenu` primitives (not `AgentSplitButton`'s split
shape), reusing its agent-choice logic (`chooseAgent`) for the agent section — a
new bespoke `.btn`/`.card` would violate the design system (ADR 0012).

- **Two sections in the dropdown.** Top: the agent selector, each row **labelled by
  that agent's resolved model** (decision (b) — no new model axis; the label just
  surfaces what the agent already carries), opening on the space's remembered agent
  (`space.lastAgent`) so it is one line to confirm and rarely changed. Below a
  divider: the on-ramp skills, `snapshot.skills.filter(s => s.onRamp)`. Clicking a
  skill launches it on the currently-selected agent — that click *is* the launch;
  there is no separate run button. Phosphor icons and tokens only — no raw colour,
  no bespoke chrome CSS.

- **The empty-library and unchosen-agent states** stay as `AgentSplitButton`
  handles them today — route to registration, never a dead button — reused through
  the same `chooseAgent`/`onregister` path rather than re-implemented.

- **The callback** is `onrun(agent, skill)`. Update both
  mounts: the sidebar space card in `App.svelte` (the `ideateSpace` handler becomes
  a `launchSpace(space, agent, skill)` calling a new `launch` action in
  `actions.ts` beside `ideate`) and the empty-stage on-ramp in `SpacePane.svelte`.
  Keep the compact `nameOnLabel={false}` sizing the sidebar row already uses; the
  agent's model and the skill name live in the menu, not on the cramped label.

Leave the *context box* to 03 — this ticket launches every on-ramp skill bare
(passing no `context`), which is the correct behaviour for the self-driving ones
regardless. A `needs-context` skill launching bare here is valid (context is
optional) and simply lands 03's affordance next.

Tests lead where there is pure logic: the selection helper — given the resolved
skills and the remembered agent, which agents and which skills the menu offers, and
what a skill click launches `(agent, skill)` — is a `vitest` unit like the existing
component/logic tests. The menu rendering is trusted once the helper is right, as
the current `AgentSplitButton` is.

Done when: the space card's control is a single `skills ▾` dropdown listing the
on-ramp skills under the agent selector (agents labelled by model, opening on the
remembered agent), clicking any on-ramp skill launches it on the selected agent via
`/launch`, and both the sidebar and empty-stage mounts drive the `(agent, skill)`
callback; `check` / `build` / `vitest` and `go vet` / `go test` pass; no amber in
the built CSS.

## Answer

Shipped the launcher as a single dropdown, driving 01's `/launch`.

**Selection logic (`launchmenu.ts`, unit-tested).** `launchMenu(agents, lastAgent,
skills, selected?)` is the launcher analogue of `chooseAgent`: it offers every
registered agent, the on-ramp skills (`skills.filter(s => s.onRamp)`), and the
effective agent a skill click runs on — the operator's in-menu pick this open, else
the remembered agent, resolved through `chooseAgent` (so `empty` routes to
registration and `unchosen` means "pick one first", exactly as elsewhere).
`launchClick(menu, skill)` returns the `(agent, skill)` a click fires or `null` when
no agent is ready. `agentModel(agent)` surfaces decision (b)'s label by reading the
`--model` / `-m` value out of the agent's already-resolved `command` — no new model
field, just what the agent carries.

**The control (`SkillLauncher.svelte`).** Built on the vendored `DropdownMenu`
primitives, not a split button. One `Skills ▾` trigger opens two sections: a
`RadioGroup` agent selector (each present row labelled by its model,
`closeOnSelect={false}` so picking an agent keeps the menu open, opening checked on
the remembered agent) over a divider and the on-ramp skills. A skill click *is* the
launch; skills sit disabled until an agent is chosen, and the empty-library state
routes to `onregister` — never a dead button. Callback is `onrun(agent, skill)`.

**Both mounts + the action.** `actions.ts` gained `launch(id, agent, skill,
context='')` beside `ideate`. `App.svelte`'s `ideateSpace` became `launchSpace(space,
agent, skill)`; the sidebar card and `SpacePane`'s empty-stage on-ramp both mount
`SkillLauncher` and thread the `(agent, skill)` callback (`SpacePane`'s `onIdeate`
prop is now `onLaunch`). Every launch is bare (no context — that is 03's box).

**Notes.** Dropped the old `nameOnLabel` knob — it toggled the *agent name* on the
split button's label, but the launcher always keeps agent/skill detail in the menu,
so it was meaningless; the sidebar stays compact via `size="xs"`. `AgentSplitButton`
is deleted — both its mounts moved to `SkillLauncher` and nothing else used it, so
leaving the duplicate picker would be dead code against the design system. Frontend
`check` (0/0), `vitest` (141 pass, incl. the new `launchmenu.test.ts`), and `build`
are green with no amber in the built CSS; `go vet ./...` and `go test ./...` pass
(the embed test compiled against the fresh `dist/`).
