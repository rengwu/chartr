// Attention (ticket 14): pure derivations over the pushed model, computed
// fresh from `Ticket.frontier`/`Ticket.blockedBy` and
// `Terminal.session`/`Terminal.alive` — nothing here is stored, mirroring how
// ticket 13's `sessionStates` derives the moons overlay from the same snapshot.
//
// Two altitudes (spec, "The interface"):
//   - the map's own action station (`mapActionItems`/`mapActionCount`) — the
//     frontier ranked by how many tickets each directly unblocks;
//   - the sidebar's ambient cross-space echo (`spaceAttention`,
//     `spaceLiveness`) — decision-level signals only: a session halted. Ambient
//     liveness is a separate, weaker signal that never promotes into a flag.

import type { Map as WMap, Space, Ticket } from './model'

export interface ActionItem {
  ticket: Ticket
  // How many other tickets on this map are directly blocked on this one.
  unblockCount: number
}

// Direct dependents of a ticket number.
function unblockCountOf(map: WMap, num: number): number {
  return map.tickets.filter((t) => t.blockedBy?.includes(num)).length
}

// Everything actionable on one map: the spawnable frontier tickets ranked by
// unblock count (ties broken by ticket number, for determinism). The frontier is
// the whole condition — every discovered map is live, so `frontier` is the only
// thing standing between a ticket and a session.
export function mapActionItems(map: WMap): ActionItem[] {
  return map.tickets
    .filter((t) => t.frontier)
    .map((ticket) => ({ ticket, unblockCount: unblockCountOf(map, ticket.num) }))
    .sort((a, b) => b.unblockCount - a.unblockCount || a.ticket.num - b.ticket.num)
}

// The count the action-station badge shows — on the drawer's own toggle, and
// echoed onto the map's handle when the card is tucked away (story 24).
export function mapActionCount(map: WMap): number {
  return mapActionItems(map).length
}

// Summed across every map in the space — what the tucked-away handle shows
// when no one map is open yet (the picker screen, or the card dismissed).
export function spaceActionCount(space: Space): number {
  return space.maps.reduce((n, m) => n + mapActionCount(m), 0)
}

// The sidebar row's ambient "wants-you" flag (story 8: flags a row, never
// re-sorts it).
export type Attention = 'halt' | null

export function spaceAttention(space: Space): Attention {
  if (space.terminals.some((t) => t.session && !t.alive)) return 'halt'
  return null
}

// Where the flag's click lands: the halted session's ticket in this space, or
// null when nothing is halted. Derived from exactly the predicate
// `spaceAttention` tests, so the flag and its jump can never disagree — if one
// is shown, the other exists. A space can hold more than one halted terminal;
// this takes the first in terminal order, because the flag is one glyph and
// cannot offer a choice.
export function spaceHaltTarget(space: Space): { mapSlug: string; ticketNum: number } | null {
  const halted = space.terminals.find((t) => t.session && !t.alive)
  if (!halted?.session) return null
  return { mapSlug: halted.session.mapSlug, ticketNum: halted.session.ticketNum }
}

// Ambient liveness across a space's one live session (ADR 0003 caps a space
// at one) — a weaker signal than `spaceAttention`, and independent of it: a
// session can be working on one ticket while another sits halted, so both may
// be true for the same space at once.
export type Liveness = 'working' | 'quiet' | null

export function spaceLiveness(space: Space): Liveness {
  if (space.terminals.some((t) => t.session && t.status === 'working')) return 'working'
  if (space.terminals.some((t) => t.session && t.status === 'quiet')) return 'quiet'
  return null
}
