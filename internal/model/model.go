// Package model holds the derived model chartr pushes to every browser.
//
// The whole model travels over the control socket as a single JSON snapshot on
// every change (ADR 0010): it is server-authoritative, small enough that
// diffing buys nothing, and re-sent wholesale on reconnect. The walking
// skeleton ships it near-empty — the point of this slice is the transport, and
// every later ticket hangs its state (spaces, maps, tickets, sessions)
// off these fields.
package model

// Model is the complete derived state of the cockpit at one instant. A browser
// replaces its entire view state with each snapshot it receives; there is no
// client-side merge.
type Model struct {
	// Spaces are the registered spaces and everything derived beneath them.
	// Empty until ticket 02 registers the first space. Never nil on the wire —
	// New seeds an empty slice so the snapshot is always a well-formed array.
	Spaces []Space `json:"spaces"`
	// Config are the config layers that are not a space's own — the operator's
	// one local config file and the two skill libraries above and below it. They
	// participate in resolving every space, so they are derived once here rather
	// than repeated under each one, and the settings route reads them as the
	// global half of the effective config surface (ADR 0014). Never nil.
	Config []ConfigLayer `json:"config"`
	// Agents is the operator's registered agent library — named launch specs a
	// role in any space may be assigned to. Global rather than per space (it lives
	// in the operator's own config and is never committed), so it is derived once
	// here beside the layers that resolve it. Never nil.
	Agents []Agent `json:"agents"`
	// Detected are the known agent CLIs found on this machine's PATH, in a curated
	// order — the advisory hint the registration surface renders beneath the adapter
	// input so a fresh operator need not recall exact binary names. It is a
	// suggestion, never a constraint: any binary can be registered whether or not it
	// is here, and chartr asserts only that each name exists on PATH (ADR 0002).
	// Never nil on the wire.
	Detected []string `json:"detected"`
	// Terminal is the operator's resolved terminal customization — the per-machine
	// `terminal.toml` beside the agent library, parsed once (server Seam 1) and
	// carried here so every terminal island reads the same settings the way it
	// reads the rest of the model. Global rather than per space: the file is
	// per-machine and never committed. Its zero value is "all defaults" — today's
	// look — so a machine with no file rides an empty struct. Any parse warnings
	// surface on the spaces beside the agent-library warnings, not here.
	Terminal TerminalPrefs `json:"terminal"`
	// Notify is the operator's resolved machine-wide run notification rule from
	// `notify.toml`. Durations ride as Go duration strings so the read-only config
	// surface shows the same units the file accepts rather than nanoseconds.
	Notify NotifyPrefs `json:"notify"`
	// NativePicker is whether this machine can raise an OS folder chooser for
	// "add a space" — always true on macOS, true on Linux with zenity or kdialog
	// installed, false otherwise. It is capability, not state, so it never
	// changes over a run; it rides the snapshot because the frontend has no other
	// way to know whether to raise a chooser or fall back to asking the operator
	// to paste a path.
	NativePicker bool `json:"nativePicker"`
}

// ConfigLayer is one file or directory a space's effective config resolves
// through, named so the operator can open it. Legibility is the whole point
// (story 36): every value the surface shows names the layer it came from, and
// every layer names where on disk it lives.
//
// Name is the server-side token the open action resolves — the client never
// sends a path, only one of these names (ADR 0014).
type ConfigLayer struct {
	Name string `json:"name"`
	// Layer is which of the three layers this file is (built-in, workspace, user),
	// matching the provenance badges on the values it can set.
	Layer string `json:"layer"`
	// Holds names what this layer can set: "agents" (the operator's agent library),
	// "skills", "terminal" (terminal customization), or "notifications" (the
	// machine-wide run clock). Each lives in its own file and the surface shows
	// that split rather than implying one file.
	Holds string `json:"holds"`
	// Path is the absolute location on disk, and Exists whether anything is there
	// yet. A layer that does not exist is still listed: it is where the value
	// *would* go.
	Path   string `json:"path"`
	Exists bool   `json:"exists"`
}

// NotifyPrefs is the resolved notify.toml value on the wire. Unlike terminal
// customization these are complete values, never tri-state: absent or invalid
// keys have already become the shipped defaults.
type NotifyPrefs struct {
	After   string `json:"after"`
	Settle  string `json:"settle"`
	Enabled bool   `json:"enabled"`
}

