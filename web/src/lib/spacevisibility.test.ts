import { describe, expect, it } from 'vitest'
import type { Space, Terminal } from './model'
import { visibleSpaces } from './spacevisibility'

function terminal(): Terminal {
  return {
    id: 'term-1',
    title: 'shell',
    proc: 'zsh',
    status: 'idle',
    alive: true,
  }
}

function space(id: string, scratch: boolean, terminals: Terminal[]): Space {
  return {
    id,
    name: scratch ? 'Scratch' : id,
    path: scratch ? '/home/operator' : `/repos/${id}`,
    scratch: scratch || undefined,
    maps: [],
    terminals,
    prompts: [],
  }
}

describe('visible spaces', () => {
  it('filters an empty Scratch space out', () => {
    expect(visibleSpaces([space('scratch', true, [])])).toEqual([])
  })

  it('keeps Scratch while it has an open shell', () => {
    const scratch = space('scratch', true, [terminal()])
    expect(visibleSpaces([scratch])).toEqual([scratch])
  })

  it('never filters a registered space based on its shells', () => {
    const empty = space('alpha', false, [])
    const occupied = space('beta', false, [terminal()])
    expect(visibleSpaces([empty, occupied])).toEqual([empty, occupied])
  })
})
