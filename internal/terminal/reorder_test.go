package terminal

import (
	"errors"
	"testing"
)

// seatTerminals builds a manager whose global order holds the given (id, space)
// pairs in the given order, with no PTY behind any of them — Reorder is pure
// bookkeeping over m.order and never touches a process, so a bare Terminal is the
// honest subject here.
func seatTerminals(pairs ...[2]string) *Manager {
	m := NewManager(nil, nil)
	for _, p := range pairs {
		id, space := p[0], p[1]
		m.terms[id] = &Terminal{ID: id, SpaceID: space}
		m.order = append(m.order, id)
	}
	return m
}

// spaceIDs is the space's terminals in current global order — what ForSpace
// reports, reduced to ids for comparison.
func spaceIDs(m *Manager, space string) []string {
	out := make([]string, 0)
	for _, info := range m.ForSpace(space) {
		out = append(out, info.ID)
	}
	return out
}

func eqIDs(a, b []string) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}

// A reorder rewrites only its own space's slots, and it rewrites them into the
// posted sequence — the whole point of the scope-to-one-card rule.
func TestReorderScopedToSpaceLeavesOthersInPlace(t *testing.T) {
	// A and B interleave in the global order, so a naive rewrite that ignored the
	// space would move B's tabs too. a2 sits between a1 and a3.
	m := seatTerminals(
		[2]string{"a1", "A"},
		[2]string{"b1", "B"},
		[2]string{"a2", "A"},
		[2]string{"a3", "A"},
		[2]string{"b2", "B"},
	)

	if err := m.Reorder("A", []string{"a3", "a1", "a2"}); err != nil {
		t.Fatalf("Reorder: %v", err)
	}

	if got := spaceIDs(m, "A"); !eqIDs(got, []string{"a3", "a1", "a2"}) {
		t.Errorf("space A order = %v, want [a3 a1 a2]", got)
	}
	// B never moved, and its tabs kept the exact global slots they held between A's.
	if got := spaceIDs(m, "B"); !eqIDs(got, []string{"b1", "b2"}) {
		t.Errorf("space B order = %v, want [b1 b2] (a scoped reorder must not touch it)", got)
	}
	// The foreign slot (b1) stays wedged where it was — between what are now a3 and a2.
	if !eqIDs(m.order, []string{"a3", "b1", "a1", "a2", "b2"}) {
		t.Errorf("global order = %v, want [a3 b1 a1 a2 b2]", m.order)
	}
}

// Every non-permutation is refused whole, leaving the order untouched — the same
// all-or-nothing contract the sidebar reorder keeps.
func TestReorderRefusesNonPermutations(t *testing.T) {
	cases := map[string][]string{
		"too few":     {"a1", "a2"},
		"too many":    {"a1", "a2", "a3", "a1"},
		"foreign id":  {"a1", "a2", "b1"},
		"a duplicate": {"a1", "a1", "a3"},
	}
	for name, ids := range cases {
		t.Run(name, func(t *testing.T) {
			m := seatTerminals(
				[2]string{"a1", "A"},
				[2]string{"a2", "A"},
				[2]string{"a3", "A"},
				[2]string{"b1", "B"},
			)
			before := append([]string(nil), m.order...)

			err := m.Reorder("A", ids)
			if !errors.Is(err, ErrBadReorder) {
				t.Fatalf("Reorder(%v) error = %v, want ErrBadReorder", ids, err)
			}
			if !eqIDs(m.order, before) {
				t.Errorf("a refused reorder changed the order: %v, want %v", m.order, before)
			}
		})
	}
}

// A drop back where it started posts the same sequence, and that is a well-formed
// permutation — it must be accepted, not mistaken for an error, and leave the
// order exactly as it was.
func TestReorderIdentityIsAccepted(t *testing.T) {
	m := seatTerminals(
		[2]string{"a1", "A"},
		[2]string{"a2", "A"},
	)
	if err := m.Reorder("A", []string{"a1", "a2"}); err != nil {
		t.Fatalf("identity Reorder: %v", err)
	}
	if got := spaceIDs(m, "A"); !eqIDs(got, []string{"a1", "a2"}) {
		t.Errorf("order after identity reorder = %v, want [a1 a2]", got)
	}
}

// A space with no terminals accepts only the empty list — there is nothing to
// permute, and a stale view naming a tab that has since ended is refused rather
// than half-applied.
func TestReorderEmptySpace(t *testing.T) {
	m := seatTerminals([2]string{"b1", "B"})
	if err := m.Reorder("A", nil); err != nil {
		t.Fatalf("empty Reorder: %v", err)
	}
	if err := m.Reorder("A", []string{"gone"}); !errors.Is(err, ErrBadReorder) {
		t.Fatalf("Reorder naming a nonexistent tab = %v, want ErrBadReorder", err)
	}
}