// Space is one registry entry in the pushed model: normally a registered git
// repository chartr drives, or the one flagged synthetic Scratch entry. Spaces
// arrive already ordered — in the operator's own stored order, which nothing but
// an explicit reorder moves — so the sidebar renders them without re-sorting.
type Space struct {
	ID   string `json:"id"`
	Name string `json:"name"`
	// Scratch marks the one synthetic home for ad-hoc shells. Registered spaces
	// omit it; consumers that offer repository actions use it as their guard.
	Scratch bool `json:"scratch,omitempty"`
	// Path is the absolute working directory. For a registered space it is the
	// working-tree root; for Scratch it is the operator's home directory.
	Path string `json:"path"`
	// Branch is the working tree's current git branch — the checked-out ref's
	// short name, or a short sha for a detached HEAD — read live on each rebuild.
	// Empty when it can't be determined; the sidebar simply omits it then. A
	// label, never a guarantee.
	Branch string `json:"branch,omitempty"`
	// Dirty is true when the working tree carries uncommitted changes — modified,
	// staged, or untracked files a session or an ad-hoc shell left behind. It is a
	// badge, never a spawn gate (spec, Git and the gate; story 68): the operator
	// decides whether the debris is harmless, and chartr spawns into it all
	// the same. A label, not a guarantee — empty on a tree it cannot read.
	Dirty bool `json:"dirty"`
	// LastAgent is the registered agent this space last spawned with — the
	// remembered choice the next spawn reuses (stories 12, 13, 20). It is state,
	// not config: nothing edits it, and it is reported exactly as the registry
	// holds it. A name that no longer names a registered agent reads as *nothing
	// remembered* on the client, which is what re-opens the picker rather than
	// substituting something (story 19). Empty until the first spawn.
	LastAgent string `json:"lastAgent,omitempty"`
	// Layers are this space's own config files, each with its path. Nothing puts a
	// layer here since the committed skill library went with the layer model
	// (ADR 0017); the layers a space shares with every other one live on
	// Model.Config. Never nil on the wire.
	Layers []ConfigLayer `json:"layers"`
	// Maps are the space's discovered wayfinder maps (ticket 03), derived live
	// from `.plan/` and re-pushed whenever the filesystem watch notices a change.
	// Ordered for the sidebar: finished maps sort last. Never nil on the wire.
	Maps []Map `json:"maps"`
	// Terminals are the space's open ad-hoc shells (ticket 05) in the order the
	// operator opened them — the tabs of the terminal column. They are chartr-
	// owned runtime state, not derived from disk: deliberately *not* sessions
	// (no ticket, no lifecycle, ended by the human), so a mapless space is still
	// usable as a plain multiplexer. Never nil on the wire.
	Terminals []Terminal `json:"terminals"`
	// Warnings are non-fatal notices surfaced against the space — an unknown
	// role in config, a malformed config file. Surface, never enforce.
	Warnings []string `json:"warnings,omitempty"`
}

// Map is one discovered wayfinder map beneath a space: its body material and its
// tickets with their derived status. It is read from the one fixed root chartr's
// conventions declare — `.plan/maps/<slug>/` — and rendered as-is: a malformed
// map is never refused, only surfaced through Malformations (story 17). A discovered map is live: it opens and offers session actions the moment
// it is found, with no classification step in between.
type Map struct {
	// Slug is the map directory's name — its stable identity within the space.
	Slug string `json:"slug"`
	// Name is the map's H1 title; Slug stands in when the body has none.
	Name string `json:"name"`
	// Dir is the absolute path of the map directory (the one holding map.md).
	Dir string `json:"dir"`
	// Destination is the map's stated destination, shown when the map material
	// pane opens (ticket 07). Empty on a map that omits it — surfaced, not refused.
	Destination string `json:"destination"`
	// Body is the map's markdown below its H1 title — Destination, Notes,
	// Decisions, Out of scope, and fog. Inlined so the map-material pane (ticket
	// 07) opens from the title with no second fetch. Empty on a bodyless map.
	Body string `json:"body,omitempty"`
	// Tickets are the map's tickets in number order, each with its derived
	// status and stricter-frontier membership.
	Tickets []Ticket `json:"tickets"`
	// Finished is true when the map has tickets and every one of them is closed
	// (resolved or ruled out); finished maps sort last in the sidebar.
	Finished bool `json:"finished"`
	// Malformations are the map's surfaced defects — an unparseable ticket, a
	// dangling blocked_by, a drifted index — each rendered where it bites and
	// never a reason to refuse the map (story 17).
	Malformations []string `json:"malformations,omitempty"`
}

