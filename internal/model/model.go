// Package model holds the derived model the harness pushes to every browser.
//
// The whole model travels over the control socket as a single JSON snapshot on
// every change (ADR 0010): it is server-authoritative, small enough that
// diffing buys nothing, and re-sent wholesale on reconnect. The walking
// skeleton ships it near-empty — the point of this slice is the transport, and
// every later ticket hangs its state (spaces, maps, tickets, sessions, review)
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
	// Pinned spaces sort first; the flag is local, per-machine registry state.
	Pinned bool `json:"pinned"`
	// Bindings are the space's effective, fully-resolved role bindings in role
	// order, each carrying per-field provenance and PATH presence so the
	// operator sees what will actually run (stories 39, 40).
	Bindings []RoleBinding `json:"bindings"`
	// Warnings are non-fatal notices surfaced against the space — a committed
	// autopilot flag ignored, an unknown role in config, a malformed config
	// file. Surface, never enforce.
	Warnings []string `json:"warnings,omitempty"`
}

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

// Empty returns a well-formed, near-empty model: no spaces, but a non-nil slice
// so the JSON snapshot is always `{"spaces":[]}` rather than `{"spaces":null}`.
func Empty() Model {
	return Model{Spaces: []Space{}}
}
