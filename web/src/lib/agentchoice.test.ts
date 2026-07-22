import { describe, expect, it } from 'vitest'
import { chooseAgent } from './agentchoice'
import type { Agent } from './model'

function agent(overrides?: Partial<Agent>): Agent {
  return {
    name: 'default',
    adapter: 'claude',
    args: [],
    prompt: '--prompt',
    delivery: 'flag',
    command: ['claude', '--prompt', '<opener>'],
    present: true,
    ...overrides,
  }
}

describe('chooseAgent', () => {
  it('is empty when no agents are registered', () => {
    expect(chooseAgent([], undefined)).toEqual({ kind: 'empty' })
    expect(chooseAgent([], 'claude')).toEqual({ kind: 'empty' })
  })

  it('is unchosen when nothing is remembered', () => {
    const agents = [agent({ name: 'claude' })]
    expect(chooseAgent(agents, undefined)).toEqual({ kind: 'unchosen' })
  })

  it('is ready when the remembered agent resolves and is present', () => {
    const agents = [agent({ name: 'claude' }), agent({ name: 'grill', adapter: 'grill' })]
    expect(chooseAgent(agents, 'grill')).toEqual({ kind: 'ready', agent: agents[1] })
  })

  it('is unchosen when the remembered name no longer resolves', () => {
    const agents = [agent({ name: 'claude' })]
    expect(chooseAgent(agents, 'deleted')).toEqual({ kind: 'unchosen' })
  })

  it('is unchosen when the remembered agent is absent from PATH', () => {
    const agents = [agent({ name: 'claude', present: false, missing: 'claude not found on PATH' })]
    expect(chooseAgent(agents, 'claude')).toEqual({ kind: 'unchosen' })
  })

  it('does not automatically choose the only registered agent', () => {
    const agents = [agent({ name: 'claude' })]
    expect(chooseAgent(agents, undefined)).toEqual({ kind: 'unchosen' })
  })
})