// Ticket is one ticket's derived state on the wire: its identity, type, the
// status derived from its file (open, claimed, resolved, out_of_scope — ADR
// 0004), its blockers, and whether it sits on the frontier (open, every blocker
// resolved).
type Ticket struct {
	Num       int    `json:"num"`
	Slug      string `json:"slug"`
	Title     string `json:"title"`
	Type      string `json:"type"`
	Status    string `json:"status"`
	BlockedBy []int  `json:"blockedBy,omitempty"`
	// Frontier is membership in the frontier — the takeable edge: open, with
	// every blocker resolved.
	Frontier bool `json:"frontier"`
	// ClaimedBy and ClaimedAt are the claim the ticket file carries — the session
	// id stamped by the claim commit and when. Both are empty for every status but
	// `claimed`, and both are read straight off the frontmatter, so a claim written
	// on another machine (or by a chartr that has since restarted) travels exactly
	// as a local one does. They are what lets the frontier show *who* holds a
	// claimed ticket and how long they have held it, and what the ticket-level
	// release names in its commit.
	ClaimedBy string `json:"claimedBy,omitempty"`
	ClaimedAt string `json:"claimedAt,omitempty"`
	// Body is the ticket's markdown below its H1 title — Question and Done-when,
	// and any closing answer. Inlined so the detail pane (ticket 07) reads the
	// full ticket, and a blocker's answer, straight from the snapshot.
	Body string `json:"body,omitempty"`
}

// Terminal is one open ad-hoc shell on the wire: its identity, a tab label, the
// process currently in its foreground and that shell's activity, and whether its
// process is still alive. It is not a session — it carries no ticket and no
// lifecycle. Its raw bytes travel on the separate terminal socket keyed by ID,
// never in this snapshot (ADR 0010).
type Terminal struct {
	ID    string `json:"id"`
	Title string `json:"title"`
	// Proc is the process currently in the foreground of the shell's PTY — the
	// shell itself while it sits at its prompt, or the command it is running (an
	// agent, an editor). Falls back to Title where the platform can't report it.
	Proc string `json:"proc"`
	// Status is the tab's live activity. A tab with no known agent in its foreground
	// reads TerminalIdle at the prompt, TerminalWorking while a command runs, and
	// TerminalExited once the process is gone. A tab with a known agent reads the
	// agent's own broadcast state instead — TerminalIdle / TerminalWorking /
	// TerminalBlocked — and a session whose process died reads TerminalDead. It
	// drives the sidebar's per-tab status indicator.
	Status string `json:"status"`
	// Alive is false the instant the process exits. A dead ad-hoc shell drops from
	// the model; a dead session stays pinned (Alive false, Status TerminalDead) so
	// the operator can resume, respawn, or release it.
	Alive bool `json:"alive"`
	// Session is set only when this tab is a session — a PTY running an agent
	// against exactly one ticket (ticket 09). It carries the binding the tab
	// renders: the map and ticket it is claimed on, the role it was spawned as,
	// and the resolved agent and model. Absent on an ad-hoc shell, which is
	// deliberately not a session; the chrome tells the two apart by its presence.
	Session *Session `json:"session,omitempty"`
	// FinishedUnseen records that this tab finished a run long enough to be worth
	// interrupting the operator over — the same event the OS notification carries —
	// and that they have not looked at it since. It is set when the run clock emits
	// and cleared when the operator focuses the tab, which is the only
	// acknowledgement there is: no manual dismiss, no clear-all, no count.
	//
	// It lives here, in the snapshot, rather than in the browser, because the run
	// most worth recording is the one that ended with no browser attached at all. A
	// client-side flag would show nothing in exactly that case; this one survives a
	// reload because the server never forgot it.
	FinishedUnseen bool `json:"finishedUnseen,omitempty"`
}

