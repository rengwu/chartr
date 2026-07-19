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

// Space is a registered git repository the harness drives. The skeleton defines
// only the identity fields every later surface needs; ticket 02 fills in the
// registry semantics (pin, recency, role bindings) and ticket 03 the maps.
type Space struct {
	ID   string `json:"id"`
	Name string `json:"name"`
}

// Empty returns a well-formed, near-empty model: no spaces, but a non-nil slice
// so the JSON snapshot is always `{"spaces":[]}` rather than `{"spaces":null}`.
func Empty() Model {
	return Model{Spaces: []Space{}}
}
