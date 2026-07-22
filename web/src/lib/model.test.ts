import { describe, expect, it } from 'vitest'
import { defaultRole, ROLES, type Role } from './model'

// The role a ticket spawns as comes from the ticket's own type — the one
// behaviour the kind cut changes. A map's kind used to clamp this to the roles
// its lifecycle offered; nothing clamps it now.
describe('defaultRole', () => {
  it('maps each of wayfinder’s four ticket types onto its role', () => {
    expect(defaultRole('grilling')).toBe('grill')
    expect(defaultRole('prototype')).toBe('prototype')
    expect(defaultRole('research')).toBe('research')
    expect(defaultRole('task')).toBe('implement')
  })

  it('falls to implement on an unrecognised type', () => {
    expect(defaultRole('')).toBe('implement')
    expect(defaultRole('spelunking')).toBe('implement')
  })

  it('never returns a role outside the closed set', () => {
    for (const type of ['grilling', 'prototype', 'research', 'task', 'nonsense']) {
      expect(ROLES).toContain(defaultRole(type) as Role)
    }
  })
})
