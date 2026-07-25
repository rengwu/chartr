import { describe, expect, it } from 'vitest'
import { agentModel, contextHint, launchClick, launchMenu, launchPayload } from './launchmenu'
import type { Agent, ResolvedSkill } from './model'

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

function skill(name: string, overrides?: Partial<ResolvedSkill>): ResolvedSkill {
  return { name, layer: 'built-in', ...overrides }
}

const library: ResolvedSkill[] = [
  skill('ideate', { onRamp: true }),
  skill('wayfinder', { onRamp: true }),
  skill('grill', { onRamp: true, needsContext: true }),
  skill('core'), // augmentative — not on-ramp
  skill('to-tickets'), // second-step — not on-ramp
]

describe('launchMenu', () => {
  it('offers only the on-ramp skills, in library order', () => {
    const menu = launchMenu([agent({ name: 'claude' })], 'claude', library)
    expect(menu.skills.map((s) => s.name)).toEqual(['ideate', 'wayfinder', 'grill'])
  })

  it('offers every registered agent, present or not', () => {
    const agents = [agent({ name: 'claude' }), agent({ name: 'kimi', present: false })]
    expect(launchMenu(agents, 'claude', library).agents).toBe(agents)
  })

  it('opens on the remembered agent when it is ready', () => {
    const agents = [agent({ name: 'claude' }), agent({ name: 'kimi' })]
    const menu = launchMenu(agents, 'kimi', library)
    expect(menu.choice).toEqual({ kind: 'ready', agent: agents[1] })
  })

  it('opens on the first present agent when nothing is remembered', () => {
    const agents = [agent({ name: 'claude' }), agent({ name: 'kimi' })]
    const menu = launchMenu(agents, undefined, library)
    expect(menu.choice).toEqual({ kind: 'ready', agent: agents[0] })
  })

  it('skips an absent agent when defaulting — the default must be launchable', () => {
    const agents = [agent({ name: 'kimi', present: false }), agent({ name: 'claude' })]
    const menu = launchMenu(agents, undefined, library)
    expect(menu.choice).toEqual({ kind: 'ready', agent: agents[1] })
  })

  it('is unchosen when nothing is remembered and no agent is present', () => {
    const menu = launchMenu([agent({ name: 'kimi', present: false })], undefined, library)
    expect(menu.choice).toEqual({ kind: 'unchosen' })
  })

  it('is empty with no agents — routes to registration', () => {
    const menu = launchMenu([], 'claude', library)
    expect(menu.choice).toEqual({ kind: 'empty' })
    expect(menu.skills.map((s) => s.name)).toEqual(['ideate', 'wayfinder', 'grill'])
  })

  it('lets an in-menu pick override the remembered agent without persisting', () => {
    const agents = [agent({ name: 'claude' }), agent({ name: 'kimi' })]
    const menu = launchMenu(agents, 'claude', library, 'kimi')
    expect(menu.choice).toEqual({ kind: 'ready', agent: agents[1] })
  })

  it('is unchosen when the in-menu pick does not resolve — never a silent swap', () => {
    // The picker only ever sets `selected` to a present agent it listed, so this is
    // defensive: an override that does not resolve does not fall back to the
    // remembered agent, it simply leaves nothing chosen.
    const agents = [agent({ name: 'claude' })]
    const menu = launchMenu(agents, 'claude', library, 'kimi')
    expect(menu.choice).toEqual({ kind: 'unchosen' })
  })
})

describe('launchClick', () => {
  it('launches a self-driving skill straight away — no box', () => {
    const menu = launchMenu([agent({ name: 'claude' })], 'claude', library)
    expect(launchClick(menu, library[1])).toEqual({
      kind: 'launch',
      agent: 'claude',
      skill: 'wayfinder',
    })
  })

  it('offers the box first for a needs-context skill', () => {
    const menu = launchMenu([agent({ name: 'claude' })], 'claude', library)
    expect(launchClick(menu, library[2])).toEqual({
      kind: 'context',
      agent: 'claude',
      skill: library[2],
    })
  })

  // Nothing remembered now opens on the first *present* agent, so an unchosen
  // menu is one where no agent is launchable at all.
  it('does not launch while no agent is chosen', () => {
    const menu = launchMenu([agent({ name: 'claude', present: false })], undefined, library)
    expect(launchClick(menu, library[0])).toBeNull()
  })

  it('does not open the box while no agent is chosen either', () => {
    const menu = launchMenu([agent({ name: 'claude', present: false })], undefined, library)
    expect(launchClick(menu, library[2])).toBeNull()
  })

  it('does not launch when the library is empty', () => {
    const menu = launchMenu([], undefined, library)
    expect(launchClick(menu, library[0])).toBeNull()
  })
})

describe('launchPayload', () => {
  it('carries a typed line as the context', () => {
    expect(launchPayload('claude', 'grill', 'the retry budget')).toEqual({
      agent: 'claude',
      skill: 'grill',
      context: 'the retry budget',
    })
  })

  it('omits the context entirely when the line is empty — an empty box is a valid launch', () => {
    expect(launchPayload('claude', 'grill', '')).toEqual({ agent: 'claude', skill: 'grill' })
  })

  it('treats a whitespace-only line as empty', () => {
    expect(launchPayload('claude', 'grill', '   \t ')).toEqual({ agent: 'claude', skill: 'grill' })
  })

  it('trims the line it does carry', () => {
    expect(launchPayload('claude', 'research', '  bits-ui popovers  ').context).toBe(
      'bits-ui popovers',
    )
  })
})

describe('contextHint', () => {
  it('names the skill that will read the line', () => {
    expect(contextHint(library[2])).toBe('What should grill work on?')
  })
})

describe('agentModel', () => {
  it('surfaces a --model flag from the resolved command', () => {
    expect(agentModel(agent({ command: ['claude', '--model', 'sonnet', '<opener>'] }))).toBe(
      'sonnet',
    )
  })

  it('reads the -m short flag', () => {
    expect(agentModel(agent({ command: ['claude', '-m', 'opus', '<opener>'] }))).toBe('opus')
  })

  it('reads the --model=value form', () => {
    expect(agentModel(agent({ command: ['claude', '--model=haiku'] }))).toBe('haiku')
  })

  it('is undefined when the agent names no model', () => {
    expect(agentModel(agent({ command: ['claude', '--prompt', '<opener>'] }))).toBeUndefined()
  })
})
