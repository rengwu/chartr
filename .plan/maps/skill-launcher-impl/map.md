# Skill launcher — implementation

## Destination

The sidebar space card's `+ Idea ▾` on-ramp becomes a **skill launcher**: one
control that runs any *self-driving* skill — the ones that can drive a session
from the start or from a line of context — on a chosen agent. The dropdown carries
two sections: the agent picker it already has (labelled by that agent's resolved
model), and, below it, the on-ramp skills the resolved library offers. Skills
opt in from their own `SKILL.md` frontmatter, so an operator's own skill dropped
into the user or workspace layer appears in the picker with no chartr-side change
— the payoff the hackable skill library was built for. A skill that wants a line
of context first offers an **optional** one-line box before launch; a self-driving
skill launches bare, exactly as ideate does today. Done when the picker launches
every on-ramp skill (ideate unchanged as the default), a user-authored on-ramp
skill shows up untouched, and the optional context reaches the agent.

## Notes

**This map carries execution.** Every ticket is a `task` that delivers working
code, not a decision. The deciding happened in conversation and lives here in the
Destination and the locked decisions below — there is no separate planning `map.md`
or `spec.md` for this effort. Do not re-litigate a decision; if implementation
exposes one as wrong, raise it here rather than quietly deviating.

**Decisions locked in conversation:**

- **The top selector stays the agent picker** — the registered agents from
  Settings — *labelled by each agent's resolved model*. There is no new,
  orthogonal "model" axis: an agent already carries its model, so the label just
  surfaces it. (No per-launch model override.)
- **Only self-driving skills are launchable.** A skill opts in with
  `on-ramp: true`. The launchable set can drive itself from the start or after a
  line of context — **ideate, wayfinder, grill, research, prototype** ship tagged.
  The augmentative or second-step skills stay off the picker: **core**,
  **tracker-convention**, **domain-modeling**, **to-spec**, **to-tickets**,
  **implement**.
- **Context is optional.** A skill marks `needs-context: true` to offer a one-line
  box before launch, but an empty box is valid and launches bare. A skill without
  the flag never shows a box.
- **An on-ramp launch is ticketless and mapless**, exactly like ideate: it shares
  only the adapter's spawn primitive — no map/ticket lookup, no claim, no Session,
  no death halt. It is `prompt.Ideate` generalised to any on-ramp skill, not a new
  spawn path.

**Per-session reading order:** this map, then the ticket you claim. Use
`CONTEXT.md` at the repo root for vocabulary — "island", "chrome", "control
socket", "user config", "skill library", "on-ramp". The relevant seams are
`internal/prompt` (skill resolution + composition), `internal/server`
(`terminals.go` `handleIdeate`, `spawn.go` `agentSpec`), and, on the frontend,
`web/src/lib/AgentSplitButton.svelte`, `web/src/App.svelte`,
`web/src/lib/SpacePane.svelte`, and `web/src/lib/actions.ts`. Respect ADR 0002
(chartr wires the session; the skill *format* is open, the injection path is not),
ADR 0010 (Svelte chrome / imperative islands), and ADR 0012 (shadcn-svelte design
system — tokens + primitives, no raw colour in the chrome, no amber).

**The test seams:** the pure Go `internal/prompt` parse (frontmatter flags survive
resolution and whole-skill shadowing) folded with the server launch handler
(refuses a non-on-ramp skill, threads the optional context, remembers the last
skill); and the frontend picker's pure selection logic (which agents, which skills,
what the primary click runs) beside the existing component tests. The imperative
terminal island is trusted once the launch hands it the right opener, as today.

**Before committing frontend changes** (per CLAUDE.md): run the frontend `check`
and `build` scripts plus `vitest`, and `go vet ./...` / `go test ./...` (the embed
test compiles against `dist/`). No amber in the built CSS; a colour is a token, a
component is a primitive. Drive the real binary where "Done when" is only real at
runtime, then resolve by shipping: append `## Answer` with what shipped plus a gist
under Decisions so far.

## Decisions so far

- **01 — the launch spine.** Skills declare `on-ramp` / `needs-context` in their own
  frontmatter (rides whole-skill shadowing); the flags reach the browser on the
  resolved-library snapshot. `prompt.Launch(roots, skill, context)` composes an
  on-ramp skill's body alone, appending an optional `## Your task` trailer only when
  context is present (`Ideate` is now `Launch(…, ideate, "")`). `POST /launch`
  generalises `handleIdeate`: same agent doorstep, refuses any skill the resolved
  library does not mark on-ramp (400 — the pushed library is the allowlist),
  threads context into the payload, remembers the agent (no remembered skill).
  `/ideate` stays as the `skill=ideate` delegate. [01](tickets/01-on-ramp-metadata-and-launch-spine.md)

- **02 — the sidebar picker.** The space card's on-ramp is one `Skills ▾` dropdown
  (`SkillLauncher.svelte`, on the vendored `DropdownMenu` primitives — the split
  `AgentSplitButton` is deleted). A `RadioGroup` agent selector, each row labelled by
  the model the agent already carries (parsed from its resolved `command`; no new
  model field), sits over a divider and the on-ramp skills; picking an agent keeps
  the menu open, a skill click *is* the launch on the selected agent via the new
  `launch` action, and skills disable until an agent is chosen (empty library →
  registration, never a dead button). Pure logic in `launchmenu.ts`
  (`launchMenu` / `launchClick` / `agentModel`, unit-tested); both the sidebar and
  `SpacePane`'s empty-stage mount drive `onrun(agent, skill)`. Launches are bare —
  the optional context box is 03.
  [02](tickets/02-the-sidebar-skill-picker.md)

## Not yet specified

<!-- Empty. Every decision is settled above; this map only executes it. A ticket
that exposes a genuinely new question raises it here rather than deviating. -->

## Out of scope

- **A per-launch model picker as a separate axis** — the top selector is the agent
  picker, labelled by its resolved model. Overriding a model per launch would be a
  new concept threaded through spawn and the registry.
- **Making auxiliary or second-step skills launchable** — `core`,
  `tracker-convention`, `domain-modeling`, `to-spec`, `to-tickets`, and `implement`
  stay off the picker. They augment a session or continue a flow; they do not open
  one cold.
- **Map/ticket-context spawns from the sidebar** — those are the star-map's spawn
  buttons, which resolve a claim, a role, and a context bundle. An on-ramp launch
  is deliberately ticketless and mapless.
- **A skill-authoring UI or marketplace** — skills are authored as `SKILL.md`
  directories on disk across the three layers; the config surface already renders
  the resolved library. This effort only *launches* them.
- **Persisting context history or templates** per skill — the optional box is a
  one-shot line, not a remembered form.
