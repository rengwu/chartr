// Attention (ticket 14): pure derivations over the pushed model, computed
// fresh from `Ticket.frontier`/`Ticket.blockedBy` and
// `Terminal.session`/`Terminal.alive` — nothing here is stored, mirroring how
// ticket 13's `sessionStates` derives the moons overlay from the same snapshot.
//
// Two altitudes (spec, "The interface"):
//   - the map's own action station (`mapActionItems`/`mapActionCount`) — the
//     frontier ranked by how many tickets each directly unblocks;
//   - the cross-space "Needs you" queue (`needsYouQueue`) and the sidebar's
//     ambient echo (`spaceAttention`, `spaceLiveness`) — decision-level signals
//     only: a session halted. Ambient liveness is a separate, weaker signal that
//     never promotes into the pull-only queue.

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
// unblock count (ties broken by ticket number, for determinism). An
// unclassified map offers no spawn affordance (ADR 0007), so it contributes
// nothing here even if the model already marks a ticket `frontier`.
export function mapActionItems(map: WMap): ActionItem[] {
  if (map.kind === '') return []

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

// One row in the cross-space "Needs you" queue: exactly the decision-level
// signals (spec story 63) — never plain liveness, which stays ambient-only.
export interface QueueEntry {
  spaceId: string
  spaceName: string
  mapSlug: string
  mapName: string
  ticketNum: number
  ticketTitle: string
  kind: 'halt'
}

// Halted sessions across every space. Never sorted by recency or anything
// else: the queue is a flat, small, pull-only list (strictly summoned, never
// auto-surfaced).
export function needsYouQueue(spaces: Space[]): QueueEntry[] {
  const halts: QueueEntry[] = []

  for (const space of spaces) {
    for (const t of space.terminals) {
      if (!t.session || t.alive) continue
      const map = space.maps.find((m) => m.slug === t.session!.mapSlug)
      const ticket = map?.tickets.find((tk) => tk.num === t.session!.ticketNum)
      halts.push({
        spaceId: space.id,
        spaceName: space.name,
        mapSlug: t.session.mapSlug,
        mapName: map?.name ?? t.session.mapSlug,
        ticketNum: t.session.ticketNum,
        ticketTitle: ticket?.title ?? `#${t.session.ticketNum}`,
        kind: 'halt',
      })
    }
  }
  return halts
}

// The sidebar row's ambient "wants-you" flag (story 8: flags a row, never
// re-sorts it) — exactly the same condition the queue pulls for that space,
// so the two never disagree.
export type Attention = 'halt' | null

export function spaceAttention(space: Space): Attention {
  if (space.terminals.some((t) => t.session && !t.alive)) return 'halt'
  return null
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
