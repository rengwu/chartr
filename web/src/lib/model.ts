// The derived model as it arrives over the control socket. This mirrors the Go
// `model.Model` exactly; it is the whole state a snapshot carries, replaced
// wholesale on every push (ADR 0010). Later tickets grow both sides together.

// Layer names where a skill resolved from — the shipped floor, the operator's own
// fork, or a space's committed library. Skill resolution is the content half of
// the config story; execution is no longer layered at all (agents.ts).
export type Layer = 'built-in' | 'workspace' | 'user'

// Agent is one entry of the operator's registered agent library: a named,
// complete way to run a harness — the binary, whatever flags that harness wants
// (its model among them), and how it takes its opening prompt. The library is
// global and is the only execution config there is; a spawn picks from it at the
// gate.
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

// A terminal's live activity (mirrors the Go model.Terminal* states). Which
// grammar a tab reads on follows what holds its PTY's foreground, not whether it
// is a session.
//
// A tab with no known agent in front reads the shell grammar: idle at the prompt
// (a tick), working while a foreground command runs (a spinner), exited once the
// process is gone. A tab with a known agent reads that agent's own broadcast
// state: idle when it is present but not generating, working while it is, and
// blocked when it has stopped on a permission prompt and is waiting on its human —
// the one state worth an operator's attention. `dead` belongs to sessions: a dead
// session freezes pinned to its ticket rather than vanishing, awaiting the
// operator's halt choice.
export type TerminalStatus = 'idle' | 'working' | 'exited' | 'blocked' | 'dead'

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
  // whether the debris is harmless; chartr spawns into it all the same.
  dirty: boolean
  // The registered agent this space last spawned with — the remembered choice the
  // next spawn reuses. State, never config: nothing edits it, and it arrives
  // exactly as the server holds it. A name that no longer matches a registered
  // agent reads as *nothing remembered* here, which re-opens the picker rather
  // than substituting one. Absent until the space's first spawn.
  lastAgent?: string
  // The resolved skill library: every skill with the layer that won its whole
  // directory and its stale-fork state (story 34).
  skills: ResolvedSkill[]
  // This space's own config files — its committed skill library. The layers it
  // shares with every space live on `Model.config`.
  layers: ConfigLayer[]
  maps: Map[]
  terminals: Terminal[]
  warnings?: string[]
  // chartr's standing offer to write its own tracker adapter
  // (docs/agents/issue-tracker.md) into this space, redirecting a vanilla
  // wayfinder skill's map reads to .plan/maps/ in chartr's format. Present only
  // when there is something to act on and the operator hasn't dismissed it — an
  // up-to-date or dismissed adapter never rides the wire — so "show the prompt iff
  // space.trackerAdapter exists" is the whole gating rule.
  trackerAdapter?: TrackerAdapterOffer
}

// TrackerAdapterOffer is chartr's live read of a space's tracker adapter, on the
// snapshot only while it wants the operator's hand. The three states name the one
// action each earns:
//   - absent  → Install (a clean first write)
//   - stale   → Refresh (chartr's own copy drifted — a template bump or an edit)
//   - foreign → Replace or Leave (a non-chartr file is in the way)
export interface TrackerAdapterOffer {
  state: 'absent' | 'stale' | 'foreign'
  // Absolute path of the docs/agents/issue-tracker.md the action writes.
  path: string
  // A cosmetic guess at what foreign file is in the way ('gh'|'glab'|'linear'),
  // for phrasing only; absent on the non-foreign states.
  remoteHint?: string
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
  // Whether the sidebar launcher may open this skill cold (`on-ramp: true`), and
  // whether it offers the optional one-line context box first (`needs-context:
  // true`). Declared in the skill's own frontmatter, so an operator's own on-ramp
  // skill appears in the picker with no chartr-side change.
  onRamp?: boolean
  needsContext?: boolean
}

// ConfigLayer is one file or directory the operator's config lives in. `name` is
// the server-side token the open action resolves — the client never sends a path.
// `holds` is what the file carries: the agent library, skills, or the per-machine
// terminal customization.
export interface ConfigLayer {
  name: string
  layer: Layer
  holds: 'agents' | 'skills' | 'terminal'
  path: string
  exists: boolean
}

