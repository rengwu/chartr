import type { Terminal } from './model'

/**
 * What the Prompts pane may do with the space's active tab (prompt-presets
 * ticket 04). One preset is either sent now, queued for the next observed idle,
 * or refused with a plain sentence — there is no fourth state, and the pane
 * never guesses: `promptTarget` is the server's own answer to "is a live agent
 * in front of this tab", and the status is the same activity the sidebar shows.
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
    // is no process left to type into. Anything else that lacks the flag is a
    // shell sitting at its own prompt, where the preset would be run as a command
    // rather than read — an agent the operator started themselves is a target
    // exactly as chartr's own launch is, for as long as it holds the foreground.
    if (!term.alive) return { kind: 'ineligible', reason: 'This tab’s process has exited.' }
    return {
      kind: 'ineligible',
      reason: 'This tab has no agent in front of it — start one to send it a preset.',
    }
  }
  return term.status === 'idle' ? { kind: 'send' } : { kind: 'queue' }
}
