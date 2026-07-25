// What the sidebar skill launcher offers, as a pure function over the resolved
// skill library and the space's remembered agent — the launcher analogue of
// `agentchoice`, keeping the menu component thin and untested (skill-launcher
// ticket 02). The launcher is always a dropdown: an agent section (the same agents
// `chooseAgent` ranks, each labelled by the model it already carries) over the
// on-ramp skills the library marks launchable. A skill click *is* the launch, on
// the currently-selected agent — there is no primary-action button and no
// remembered skill. The one exception is a skill that reads a subject
// (`needs-context: true`), which earns an optional one-line box between the click
// and the launch (ticket 03).

import type { Agent, ResolvedSkill } from './model'
import { chooseAgent, type AgentChoice } from './agentchoice'

export interface LaunchMenu {
  // The agent rows the picker offers — every registered agent, present or not
  // (an absent one renders disabled with its reason, exactly as the agent picker
  // does elsewhere). Empty when nothing is registered, which is the `empty`
  // choice below routing to registration.
  agents: Agent[]
  // The on-ramp skills the resolved library marks launchable (`on-ramp: true`).
  // An operator's own on-ramp skill rides here with no chartr-side change — the
  // payoff the hackable library was built for.
  skills: ResolvedSkill[]
  // The agent a skill click launches on: the operator's in-menu pick this open if
  // they made one, else the space's remembered agent. `ready` carries that agent;
  // `unchosen` means the operator must pick one from the section above before a
  // skill can run; `empty` routes to registration.
  choice: AgentChoice
}

/**
 * Compute the launcher menu for a space.
 *
 * `selected` is the agent the operator picked in the open menu this session, if
 * any — it overrides the remembered `lastAgent` without persisting, exactly the
 * one-off override the split button offered. When absent, the menu opens on the
 * remembered agent so it is one line to confirm and rarely changed.
 */
export function launchMenu(
  agents: Agent[],
  lastAgent: string | undefined,
  skills: ResolvedSkill[],
  selected?: string,
): LaunchMenu {
  return {
    agents,
    skills: skills.filter((s) => s.onRamp),
    choice: chooseAgent(agents, selected ?? lastAgent),
  }
}

// The `/launch` call a skill click assembles. `context` is the operator's optional
// one line, and it is *present only when they typed one* — an empty box is a valid
// launch that opens the skill bare (ticket 03).
export interface LaunchPayload {
  agent: string
  skill: string
  context?: string
}

// What a skill click does next. A skill that reads a subject
// (`needs-context: true`) earns the optional one-line box before it runs; a
// self-driving one launches on the click, exactly as 02 already did.
export type LaunchStep =
  // Offer the box. Carries the agent the launch will run on (so the box can name
  // it) and the skill itself (for its hint), not a payload — nothing launches
  // until the operator confirms.
  | { kind: 'context'; agent: string; skill: ResolvedSkill }
  // Launch now, bare.
  | { kind: 'launch'; agent: string; skill: string }

/**
 * What a click on `skill` does: launch it on the ready agent, offer the context
 * box first when the skill asks for one, or nothing (`null`) when no agent is
 * ready yet (no library, or nothing chosen). A null result is not a dead click —
 * the agent section above is the actionable path, and the skill row renders
 * disabled until an agent is chosen.
 */
export function launchClick(menu: LaunchMenu, skill: ResolvedSkill): LaunchStep | null {
  if (menu.choice.kind !== 'ready') return null
  const agent = menu.choice.agent.name
  if (skill.needsContext) return { kind: 'context', agent, skill }
  return { kind: 'launch', agent, skill: skill.name }
}

/**
 * The `/launch` call the context box fires for `line`. Optional means optional:
 * a blank (or whitespace-only) line carries no `context` at all, so the skill
 * opens bare — the payload is its body unchanged, per 01. There is no validation
 * gate; both spellings are a real launch.
 */
export function launchPayload(agent: string, skill: string, line: string): LaunchPayload {
  const context = line.trim()
  return context ? { agent, skill, context } : { agent, skill }
}

/**
 * The box's placeholder, taken from the skill that will read the line — `grill`
 * wants to know *what* to grill, `research` *what* to research. The skill's
 * `description` says what the skill does rather than what the line should say, so
 * it stays on the menu row above and the field asks the question instead.
 */
export function contextHint(skill: ResolvedSkill): string {
  return `What should ${skill.name} work on?`
}

/**
 * The model an agent carries in its flags — decision (b): there is no new model
 * axis, so the label just surfaces what the agent's own args already say. Scans
 * the resolved launch command (which folds in the adapter's own defaults) for a
 * `--model` / `-m` value; `undefined` when the agent names none and the adapter's
 * built-in default stands, in which case the row shows the name alone.
 */
export function agentModel(agent: Agent): string | undefined {
  const argv = agent.command ?? []
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]
    if (a === '--model' || a === '-m') return argv[i + 1] || undefined
    const eq = /^--model=(.+)$/.exec(a)
    if (eq) return eq[1]
  }
  return undefined
}
