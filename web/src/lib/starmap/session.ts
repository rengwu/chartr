// The session overlay's vocabulary: what a live session looks like on top of a
// star, and how that state is derived from the pushed model.
//
// The base palette (theme.ts) is spent on the six derived statuses. The harness
// adds a second axis — a session's pipeline stage and its liveness — so the
// overlay is deliberately *not* more colour: the session is a **body**, an amber
// moon orbiting the star it holds (spec, stories 25–27). Liveness is the moon's
// motion; the pipeline is where the moon sits. The one new hue is the violet
// counter-orbiter of agent review, and human review breaks the orbital grammar on
// purpose, because a call to action must not read like one more orbital fact.
//
// Every state carries a **non-colour channel** — motion or shape — so nothing
// here is colour-only. GRAMMAR is that promise written down: the renderer draws
// from it, and the seam test asserts the property over it.

import type { Map as WMap, Terminal, Ticket } from '../model'
import { SESSION_HUE } from './theme'

export type SessionState =
  | 'implementing' // a live session, its PTY producing
  | 'quiet' // an AFK session silent past the threshold, no answer yet (a hint)
  | 'dead' // the PTY exited mid-ticket; the claim stands, the star halts to you
  | 'proposed' // work landed and committed, nobody circling — awaiting the gate
  | 'agent-review' // an adversarial session circling the proposal
  | 'human-review' // the review ran; the brief awaits you — the one star that wants you

// How the moon carries each state. `motion` and `marks` are the non-colour
// channels; `hue` is the colour one. The renderer reads this record rather than
// re-encoding the grammar in its branches, so the test's no-colour-only property
// is a property of what actually gets drawn.
export interface Grammar {
  hue: string
  // The primary moon's carriage: it orbits, crawls, or holds still.
  motion: 'orbit' | 'crawl' | 'still'
  // Where the moon sits: circling the star, docked at its rim, or frozen mid-orbit.
  moon: 'orbiting' | 'docked' | 'frozen'
  // Shapes drawn beside the moon.
  marks: readonly ('trail' | 'blink' | 'halo' | 'counter-orbit' | 'ping-rings')[]
}

export const GRAMMAR: Record<SessionState, Grammar> = {
  implementing: { hue: SESSION_HUE.session, motion: 'orbit', moon: 'orbiting', marks: ['trail'] },
  quiet: { hue: SESSION_HUE.session, motion: 'crawl', moon: 'orbiting', marks: ['blink'] },
  dead: { hue: SESSION_HUE.dead, motion: 'still', moon: 'frozen', marks: ['halo'] },
  proposed: { hue: SESSION_HUE.human, motion: 'still', moon: 'docked', marks: [] },
  'agent-review': {
    hue: SESSION_HUE.violet,
    motion: 'still',
    moon: 'docked',
    marks: ['counter-orbit'],
  },
  'human-review': {
    hue: SESSION_HUE.beacon,
    motion: 'still',
    moon: 'docked',
    marks: ['ping-rings'],
  },
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
// with no proposed answer yet, so an idling HITL grilling deliberately shows
// nothing here).
//
// The pipeline stages read off the ticket's status and the *review* session on
// it: a proposal with a live reviewer is agent review; a proposal whose reviewer
// has exited is the verdict waiting on a human. Nothing new is stored — the
// state is a pure function of the snapshot.
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
    const st = stateOf(tk, onTicket.get(tk.num) ?? [])
    if (st) out[tk.num] = st
  }
  return out
}

function stateOf(tk: Ticket, tabs: Terminal[]): SessionState | null {
  if (tk.status === 'proposed') {
    const review = tabs.filter((t) => t.session?.role === 'review')
    if (review.some((t) => t.alive)) return 'agent-review'
    if (review.length) return 'human-review'
    return 'proposed'
  }
  // Any other status: only a session tab on the ticket says anything, and what it
  // says is its liveness.
  const live = tabs.find((t) => t.alive) ?? tabs[0]
  if (!live) return null
  if (!live.alive || live.status === 'dead') return 'dead'
  if (live.status === 'quiet') return 'quiet'
  return 'implementing'
}
