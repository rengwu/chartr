// What the sidebar skill launcher offers, as a pure function over the resolved
// skill library and the space's remembered agent — the launcher analogue of
// `agentchoice`, keeping the menu component thin and untested (skill-launcher
// ticket 02). The launcher is always a dropdown: an agent section (the same agents
// `chooseAgent` ranks, each labelled by the model it already carries) over the
// on-ramp skills the library marks launchable. A skill click *is* the launch, on
// the currently-selected agent — there is no primary-action button and no
// remembered skill.

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

/**
 * What a click on `skill` launches: the `(agent, skill)` pair the callback fires,
 * or `null` when no agent is ready yet (no library, or nothing chosen). A null
 * result is not a dead click — the agent section above is the actionable path, and
 * the skill row renders disabled until an agent is chosen.
 */
export function launchClick(
  menu: LaunchMenu,
  skill: ResolvedSkill,
): { agent: string; skill: string } | null {
  if (menu.choice.kind !== 'ready') return null
  return { agent: menu.choice.agent.name, skill: skill.name }
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
