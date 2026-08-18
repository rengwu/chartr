import { describe, expect, it } from 'vitest'
import type { Terminal, TerminalStatus } from './model'
import { promptTarget } from './prompttarget'

function term(over: Partial<Terminal> = {}): Terminal {
  return {
    id: 'term-1',
    title: 'claude',
    proc: 'claude',
    status: 'idle',
    alive: true,
    promptTarget: true,
    ...over,
  }
}

describe('prompt target', () => {
  it('offers Send to an idle agent', () => {
    expect(promptTarget(term())).toEqual({ kind: 'send' })
  })

  it('offers Queue while the agent is busy or waiting on its human', () => {
    for (const status of ['working', 'running', 'blocked'] as TerminalStatus[]) {
      expect(promptTarget(term({ status }))).toEqual({ kind: 'queue' })
    }
  })

  it('explains that there is no tab to send to', () => {
    expect(promptTarget(null)).toEqual({
      kind: 'ineligible',
      reason: 'No active tab — open an agent in this space to send it a preset.',
    })
  })

  it('explains an exited tab rather than offering an action', () => {
    expect(promptTarget(term({ alive: false, status: 'exited', promptTarget: undefined }))).toEqual({
      kind: 'ineligible',
      reason: 'This tab’s process has exited.',
    })
  })

  it('explains a dead session the same way', () => {
    expect(promptTarget(term({ alive: false, status: 'dead', promptTarget: undefined }))).toEqual({
      kind: 'ineligible',
      reason: 'This tab’s process has exited.',
    })
  })

  it('refuses a shell sitting at its own prompt', () => {
    expect(promptTarget(term({ title: 'zsh', proc: 'zsh', promptTarget: undefined }))).toEqual({
      kind: 'ineligible',
      reason: 'This tab has no agent in front of it — start one to send it a preset.',
    })
  })

  it('offers an agent the operator started themselves the same actions', () => {
    // The server flags the tab because an agent holds its foreground, not because
    // chartr launched it; from here the two are one case and read the same status.
    expect(promptTarget(term({ title: 'zsh', proc: 'claude' }))).toEqual({ kind: 'send' })
    expect(promptTarget(term({ title: 'zsh', proc: 'claude', status: 'working' }))).toEqual({
      kind: 'queue',
    })
  })
})
