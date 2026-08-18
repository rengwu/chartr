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
  it('offers Send to an idle agent chartr launched', () => {
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

  it('refuses an ordinary shell', () => {
    expect(promptTarget(term({ title: 'zsh', proc: 'zsh', promptTarget: undefined }))).toEqual({
      kind: 'ineligible',
      reason: 'Only a live agent chartr launched can be sent a preset.',
    })
  })

  it('refuses an agent the operator started themselves, whatever it reads as', () => {
    // The shell is running `claude`, so the tab reads the agent grammar and can
    // even show idle — eligibility is who launched the binary, not what is in it.
    expect(promptTarget(term({ proc: 'claude', promptTarget: undefined }))).toEqual({
      kind: 'ineligible',
      reason: 'Only a live agent chartr launched can be sent a preset.',
    })
  })
})