// TerminalPrefs is the operator's resolved terminal customization on the wire —
// the mirror of config.TerminalPrefs the server converts once it has resolved the
// per-machine `terminal.toml`. Every field is a pref the file set; a field left at
// its zero value is *unset*, and the client resolve seam (tokens.ts) falls it
// through to the app default — a colour to the live design token, the font to the
// bundled family. Ticket 01 carried the spine (font family, size, the two base
// colours); ticket 02 widened it to a named theme preset plus the full slot set
// (all sixteen ANSI slots and the five base slots), which the client layers as
// tokens → preset → explicit slots; ticket 03 adds the remaining pass-through
// options — font weight/line-height/letter-spacing, the cursor, scrolling, a
// minimum-contrast floor, and the unicode11 glyph-width toggle; ticket 04 adds the
// scrollbar and padding (CSS custom properties at the seam, not xterm options) and
// the keybinding/selection behaviours; ticket 05 adds the ligatures toggle, which
// the client resolves into the renderer choice (WebGL default vs. canvas).
type TerminalPrefs struct {
	FontFamily     string  `json:"fontFamily,omitempty"`
	FontSize       float64 `json:"fontSize,omitempty"`
	FontWeight     string  `json:"fontWeight,omitempty"`
	FontWeightBold string  `json:"fontWeightBold,omitempty"`
	LineHeight     float64 `json:"lineHeight,omitempty"`
	LetterSpacing  float64 `json:"letterSpacing,omitempty"`

	// Ligatures forces this terminal onto the canvas renderer (WebGL off) when on,
	// and the client suppresses it for a non-bundled font. Unset (nil) is off.
	Ligatures *bool `json:"ligatures,omitempty"`

	CursorStyle         string  `json:"cursorStyle,omitempty"`
	CursorBlink         *bool   `json:"cursorBlink,omitempty"`
	CursorInactiveStyle string  `json:"cursorInactiveStyle,omitempty"`
	CursorWidth         float64 `json:"cursorWidth,omitempty"`

	Scrollback            float64 `json:"scrollback,omitempty"`
	ScrollSensitivity     float64 `json:"scrollSensitivity,omitempty"`
	FastScrollModifier    string  `json:"fastScrollModifier,omitempty"`
	FastScrollSensitivity float64 `json:"fastScrollSensitivity,omitempty"`
	SmoothScrollDuration  float64 `json:"smoothScrollDuration,omitempty"`

	MinimumContrastRatio float64 `json:"minimumContrastRatio,omitempty"`

	Unicode11 *bool `json:"unicode11,omitempty"`

	// The scrollbar and the padding are CSS, not xterm options: the client resolve
	// seam turns them into custom properties on the island host (xterm exposes no
	// options for either), and a padding change is followed by a refit.
	ScrollbarWidth    float64 `json:"scrollbarWidth,omitempty"`
	ScrollbarThumb    string  `json:"scrollbarThumb,omitempty"`
	ScrollbarTrack    string  `json:"scrollbarTrack,omitempty"`
	ScrollbarAutoHide *bool   `json:"scrollbarAutoHide,omitempty"`

	PaddingTop    float64 `json:"paddingTop,omitempty"`
	PaddingRight  float64 `json:"paddingRight,omitempty"`
	PaddingBottom float64 `json:"paddingBottom,omitempty"`
	PaddingLeft   float64 `json:"paddingLeft,omitempty"`

	// The keybinding and selection behaviours. ShiftEnterNewline is unset-means-on
	// (the Ghostty-style newline is the default the cockpit ships); the other three
	// are unset-means-off, matching xterm's own defaults.
	ShiftEnterNewline     *bool `json:"shiftEnterNewline,omitempty"`
	CopyOnSelect          *bool `json:"copyOnSelect,omitempty"`
	RightClickSelectsWord *bool `json:"rightClickSelectsWord,omitempty"`
	MacOptionIsMeta       *bool `json:"macOptionIsMeta,omitempty"`

	Preset string `json:"preset,omitempty"`

	Background   string `json:"background,omitempty"`
	Foreground   string `json:"foreground,omitempty"`
	Cursor       string `json:"cursor,omitempty"`
	CursorAccent string `json:"cursorAccent,omitempty"`
	Selection    string `json:"selection,omitempty"`

	Black   string `json:"black,omitempty"`
	Red     string `json:"red,omitempty"`
	Green   string `json:"green,omitempty"`
	Yellow  string `json:"yellow,omitempty"`
	Blue    string `json:"blue,omitempty"`
	Magenta string `json:"magenta,omitempty"`
	Cyan    string `json:"cyan,omitempty"`
	White   string `json:"white,omitempty"`

	BrightBlack   string `json:"brightBlack,omitempty"`
	BrightRed     string `json:"brightRed,omitempty"`
	BrightGreen   string `json:"brightGreen,omitempty"`
	BrightYellow  string `json:"brightYellow,omitempty"`
	BrightBlue    string `json:"brightBlue,omitempty"`
	BrightMagenta string `json:"brightMagenta,omitempty"`
	BrightCyan    string `json:"brightCyan,omitempty"`
	BrightWhite   string `json:"brightWhite,omitempty"`
}

