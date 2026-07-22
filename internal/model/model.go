// Package model holds the derived model the harness pushes to every browser.
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
	// Holds names what this layer can set: "bindings" (role bindings and, in the
	// committed layer, map kinds) or "skills". The two halves live in different
	// files — bindings in the harness state root, skills under the operator's
	// config root — and the surface shows that split rather than implying one file.
	Holds string `json:"holds"`
	// Path is the absolute location on disk, and Exists whether anything is there
	// yet. A layer that does not exist is still listed: it is where the value
	// *would* go.
	Path   string `json:"path"`
	Exists bool   `json:"exists"`
}

// ResolvedSkill is one skill of the library as it resolves for a space: which
// layer won its whole directory (whole-skill shadowing), and whether a fork has
// fallen behind the shipped default. It is the positive statement of resolution —
// "your grill resolves from: user" — not just the warning (story 34).
type ResolvedSkill struct {
	Name  string `json:"name"`
	Layer string `json:"layer"`
	// Dir is where the winning directory sits, or empty when no layer defines it
	// on disk and the copy embedded in the binary is the floor.
	Dir         string `json:"dir,omitempty"`
	Description string `json:"description,omitempty"`
	// ForkedFrom is the shipped content hash a shadowing skill recorded in its
	// frontmatter; Stale is true once the shipped default has moved past it. A
	// stale fork is surfaced, never auto-merged.
	ForkedFrom string `json:"forkedFrom,omitempty"`
	Stale      bool   `json:"stale,omitempty"`
}

// Space is a registered git repository the harness drives. Ticket 02 fills in
// the registry semantics (path, pin) and the effective role bindings; ticket 03
// adds the maps beneath. Spaces arrive already ordered — pinned first, then by
// recency — so the sidebar renders them in slice order without re-sorting.
type Space struct {
	ID   string `json:"id"`
	Name string `json:"name"`
	// Path is the absolute working-tree root, shown in the UI and the stable
	// thing a local binding override is keyed by.
	Path string `json:"path"`
	// Branch is the working tree's current git branch — the checked-out ref's
	// short name, or a short sha for a detached HEAD — read live on each rebuild.
	// Empty when it can't be determined; the sidebar simply omits it then. A
	// label, never a guarantee.
	Branch string `json:"branch,omitempty"`
	// Pinned spaces sort first; the flag is local, per-machine registry state.
	Pinned bool `json:"pinned"`
	// Dirty is true when the working tree carries uncommitted changes — modified,
	// staged, or untracked files a session or an ad-hoc shell left behind. It is a
	// badge, never a spawn gate (spec, Git and the gate; story 68): the operator
	// decides whether the debris is harmless, and the harness spawns into it all
	// the same. A label, not a guarantee — empty on a tree it cannot read.
	Dirty bool `json:"dirty"`
	// Bindings are the space's effective, fully-resolved role bindings in role
	// order, each carrying per-field provenance and PATH presence so the
	// operator sees what will actually run (stories 39, 40).
	Bindings []RoleBinding `json:"bindings"`
	// Skills are the space's resolved skill library — every skill with the layer
	// that won its whole directory and its stale-fork state (ticket 05). Derived
	// beside Bindings so the settings route reads content provenance and execution
	// provenance out of the same push. Never nil on the wire.
	Skills []ResolvedSkill `json:"skills"`
	// Layers are this space's own config files — its committed workspace config
	// and committed skill library — each with its path. The layers it shares with
	// every other space live on Model.Config.
	Layers []ConfigLayer `json:"layers"`
	// Maps are the space's discovered wayfinder maps (ticket 03), derived live
	// from `.plan/` and re-pushed whenever the filesystem watch notices a change.
	// Ordered for the sidebar: finished maps sort last. Never nil on the wire.
	Maps []Map `json:"maps"`
	// Terminals are the space's open ad-hoc shells (ticket 05) in the order the
	// operator opened them — the tabs of the terminal column. They are harness-
	// owned runtime state, not derived from disk: deliberately *not* sessions
	// (no ticket, no lifecycle, ended by the human), so a mapless space is still
	// usable as a plain multiplexer. Never nil on the wire.
	Terminals []Terminal `json:"terminals"`
	// Warnings are non-fatal notices surfaced against the space — an unknown
	// role in config, an unrecognised map kind, a malformed config file.
	// Surface, never enforce.
	Warnings []string `json:"warnings,omitempty"`
}

