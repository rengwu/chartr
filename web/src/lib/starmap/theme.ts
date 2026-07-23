// The star-map's visual vocabulary for the five base states, cribbed from the
// shipped wayfinder-maps viewer's palette so the feel carries over (ADR 0010:
// constants cribbed to prevent feel-drift). Status is the whole star — colour,
// size, glow, pulse (starmap-design.md, decision 4). Ticket type rides only in
// the label, never the celestial body.
//
// chartr derives its ticket status from `.plan/` (ADR 0004, amended): a
// ticket is open, claimed, resolved, or out_of_scope, with a `frontier` flag
// splitting open into the takeable edge and the still-blocked interior. That is
// exactly five visual states, and this module maps the derived status onto them.
// Session liveness (working / blocked / dead) is a strictly-additive overlay a
// later ticket layers on top — this palette is only the base star.

import type { Ticket } from '../model'

export type VisualState =
  | 'resolved'
  | 'frontier'
  | 'claimed'
  | 'blocked'
  | 'out_of_scope'

export interface StarStyle {
  core: string
  glow: string
  r: number
  gr: number
}

// The five base states, ported verbatim from the viewer's theme — chartr
// adds none of its own (the `proposed` star went with the review feature).
// Ticket 04: the card the map sits on moved from a near-black `#05070d` to the
// theme's warm near-black `--card` (`oklch(0.228 0.013 107.4)`, ~`#1d1d16`) —
// meaningfully lighter than before. Four of five states still clear WCAG-ish
// contrast comfortably against it; `out_of_scope`, deliberately the dimmest
// star, fell under 4:1. Its three values are lifted just enough to stay
// legible on the new card; sizes, glow radii, and every other state are
// untouched (map decision: this is a palette re-tune, not a renderer change).
export const STAR: Record<VisualState, StarStyle> = {
  resolved: { core: '#b9d6c4', glow: '#5b9077', r: 5.4, gr: 24 },
  frontier: { core: '#8ad8ff', glow: '#2f9be0', r: 8.1, gr: 49 },
  claimed: { core: '#ffd873', glow: '#ffb020', r: 7.2, gr: 36 },
  blocked: { core: '#e2c3c3', glow: '#9a6f6f', r: 4.5, gr: 20 },
  out_of_scope: { core: '#948da4', glow: '#6b6478', r: 4.5, gr: 18 },
}

export const LABEL: Record<VisualState, string> = {
  resolved: '#a2c1ac',
  frontier: '#b3e5ff',
  claimed: '#ffe6a0',
  blocked: '#d0b3b3',
  out_of_scope: '#a89fb2',
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
    case 'out_of_scope':
      return 'out_of_scope'
    case 'open':
    default:
      return t.frontier ? 'frontier' : 'blocked'
  }
}

// The session overlay's hues (ticket 13). They live here with the five base
// states because the star-map's palette is the island's own exempt data-viz
// colour (docs/design-system.md) — the chrome around it stays monochrome. The
// grammar these serve, and the non-colour channel each state also carries, is
// session.ts; the amber is the spec's session moon (story 25), so the set is
// closed here, not open to growth.
export const SESSION_HUE = {
  // The session itself: the same amber as a live claim, because the moon *is*
  // the claim's body.
  session: '#ffd873',
  // The island's own chrome: the ticker line naming what just changed.
  gold: '#ffe6a0',
  // A dead session greys its whole orbital apparatus, not just the moon.
  dead: '#6b7280',
} as const

export function hexA(hex: string, a: number): string {
  const r = parseInt(hex.slice(1, 3), 16),
    g = parseInt(hex.slice(3, 5), 16),
    b = parseInt(hex.slice(5, 7), 16)
  return `rgba(${r},${g},${b},${a})`
}