// Session is a session tab's ticket binding on the wire — enough for the sidebar
// to render a session row as bound to its ticket and driven by its agent, without
// the PTY. The session↔ticket invariant lives here: exactly one ticket per
// session, named by its map slug and number.
type Session struct {
	MapSlug   string `json:"mapSlug"`
	TicketNum int    `json:"ticketNum"`
	Role      string `json:"role"`
	Agent     string `json:"agent"`
}

// A terminal's activity states, uniform across the wire and the sidebar's status
// indicator. Which grammar a tab reads on is decided by what holds its PTY's
// foreground, not by whether it is a session.
//
// A tab with no known agent in the foreground reads the shell grammar: idle at
// the prompt (a tick), working while a foreground command runs (a spinner),
// exited once the process is gone.
//
// A tab with a known agent reads the agent grammar, from the evidence the agent
// broadcasts about itself: idle when it is present but not generating, working
// while it is, and blocked when it has stopped on a permission prompt and is
// waiting on its human — the state worth an operator's attention. `dead` is
// unchanged and belongs to sessions: a session whose process exits freezes in
// place rather than vanishing, pinned to its ticket until the operator resumes,
// respawns, or releases it.
const (
	TerminalIdle    = "idle"
	TerminalWorking = "working"
	TerminalExited  = "exited"
	TerminalBlocked = "blocked"
	TerminalDead    = "dead"
)

// Agent is one entry of the operator's registered agent library on the wire: a
// named, complete way to run a harness — the binary, whatever flags that harness
// wants (its model among them), and how it takes its opening prompt. The library is
// global rather than per space, so it hangs off the model rather than off a
// space, and roles across every space assign to it by name.
type Agent struct {
	Name    string   `json:"name"`
	Adapter string   `json:"adapter"`
	Args    []string `json:"args,omitempty"`
	// Env is the environment this agent launches with, as `KEY=VALUE` entries
	// exactly as the operator typed them — tilde and all. It is what the editing
	// surface round trips; the *expanded* values appear in Command, which is where
	// "what will actually run" is answered.
	Env    []string `json:"env,omitempty"`
	Prompt string   `json:"prompt,omitempty"`
	// Delivery is what Prompt actually resolves to once the adapter's own default
	// is taken into account — `argv`, `type`, or a flag name. Resolved server-side
	// so the surface renders how a harness is told what to do as a fact, rather
	// than re-deriving the adapter table in the browser and drifting from it.
	Delivery string `json:"delivery"`
	// Command is the command line this agent would actually launch, with a
	// placeholder standing in for the opener: the environment first, in the shell's
	// own `KEY=VALUE binary args` order, then the argv built by the same seam that
	// builds the real one. So what the operator reads in the library is what will
	// run — including the expansion, which is the only way to see it happened.
	Command []string `json:"command"`
	// Present is whether the adapter binary was found on PATH; Missing is the
	// absence badge when it was not.
	Present bool   `json:"present"`
	Missing string `json:"missing,omitempty"`
}

// Empty returns a well-formed, near-empty model: no spaces and no config layers,
// but non-nil slices so the JSON snapshot is always well-formed arrays rather
// than nulls.
func Empty() Model {
	return Model{Spaces: []Space{}, Config: []ConfigLayer{}, Agents: []Agent{}, Detected: []string{}}
}
