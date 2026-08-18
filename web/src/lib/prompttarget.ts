import type { Terminal } from './model'

/**
 * What the Prompts pane may do with the space's active tab (prompt-presets
 * ticket 04). One preset is either sent now, queued for the next observed idle,
 * or refused with a plain sentence — there is no fourth state, and the pane
 * never guesses: `promptTarget` is the server's own answer to "is this a live
 * agent chartr launched", and the status is the same activity the sidebar shows.
 */
export type PromptTarget =
  | { kind: 'send' }
  | { kind: 'queue' }
  | { kind: 'ineligible'; reason: string }

export function promptTarget(term: Terminal | null | undefined): PromptTarget {
  if (!term) {
    return {
      kind: 'ineligible',
      reason: 'No active tab — open an agent in this space to send it a preset.',
    }
  }
  if (!term.promptTarget) {
    // An exited tab and a dead session are the same thing to a delivery: there
    // is no process left to type into. Everything else that lacks the flag is an
    // ordinary shell or an agent the operator started themselves — a tab running
    // `claude` by hand reads the agent grammar and can even show idle, and is
    // still not a target, because eligibility is who launched the binary.
    if (!term.alive) return { kind: 'ineligible', reason: 'This tab’s process has exited.' }
    return {
      kind: 'ineligible',
      reason: 'Only a live agent chartr launched can be sent a preset.',
    }
  }
  return term.status === 'idle' ? { kind: 'send' } : { kind: 'queue' }
}