// Kind is a map's declared character — planning tickets resolve decisions, an
// implementation map's deliver code against a settled spec, and the kind selects
// which roles a session may be spawned as (ADR 0007). It is a property
// of the map, uniform across its tickets, declared in committed workspace config
// and never inferred at read time. The empty string is the third state: an
// undeclared map, inert until a human classifies it.
const (
	KindUnclassified   = ""
	KindPlanning       = "planning"
	KindImplementation = "implementation"
)

// ValidKind reports whether k is a declarable kind — the two the operator may
// confirm. The unclassified empty string is a derived state, not something
// config declares, so it is deliberately not valid here.
func ValidKind(k string) bool {
	return k == KindPlanning || k == KindImplementation
}

// Map is one discovered wayfinder map beneath a space: its body material and its
// tickets with their derived status. It is read wherever wayfinder writes — the
// `.plan/<slug>/` layout or the `.plan/maps/<slug>/` one — and rendered as-is: a
// malformed map is never refused, only surfaced through Malformations (story
// 17). A map's declared Kind gates its session actions: until a human classifies
// it (ADR 0007), Kind is unclassified and the map is inert — readable, rendered,
// but offering no session actions.
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
	// Kind is the map's declared lifecycle (KindPlanning, KindImplementation) or
	// KindUnclassified when no committed declaration matches this slug. An
	// unclassified map is inert: the harness offers no session actions on it until
	// a human classifies it (ADR 0007, story 13).
	Kind string `json:"kind"`
	// KindGuess is the convention-derived guess the classify affordance pre-fills
	// for a one-keystroke confirm (story 14) — the `-impl` suffix and all-`task`
	// tickets. Set only while the map is unclassified; blank once Kind is
	// declared, so the guess never lingers as gospel.
	KindGuess string `json:"kindGuess,omitempty"`
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
	// Status is the tab's live activity. An ad-hoc shell reads TerminalIdle at the
	// prompt, TerminalWorking while a foreground command runs, TerminalExited once
	// the process is gone. A session tab reads the session grammar instead
	// (TerminalWorking / TerminalQuiet / TerminalDead — ticket 10). It drives the
	// sidebar's per-tab status indicator.
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
	Model     string `json:"model"`
}

// A terminal's activity states, uniform across the wire and the sidebar's status
// indicator. An ad-hoc shell is idle at the prompt (a tick), working while a
// foreground command runs (a spinner), or exited once the shell is gone. A session
// tab reads on the session grammar instead (ticket 10): working while it is live
// and producing, quiet when an AFK session has fallen silent past the threshold
// (a hint, never an alarm), and dead once its process exits — a dead session
// freezes in place rather than vanishing, pinned to its ticket until the
// operator resumes, respawns, or releases it.
const (
	TerminalIdle    = "idle"
	TerminalWorking = "working"
	TerminalExited  = "exited"
	TerminalQuiet   = "quiet"
	TerminalDead    = "dead"
)

// RoleBinding is one role's effective binding on the wire: which adapter runs on
// which model with which args, where each field was inherited from, and whether
// the adapter's binary is actually present on the operator's PATH.
type RoleBinding struct {
	Role        string   `json:"role"`
	Adapter     string   `json:"adapter"`
	Model       string   `json:"model"`
	Args        []string `json:"args,omitempty"`
	AdapterFrom string   `json:"adapterFrom"`
	ModelFrom   string   `json:"modelFrom"`
	ArgsFrom    string   `json:"argsFrom"`
	// Present is whether the adapter binary was found on PATH; when false,
	// Missing is the absence badge naming the binding, its source, and the fix.
	Present bool   `json:"present"`
	Missing string `json:"missing,omitempty"`
}

// Empty returns a well-formed, near-empty model: no spaces and no config layers,
// but non-nil slices so the JSON snapshot is always well-formed arrays rather
// than nulls.
func Empty() Model {
	return Model{Spaces: []Space{}, Config: []ConfigLayer{}}
}