export interface Model {
  spaces: Space[]
  // The config files that are not any one space's: the operator's agent library
  // and the two skill libraries.
  config: ConfigLayer[]
  // The skill library as it resolves with no space in play — the built-in floor
  // with the operator's own forks over it. What every space starts from before
  // its committed library shadows anything.
  skills: ResolvedSkill[]
  // The operator's registered agent library — named launch specs a spawn picks
  // from at the gate. Global: it lives in the operator's own config and is never
  // committed, so it is the same list whatever space is in view.
  agents: Agent[]
  // The known agent CLIs found on this machine's PATH, in a curated order — the
  // advisory hint the registration surface renders beneath the adapter input so a
  // fresh operator need not recall exact binary names. A suggestion, never a
  // constraint: any binary can be registered whether or not it is here.
  detected: string[]
  // Whether this machine can raise a native OS folder chooser for "add a space".
  // A machine capability, not state: true on macOS, true on Linux with zenity or
  // kdialog, false otherwise — and it is what decides whether New Space opens the
  // operator's own chooser or falls back to asking them to paste a path.
  nativePicker: boolean
  // The operator's resolved terminal customization — the per-machine
  // `terminal.toml` parsed server-side (Seam 1) and carried here so every terminal
  // island reads the same settings. Global: the file is per-machine and never
  // committed. Absent / all-empty means all defaults — today's look. The client
  // resolve seam (`buildTerminalOptions` in tokens.ts) turns it into the concrete
  // xterm options and theme, falling every unset slot through to the design token.
  terminal?: TerminalPrefs
}

// TerminalPrefs mirrors the Go `model.TerminalPrefs`: every field is a pref the
// `terminal.toml` set, and a field left unset (empty / zero) falls through to the
// app default at the resolve seam. Ticket 01 carried the spine — font family,
// size, and the two base theme colours; ticket 02 widened it to a named theme
// preset plus the full slot set; ticket 03 adds the pass-through font, cursor,
// scrolling, contrast, and glyph-width options; ticket 04 adds the scrollbar,
// padding, and the keybinding/selection behaviours; ticket 05 adds the ligatures
// toggle, which the resolve seam turns into the renderer choice. The resolve seam
// (`buildTerminalOptions`) layers the theme as tokens → preset → explicit slots and
// maps every other option onto the xterm options object. `preset` is a validated
// bundled name (server-side); `selection` drives xterm's `selectionBackground`.
export interface TerminalPrefs {
  fontFamily?: string
  fontSize?: number
  // A normalised weight: 'normal', 'bold', or a numeric string like '600'. The
  // resolve seam passes a keyword through and a numeric string as a number.
  fontWeight?: string
  fontWeightBold?: string
  lineHeight?: number
  letterSpacing?: number

  // Enables the ligatures addon. `resolveRenderer` turns it on only for a bundled
  // font and forces that terminal onto the canvas renderer (WebGL off) — the
  // ligatures addon and WebGL cannot coexist. It is an addon/renderer toggle, not
  // an xterm option, so it is not part of buildTerminalOptions' output.
  ligatures?: boolean

  cursorStyle?: string
  cursorBlink?: boolean
  cursorInactiveStyle?: string
  cursorWidth?: number

  scrollback?: number
  scrollSensitivity?: number
  fastScrollModifier?: string
  fastScrollSensitivity?: number
  smoothScrollDuration?: number

  minimumContrastRatio?: number

  // Gates the unicode11 addon (wide-glyph/emoji widths). The island reads it off
  // the prefs and lazily imports the addon at mount when set — it is an addon
  // toggle, not an xterm option, so it is not part of buildTerminalOptions' output.
  unicode11?: boolean

  // The scrollbar and the padding are CSS, not xterm options — xterm exposes no
  // options for either, so the resolve seam emits them as custom properties the
  // island sets on its host (`buildTerminalOptions().css`).
  scrollbarWidth?: number
  scrollbarThumb?: string
  scrollbarTrack?: string
  scrollbarAutoHide?: boolean

  paddingTop?: number
  paddingRight?: number
  paddingBottom?: number
  paddingLeft?: number

  // The keybinding and selection behaviours. `shiftEnterNewline` is unset-means-*on*
  // (the Ghostty-style newline is what the cockpit ships; set it false for plain
  // submit-on-Shift+Enter); the other three are unset-means-off, matching xterm.
  // `copyOnSelect` is an island behaviour (xterm has no such option); the other two
  // pass through as xterm options.
  shiftEnterNewline?: boolean
  copyOnSelect?: boolean
  rightClickSelectsWord?: boolean
  macOptionIsMeta?: boolean

  preset?: string

  background?: string
  foreground?: string
  cursor?: string
  cursorAccent?: string
  selection?: string

  black?: string
  red?: string
  green?: string
  yellow?: string
  blue?: string
  magenta?: string
  cyan?: string
  white?: string

  brightBlack?: string
  brightRed?: string
  brightGreen?: string
  brightYellow?: string
  brightBlue?: string
  brightMagenta?: string
  brightCyan?: string
  brightWhite?: string
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
