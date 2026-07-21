// The session overlay's vocabulary: what a live session looks like on top of a
// star, and how that state is derived from the pushed model.
//
// The base palette (theme.ts) is spent on the six derived statuses. The harness
// adds a second axis — a session's liveness — so the overlay is deliberately
// *not* more colour: the session is a **body**, an amber moon orbiting the star
// it holds (spec, stories 25–27). Liveness is the moon's motion.
//
// Every state carries a **non-colour channel** — motion or shape — so nothing
// here is colour-only. GRAMMAR is that promise written down: the renderer draws
// from it, and the seam test asserts the property over it.

import type { Map as WMap, Terminal } from '../model'
import { SESSION_HUE } from './theme'

export type SessionState =
  | 'implementing' // a live session, its PTY producing
  | 'quiet' // an AFK session silent past the threshold, no answer yet (a hint)
  | 'dead' // the PTY exited mid-ticket; the claim stands, the star halts to you

// How the moon carries each state. `motion` and `marks` are the non-colour
// channels; `hue` is the colour one. The renderer reads this record rather than
// re-encoding the grammar in its branches, so the test's no-colour-only property
// is a property of what actually gets drawn.
export interface Grammar {
  hue: string
  // The primary moon's carriage: it orbits, crawls, or holds still.
  motion: 'orbit' | 'crawl' | 'still'
  // Where the moon sits: circling the star, or frozen mid-orbit.
  moon: 'orbiting' | 'frozen'
  // Shapes drawn beside the moon.
  marks: readonly ('trail' | 'blink' | 'halo')[]
}

export const GRAMMAR: Record<SessionState, Grammar> = {
  implementing: { hue: SESSION_HUE.session, motion: 'orbit', moon: 'orbiting', marks: ['trail'] },
  quiet: { hue: SESSION_HUE.session, motion: 'crawl', moon: 'orbiting', marks: ['blink'] },
  dead: { hue: SESSION_HUE.dead, motion: 'still', moon: 'frozen', marks: ['halo'] },
}

/** The state's channels with colour removed — what still tells it apart in greyscale. */
export function nonColorSignature(s: SessionState): string {
  const g = GRAMMAR[s]
  return [g.motion, g.moon, ...g.marks].join('|')
}

// Derive each ticket's session state from one pushed snapshot: the map's derived
// ticket statuses (ADR 0004) plus the space's terminals, whose sessions name the
// map and ticket they are claimed on (ticket 09) and carry the liveness the
// server already decided (ticket 10 — `quiet` is surfaced only for an AFK role
// with no answer written yet, so an idling HITL grilling deliberately shows
// nothing here). Nothing new is stored — the state is a pure function of the
// snapshot.
export function sessionStates(map: WMap, terminals: Terminal[]): Record<number, SessionState> {
  const out: Record<number, SessionState> = {}
  const onTicket = new Map<number, Terminal[]>()
  for (const t of terminals) {
    const s = t.session
    if (!s || s.mapSlug !== map.slug) continue
    const list = onTicket.get(s.ticketNum)
    if (list) list.push(t)
    else onTicket.set(s.ticketNum, [t])
  }
  for (const tk of map.tickets) {
    const st = stateOf(onTicket.get(tk.num) ?? [])
    if (st) out[tk.num] = st
  }
  return out
}

function stateOf(tabs: Terminal[]): SessionState | null {
  // Only a session tab on the ticket says anything, and what it says is its
  // liveness.
  const live = tabs.find((t) => t.alive) ?? tabs[0]
  if (!live) return null
  if (!live.alive || live.status === 'dead') return 'dead'
  if (live.status === 'quiet') return 'quiet'
  return 'implementing'
}
