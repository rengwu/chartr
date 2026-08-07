import { describe, expect, it } from 'vitest'
import type { Space, Terminal } from './model'
import { configurableSpaces, visibleSpaces } from './spacevisibility'

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
    dirty: false,
    layers: [],
    maps: [],
    terminals,
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

describe('configurable spaces', () => {
  it('drops Scratch, which has no config to scope', () => {
    const alpha = space('alpha', false, [])
    expect(configurableSpaces([alpha, space('scratch', true, [])])).toEqual([alpha])
  })

  it('drops Scratch even while it has an open shell', () => {
    // The sidebar's visibility rule would keep this one; the settings surface
    // reads the unfiltered list minus Scratch, which is a different question.
    const alpha = space('alpha', false, [])
    const occupiedScratch = space('scratch', true, [terminal()])
    expect(configurableSpaces([alpha, occupiedScratch])).toEqual([alpha])
  })

  it('keeps every registered space in its given order', () => {
    const alpha = space('alpha', false, [])
    const beta = space('beta', false, [terminal()])
    expect(configurableSpaces([beta, alpha])).toEqual([beta, alpha])
  })
})
