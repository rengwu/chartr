// The derived model as it arrives over the control socket. This mirrors the Go
// `model.Model` exactly; it is the whole state a snapshot carries, replaced
// wholesale on every push (ADR 0010). Later tickets grow both sides together.

export type Layer = 'built-in' | 'workspace' | 'user'

// RoleBinding is one role's effective binding: which adapter runs on which
// model with which args, where each field was inherited from (so field-level
// inheritance is visible, story 39), and whether the adapter's binary is on the
// operator's PATH (`missing` carries the absence badge when it is not).
export interface RoleBinding {
  role: string
  adapter: string
  model: string
  args?: string[]
  adapterFrom: Layer
  modelFrom: Layer
  argsFrom: Layer
  present: boolean
  missing?: string
}

// Ticket is one ticket's derived state: its identity, type, the status derived
// from its file (open | claimed | proposed | resolved | out_of_scope, ADR 0004),
// its blockers, and whether it sits on the harness's stricter frontier.
export type TicketStatus = 'open' | 'claimed' | 'proposed' | 'resolved' | 'out_of_scope'

export interface Ticket {
  num: number
  slug: string
  title: string
  type: string
  status: TicketStatus
  blockedBy?: number[]
  frontier: boolean
  // The ticket's markdown below its H1 title — Question, Done-when, and any
  // closing answer. Inlined so the detail pane (ticket 07) reads the full ticket,
  // and a blocker's answer, from the snapshot with no second fetch.
  body?: string
}

// A map's declared lifecycle (ADR 0007). The empty string is the third state:
// an undeclared map, inert until a human classifies it.
export type Kind = '' | 'planning' | 'implementation'

// Map is one discovered wayfinder map beneath a space, derived live from
// `.plan/` and re-pushed on every filesystem notice. Rendered as-is: a malformed
// map is never dropped, only surfaced through `malformations`. `kind` gates the
// map's session actions: while it is unclassified (`''`) the map is inert, and
// `kindGuess` carries the convention proposal the classify confirm pre-fills.
export interface Map {
  slug: string
  name: string
  dir: string
  destination: string
  // The map's markdown below its H1 title — Destination, Notes, Decisions, fog.
  // Inlined so the map-material pane (ticket 07) opens from the title.
  body?: string
  tickets: Ticket[]
  finished: boolean
  kind: Kind
  kindGuess?: Kind
  malformations?: string[]
}

// Terminal is one open ad-hoc shell — a tab in the space's terminal column. It
// is deliberately not a session (no ticket, no lifecycle, ended by the human):
// its raw bytes travel on the separate terminal socket keyed by `id`, never in
// this snapshot. `alive` goes false the instant the shell exits.
export interface Terminal {
  id: string
  title: string
  alive: boolean
}

export interface Space {
  id: string
  name: string
  path: string
  pinned: boolean
  bindings: RoleBinding[]
  maps: Map[]
  terminals: Terminal[]
  warnings?: string[]
}

export interface Model {
  spaces: Space[]
}

/** A space needs an agent installed if any of its bindings is absent from PATH. */
export function needsAgents(space: Space): boolean {
  return space.bindings.some((b) => !b.present)
}
