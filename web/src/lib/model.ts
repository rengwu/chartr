// The derived model as it arrives over the control socket. This mirrors the Go
// `model.Model` exactly; it is the whole state a snapshot carries, replaced
// wholesale on every push (ADR 0010). Later tickets grow both sides together.

// Agent is one entry of the operator's registered agent library: a named,
// complete way to run a harness — the binary, whatever flags that harness wants
// (its model among them), and how it takes its opening prompt. The library is
// global and is the only execution config there is; a spawn picks from it at the
// gate.
export interface Agent {
  name: string
  adapter: string
  args?: string[]
  // The environment this agent launches with, as `KEY=VALUE` entries — resolved
  // server-side, so a `~/` the operator typed arrives here already expanded to the
  // path the process will actually be given.
  env?: string[]
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
  // The claim the ticket file carries: the session id stamped by the claim
  // and when (RFC 3339). Present only while `claimed`, and read straight off the
  // frontmatter — so a claim written by a chartr that has since restarted, or on
  // another machine entirely, reads exactly as a local one does.
  claimedBy?: string
  claimedAt?: string
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
export type TerminalStatus = 'idle' | 'working' | 'running' | 'exited' | 'blocked' | 'dead'

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
  // The tab finished a run long enough to be worth interrupting the operator over
  // — the same event the OS notification carries — and they have not looked at it
  // since. Server state, so it is here on a reload: the run may have ended with no
  // browser attached at all. Cleared by focusing the tab, which is the only
  // acknowledgement there is (`unseen.ts`).
  finishedUnseen?: boolean
  // A generated one-line summary of the tab's recent session, shown as a muted
  // scrolling marquee after the tab's label. Absent until a generation lands, and
  // always absent on a tab with no agent in front or when the operator turned
  // auto-titling off.
  autoTitle?: string
}

export interface Space {
  id: string
  name: string
  path: string
  // The one synthetic Scratch entry is flagged; registered spaces omit this.
  scratch?: boolean
  // The registered agent this space last spawned with — the remembered choice the
  // next spawn reuses. State, never config: nothing edits it, and it arrives
  // exactly as the server holds it. A name that no longer matches a registered
  // agent reads as *nothing remembered* here, which re-opens the picker rather
  // than substituting one. Absent until the space's first spawn.
  lastAgent?: string
  maps: Map[]
  terminals: Terminal[]
  warnings?: string[]
}

// ConfigLayer is one file or directory the operator's config lives in. `name` is
// the server-side token the open action resolves — the client never sends a path.
// `holds` is what the file carries: the agent library, the per-machine terminal
// customization, machine-wide notification timing, the skill-source list, the
// operator's own preferences, or one of the two core overrides that shadow
// chartr's embedded bytes — the ticket core and the standing-document core.
export interface ConfigLayer {
  name: string
  holds:
    | 'agents'
    | 'terminal'
    | 'notifications'
    | 'sources'
    | 'preferences'
    | 'ticket core'
    | 'space core'
    | 'autotitle'
  path: string
  exists: boolean
}

// Source is one row of the skill-source list: what the operator registered, plus
// what a walk of it found on this rebuild. Position in `Model.sources` *is*
// resolution order — the list is rendered in arrival order and never sorted.
export interface Source {
  name: string
  // 'dir' is a folder the operator owns and edits; 'git' is a checkout *chartr*
  // owns, which a refresh resets — the row has to say so, because the answer to
  // "I want to edit this" is a dir source, not an edit inside the checkout.
  kind: 'dir' | 'git'
  path: string
  url?: string
  ref?: string
  commit?: string
  // RFC3339, absent when never fetched.
  fetched?: string
  enabled: boolean
  status: 'ok' | 'unavailable' | 'empty'
  skills: string[]
  // The subset of `skills` an earlier enabled source already claimed at the bare
  // name. They stay reachable through the qualified `Source/skill` form.
  shadowed?: string[]
  warnings?: string[]
}

// RoleBinding is one role beside the skill it is bound to. An empty `ref` is an
// unbound role — the first-run state now that chartr seeds no bindings, and a
// legitimate later state too: the role refuses to spawn until the operator binds
// it against a source they registered.
export interface RoleBinding {
  role: Role
  // The stored value: `auto` (no preference — resolve by source precedence), a
  // `Source/skill` ref (pinned), or empty (unbound, refuses to spawn).
  ref: string
  // Whether `ref` names a skill some enabled source yields today. Bound-but-
  // unresolvable is the other way a role refuses, and the row tells them apart.
  resolves: boolean
  // The `Source/skill` a spawn would actually run today — the pin itself when it
  // resolves, or the precedence winner when `ref` is `auto`. Empty when nothing
  // resolves. Shown beside the role so "no preference" is never a mystery.
  resolved: string
  // Every `Source/skill` an enabled source offers that this role accepts, in
  // resolution order — the options the picker lists beneath "no preference". A
  // pinned ref rides here too even when it is not an alias match, so the picker
  // always has an option matching what is set.
  candidates: string[]
}

