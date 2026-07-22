// The derived model as it arrives over the control socket. This mirrors the Go
// `model.Model` exactly; it is the whole state a snapshot carries, replaced
// wholesale on every push (ADR 0010). Later tickets grow both sides together.

export type Layer = 'built-in' | 'workspace' | 'user'

// RoleBinding is one role's effective binding: which adapter runs with which
// args, where each field was inherited from (so field-level
// inheritance is visible, story 39), and whether the adapter's binary is on the
// operator's PATH (`missing` carries the absence badge when it is not).
export interface RoleBinding {
  role: string
  adapter: string
  args?: string[]
  // How the opener reaches this agent — `argv`, `type`, or a flag name like
  // `--prompt`. Absent means the adapter's own default stands, which is what
  // nearly every binding wants; it is set only to drive a harness the chartr
  // ships no knowledge of.
  prompt?: string
  adapterFrom: Layer
  argsFrom: Layer
  promptFrom: Layer
  // agent is the registered agent this role is assigned to, absent when the role
  // is bound field by field. When it is set and registered it supplied every
  // field above, so the row renders one name instead of four provenances.
  // agentMissing says the name resolved to nothing and the fields beneath it are
  // what actually runs.
  agent?: string
  agentMissing?: string
  present: boolean
  missing?: string
}

// Agent is one entry of the operator's registered agent library: a named,
// complete way to run a harness — the binary, whatever flags that harness wants
// (its model among them), and how it takes its opening prompt. The library is global
// rather than per space, so it hangs off the model; roles in every space assign
// to it by name.
export interface Agent {
  name: string
  adapter: string
  args?: string[]
  prompt?: string
  // What `prompt` resolves to once the adapter's own default is taken into
  // account — `argv`, `type`, or a flag name. Resolved server-side, so the
  // browser never re-derives the adapter table and drifts from it.
  delivery: string
  // The argv this agent would actually launch, with a placeholder standing in for
  // the opener. Built by the same seam as the real launch.
  command: string[]
  present: boolean
  missing?: string
}

// Ticket is one ticket's derived state: its identity, type, the status derived
// from its file (open | claimed | resolved | out_of_scope, ADR 0004 as amended),
// its blockers, and whether it sits on the frontier.
export type TicketStatus = 'open' | 'claimed' | 'resolved' | 'out_of_scope'

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

// Map is one discovered wayfinder map beneath a space, derived live from
// `.plan/` and re-pushed on every filesystem notice. Rendered as-is: a malformed
// map is never dropped, only surfaced through `malformations`. A discovered map
// is live — there is nothing between it arriving and opening or spawning on it.
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
// the resolved agent. Its presence is what tells a session tab apart
// from a plain shell in the sidebar.
export interface Session {
  mapSlug: string
  ticketNum: number
  role: string
  agent: string
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
  // whether the debris is harmless; the chartr spawns into it all the same.
  dirty: boolean
  bindings: RoleBinding[]
  // The resolved skill library: every skill with the layer that won its whole
  // directory and its stale-fork state (ticket 05, story 34).
  skills: ResolvedSkill[]
  // This space's own config files — its committed workspace config and committed
  // skill library. The layers it shares with every space live on `Model.config`.
  layers: ConfigLayer[]
  maps: Map[]
  terminals: Terminal[]
  warnings?: string[]
}

// ResolvedSkill is one skill of the library as it resolves for a space: which
// layer won its whole directory (whole-skill shadowing), and whether a fork has
// fallen behind the shipped default. The positive statement of resolution —
// "your grill resolves from: user" — not just the warning.
export interface ResolvedSkill {
  name: string
  layer: Layer
  // Where the winning directory sits; absent when the copy embedded in the binary
  // is the floor.
  dir?: string
  description?: string
  // The shipped content hash a shadowing skill recorded in its frontmatter, and
  // whether the shipped default has since moved past it.
  forkedFrom?: string
  stale?: boolean
}

// ConfigLayer is one file or directory a space's effective config resolves
// through. `name` is the server-side token the open action resolves — the client
// never sends a path (ADR 0014). `holds` is what the layer can set: role
// bindings or skills.
export interface ConfigLayer {
  name: string
  layer: Layer
  holds: 'bindings' | 'skills'
  path: string
  exists: boolean
}

export interface Model {
  spaces: Space[]
  // The config layers that are not any one space's: the operator's local binding
  // overrides and the two skill libraries above and below them.
  config: ConfigLayer[]
  // The skill library as it resolves with no space in play — the built-in floor
  // with the operator's own forks over it. What every space starts from before
  // its committed library shadows anything.
  skills: ResolvedSkill[]
  // The operator's registered agent library — named launch specs any space's
  // roles may be assigned to. Global: it lives in the operator's own config and
  // is never committed, so it is the same list whatever space is in view.
  agents: Agent[]
  // Whether this machine can raise a native OS folder chooser for "add a space".
  // A machine capability, not state: true on macOS, true on Linux with zenity or
  // kdialog, false otherwise — and it is what decides whether New Space opens the
  // operator's own chooser or falls back to asking them to paste a path.
  nativePicker: boolean
}

/** A space needs an agent installed if any of its bindings is absent from PATH. */
export function needsAgents(space: Space): boolean {
  return space.bindings.some((b) => !b.present)
}

// The payload preview (ticket 08): exactly what a session for a ticket and role
// would be told, with per-part layer provenance. `layer` is the config layer a
// skill segment resolved from, or 'context' for an assembled bundle artifact.
export type PartLayer = Layer | 'context'

export interface PayloadSegment {
  layer: PartLayer
  label?: string
  text: string
}

// A labelled block of the payload — a resolved skill (`kind: 'prompt'`, e.g.
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

// The role a ticket's own type points at (mirrors config.RoleForTicketType) —
// shared by the detail pane, the action station (ticket 14) and the payload
// preview so a one-click spawn always lands on the same default no matter which
// surface triggered it. The type says exactly which role the ticket is; nothing
// clamps it, and every ticket offers all four roles for the operator to pick
// from at the gate.
export function defaultRole(type: string): Role {
  return type === 'research'
    ? 'research'
    : type === 'prototype'
      ? 'prototype'
      : type === 'grilling'
        ? 'grill'
        : 'implement'
}

// Zero-padded ticket number (#04, #12) — the id format used everywhere a
// ticket is named in the chrome.
export function padTicket(n: number): string {
  return n < 10 ? '0' + n : String(n)
}
