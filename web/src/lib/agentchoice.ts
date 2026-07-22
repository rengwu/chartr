// Which agent a space spawns with — a pure function over the registered agent
// library and the space's remembered last choice. The rules from the agent
// selection spec live here so the UI component stays thin and untested.

import type { Agent } from './model'

export type AgentChoice =
  | { kind: 'ready'; agent: Agent }
  | { kind: 'unchosen' }
  | { kind: 'empty' }

/**
 * Decide which agent a space will spawn with.
 *
 * - `empty`: no agents are registered at all.
 * - `unchosen`: nothing is remembered, the remembered name no longer resolves,
 *   or the resolved agent's binary is absent from PATH.
 * - `ready`: a registered agent whose binary is present; this is what spawns.
 *
 * There is no automatic first choice: a library of one agent with nothing
 * remembered still opens the picker, because the rule must not silently change
 * when a second agent is registered.
 */
export function chooseAgent(agents: Agent[], lastAgent?: string): AgentChoice {
  if (agents.length === 0) {
    return { kind: 'empty' }
  }
  if (!lastAgent) {
    return { kind: 'unchosen' }
  }
  const agent = agents.find((a) => a.name === lastAgent)
  if (!agent) {
    return { kind: 'unchosen' }
  }
  if (!agent.present) {
    return { kind: 'unchosen' }
  }
  return { kind: 'ready', agent }
}
