// Attention (ticket 14): pure derivations over the pushed model, computed
// fresh from `Terminal.session`/`Terminal.alive` — nothing here is stored,
// mirroring how ticket 13's `sessionStates` derives the moons overlay from the
// same snapshot.
//
// The sidebar's ambient cross-space echo (`spaceAttention`, `spaceLiveness`) —
// decision-level signals only: a session halted. Ambient liveness is a separate,
// weaker signal that never promotes into a flag.

import type { Space } from './model'

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
//
// `blocked` — the agent has stopped on a permission prompt and is waiting on its
// human — takes the slot the old `quiet` hint held, which measured PTY silence and
// never fired for the TUI agents it was written for. The precedence is left exactly
// as it was: how `blocked` folds into the attention grammar, and whether it
// outranks a working session, is deliberately not decided here (map, Not yet
// specified — Notifications).
export type Liveness = 'working' | 'blocked' | null

export function spaceLiveness(space: Space): Liveness {
  if (space.terminals.some((t) => t.session && t.status === 'working')) return 'working'
  if (space.terminals.some((t) => t.session && t.status === 'blocked')) return 'blocked'
  return null
}
