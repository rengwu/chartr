// The star-map's visual vocabulary for the six base states, cribbed from the
// shipped wayfinder-maps viewer's palette so the feel carries over (ADR 0010:
// constants cribbed to prevent feel-drift). Status is the whole star — colour,
// size, glow, pulse (starmap-design.md, decision 4). Ticket type rides only in
// the label, never the celestial body.
//
// The harness derives its ticket status from `.plan/` (ADR 0004): a ticket is
// open, claimed, proposed, resolved, or out_of_scope, with a `frontier` flag
// splitting open into the takeable edge and the still-blocked interior. That is
// exactly six visual states, and this module maps the derived status onto them.
// Session liveness (working / quiet / dead) and the review grammar (moons,
// beacon) are a strictly-additive overlay a later ticket layers on top — this
// palette is only the base star.

import type { Ticket } from '../model'

export type VisualState =
  | 'resolved'
  | 'frontier'
  | 'claimed'
  | 'proposed'
  | 'blocked'
  | 'out_of_scope'

export interface StarStyle {
  core: string
  glow: string
  r: number
  gr: number
}

// The six base states. resolved/frontier/claimed/blocked/out_of_scope are ported
// verbatim from the viewer's theme; `proposed` is the harness's one added base
// status (ADR 0004) — a warm, sealed star that reads as "work has landed, the
// gate has not yet blessed it", distinct from the amber of a live claim.
export const STAR: Record<VisualState, StarStyle> = {
  resolved: { core: '#b9d6c4', glow: '#5b9077', r: 5.4, gr: 24 },
  frontier: { core: '#8ad8ff', glow: '#2f9be0', r: 8.1, gr: 49 },
  claimed: { core: '#ffd873', glow: '#ffb020', r: 7.2, gr: 36 },
  proposed: { core: '#ffedbe', glow: '#d9a441', r: 6.6, gr: 30 },
  blocked: { core: '#e2c3c3', glow: '#9a6f6f', r: 4.5, gr: 20 },
  out_of_scope: { core: '#7d7789', glow: '#4a4550', r: 4.5, gr: 18 },
}

export const LABEL: Record<VisualState, string> = {
  resolved: '#a2c1ac',
  frontier: '#b3e5ff',
  claimed: '#ffe6a0',
  proposed: '#ffe6bf',
  blocked: '#d0b3b3',
  out_of_scope: '#8a8496',
}

// Derive the visual state of a ticket from its pushed status and frontier flag.
// The frontier flag is what splits an open ticket into the bright, takeable
// `frontier` star and the small, dim `blocked` one — the whole reason the map
// exists is this at-a-glance read.
export function visualState(t: Pick<Ticket, 'status' | 'frontier'>): VisualState {
  switch (t.status) {
    case 'resolved':
      return 'resolved'
    case 'claimed':
      return 'claimed'
    case 'proposed':
      return 'proposed'
    case 'out_of_scope':
      return 'out_of_scope'
    case 'open':
    default:
      return t.frontier ? 'frontier' : 'blocked'
  }
}

export function hexA(hex: string, a: number): string {
  const r = parseInt(hex.slice(1, 3), 16),
    g = parseInt(hex.slice(3, 5), 16),
    b = parseInt(hex.slice(5, 7), 16)
  return `rgba(${r},${g},${b},${a})`
}