export interface Model {
  spaces: Space[]
  // The config files that are not any one space's: the operator's agent library
  // and their terminal and notification preferences.
  config: ConfigLayer[]
  // The operator's registered agent library — named launch specs a spawn picks
  // from at the gate. Global: it lives in the operator's own config and is never
  // committed, so it is the same list whatever space is in view.
  agents: Agent[]
  // The known agent CLIs found on this machine's PATH, in a curated order — the
  // advisory hint the registration surface renders beneath the adapter input so a
  // fresh operator need not recall exact binary names. A suggestion, never a
  // constraint: any binary can be registered whether or not it is here.
  detected: string[]
  // The operator's ordered skill-source list, each row with what a walk of it
  // just found. In resolution order — first enabled source to yield a name wins.
  sources: Source[]
  // The four role bindings as they stand in `user.toml`, each beside whether it
  // resolves. An unbound role rides with an empty ref.
  roles: RoleBinding[]
  // Whether `git` is on this machine's PATH. A git source cannot be registered
  // without it, and the refusal is at the gate — before a row is written.
  gitAvailable: boolean
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
  // The complete machine-wide run notification rule resolved from notify.toml.
  // Durations use the same strings the file accepts, and are always present
  // because absent or invalid keys have already fallen back server-side.
  notify?: NotifyPrefs
  // The operator's resolved machine-wide auto-title toggle from autotitle.toml —
  // whether idle agent tabs are summarised into labels. The server always sends it;
  // optional here only so partial test fixtures need not spell it out.
  autoTitle?: AutoTitlePrefs
}

export interface NotifyPrefs {
  after: string
  settle: string
  enabled: boolean
}

// The resolved autotitle.toml value: the machine's one auto-title setting.
export interface AutoTitlePrefs {
  enabled: boolean
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

// The payload preview: exactly what a session would be told, with per-part
// provenance. `origin` is an open string — 'chartr' for the embedded core and the
// conventions pointer, 'operator' for preferences, 'context' for anything
// assembled at compose time, and otherwise the *registered source's name* for a
// resolved skill body, which is the badge that answers the one silent failure
// source order can cause.
export type PartOrigin = 'chartr' | 'operator' | 'context' | (string & {})

// A labelled block of the payload — chartr's core, a resolved role skill, the
// conventions pointer or the operator's preferences (`kind: 'prompt'`), or an
// assembled context artifact (`kind: 'context'`: the sources block, the map body,
// the ticket, a blocker's answer). One block of text, never a segment list.
export interface PayloadPart {
  name: string
  kind: 'prompt' | 'context'
  origin: PartOrigin
  label?: string
  text: string
}

// `role` and `ticketNum` are empty on a free session's payload, which belongs to
// no ticket and no role.
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
// shared by the detail pane and the payload preview so a one-click spawn always
// lands on the same default no matter which surface triggered it. The type says
// exactly which role the ticket is; nothing
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

// heldLive reports whether a live session in these tabs is running on one ticket.
// A claimed ticket with no such tab is *orphaned* as far as this chartr is
// concerned: the claim stands on disk but nothing here is working it, which is the
// whole state the ticket-level release exists for. A dead pinned tab does not
// count — its session is over, and its claim is exactly what wants clearing.
//
// This is a view fact derived from live tabs, never a fifth status on the ticket:
// what the file says is `claimed`, and only the server's own refusal decides
// whether a release is allowed.
export function heldLive(terminals: Terminal[], slug: string, num: number): boolean {
  return terminals.some(
    (t) => t.alive && t.session?.mapSlug === slug && t.session?.ticketNum === num,
  )
}

// sinceLabel renders how long ago an RFC 3339 timestamp was, coarsely — the age of
// a claim is read for "is this stuck?", never for precision, so it stops at days.
// An unparseable or absent stamp renders nothing rather than a wrong number.
export function sinceLabel(at: string | undefined, now: number = Date.now()): string {
  if (!at) return ''
  const then = Date.parse(at)
  if (Number.isNaN(then)) return ''
  const mins = Math.floor((now - then) / 60000)
  if (mins < 1) return 'just now'
  if (mins < 60) return `${mins}m ago`
  const hours = Math.floor(mins / 60)
  if (hours < 24) return `${hours}h ago`
  return `${Math.floor(hours / 24)}d ago`
}
