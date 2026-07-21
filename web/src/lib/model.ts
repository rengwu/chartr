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

// A terminal's live activity (mirrors the Go model.Terminal* states). An ad-hoc
// shell reads idle at the prompt (a tick), working while a foreground command runs
// (a spinner), or exited once the shell is gone. A session tab reads the session
// grammar instead (ticket 10): working while live and producing, quiet when an AFK
// session has fallen silent past the threshold with no answer written yet (a hint,
// never an alarm), and dead once its process exits — a dead session freezes pinned
// to its ticket rather than vanishing, awaiting the operator's halt choice.
export type TerminalStatus = 'idle' | 'working' | 'exited' | 'quiet' | 'dead'

// Session is a tab's ticket binding when it is a session — a PTY running an agent
// against exactly one ticket (ticket 09) — rather than an ad-hoc shell. It names
// the map and ticket the session is claimed on, the role it was spawned as, and
// the resolved agent and model. Its presence is what tells a session tab apart
// from a plain shell in the sidebar.
export interface Session {
  mapSlug: string
  ticketNum: number
  role: string
  agent: string
  model: string
}

// Terminal is one tab under its space in the sidebar. Without a `session` it is an
// ad-hoc shell — deliberately not a session (no ticket, no claim, ended by the
// human); with one it is a session bound to a ticket. Its raw bytes travel on the
// separate terminal socket keyed by `id`, never in this snapshot. `alive` goes
// false the instant the process exits.
export interface Terminal {
  id: string
  title: string
  // The process currently in the tab's foreground — the shell (or agent) itself
  // while at its prompt, or a command it is running. Falls back to the title.
  proc: string
  status: TerminalStatus
  alive: boolean
  // Set only when this tab is a session; absent on an ad-hoc shell.
  session?: Session
}

export interface Space {
  id: string
  name: string
  path: string
  // The working tree's current git branch (or a short detached-HEAD sha), read
  // live. Absent when it can't be determined — the sidebar omits it then.
  branch?: string
  pinned: boolean
  // True when the working tree carries uncommitted changes — a session's or a
  // shell's debris. A badge, never a spawn gate (story 68): the operator decides
  // whether the debris is harmless; the harness spawns into it all the same.
  dirty: boolean
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

// The payload preview (ticket 08): exactly what a session for a ticket and role
// would be told, with per-part layer provenance. `layer` is the config layer a
// prompt segment resolved from, or 'context' for an assembled bundle artifact.
export type PartLayer = Layer | 'context'

export interface PayloadSegment {
  layer: PartLayer
  label?: string
  text: string
}

// A labelled block of the payload — a resolved prompt (`kind: 'prompt'`, e.g.
// core or a role) or an assembled context artifact (`kind: 'context'`, e.g. the
// glossary, map body, ticket, or a blocker's answer).
export interface PayloadPart {
  name: string
  kind: 'prompt' | 'context'
  segments: PayloadSegment[]
}

export interface Payload {
  role: string
  ticketNum: number
  parts: PayloadPart[]
  warnings?: string[]
  markdown: string
}

// The closed role set a session can be spawned as (config.Roles), in display
// order — what the preview lets the operator choose between.
export const ROLES = ['grill', 'prototype', 'research', 'implement'] as const
export type Role = (typeof ROLES)[number]

// rolesForKind mirrors the backend (config.RolesForKind): which roles a map of a
// given kind offers to spawn. A planning map grills, prototypes, and researches;
// an implementation map implements; an unclassified map offers none, so its
// tickets show no spawn affordance until a human declares its kind.
export function rolesForKind(kind: Kind): Role[] {
  if (kind === 'planning') return ['grill', 'prototype', 'research']
  if (kind === 'implementation') return ['implement']
  return []
}

// The role a ticket's own type points at, clamped to what its kind actually
// offers — shared by the detail pane and the action station (ticket 14) so a
// one-click spawn always lands on the same default no matter which surface
// triggered it.
export function defaultRole(type: string, offered: Role[]): Role {
  const guess: Role =
    type === 'research'
      ? 'research'
      : type === 'prototype'
        ? 'prototype'
        : type === 'grilling'
          ? 'grill'
          : 'implement'
  return offered.includes(guess) ? guess : offered[0]
}

// Zero-padded ticket number (#04, #12) — the id format used everywhere a
// ticket is named in the chrome.
export function padTicket(n: number): string {
  return n < 10 ? '0' + n : String(n)
}
