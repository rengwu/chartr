package registry_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/BurntSushi/toml"
	"github.com/rengwu/chartr/internal/registry"
)

// The stored sidebar order at the file seam. The migration is the test that
// matters: a registry written before spaces carried an order must load to
// exactly the sequence the *old* rule produced — pinned first, then recency,
// then path — and keep it once it is written back.
//
// That sequence is now stated as a literal expectation rather than re-derived by
// an oracle. `pinned` is deleted: it is gone from registry.Entry, so a comparator
// written here can no longer read the field it sorted by, and only the migration
// (which reads the raw key at load) still knows about it. The fixture's expected
// order is spelled out beside the fixture instead, so what is being frozen is
// visible in one place.
//
// The degradation cases follow: a hand-edited or truncated file costs the
// operator their arrangement, never their list of spaces.

func paths(entries []registry.Entry) []string {
	out := make([]string, 0, len(entries))
	for _, e := range entries {
		if e.Scratch {
			continue
		}
		out = append(out, e.Path)
	}
	return out
}

// slots names the sidebar sequence with Scratch left in, which paths deliberately
// drops. Where the synthetic entry sits among the registered rows is the whole
// subject of the stored-slot tests, and it has no path of its own worth naming.
func slots(entries []registry.Entry) []string {
	out := make([]string, 0, len(entries))
	for _, e := range entries {
		if e.Scratch {
			out = append(out, "scratch")
			continue
		}
		out = append(out, e.Path)
	}
	return out
}

// idsOf is the argument a Reorder takes: the current sequence, by ID.
func idsOf(entries []registry.Entry) []string {
	out := make([]string, 0, len(entries))
	for _, e := range entries {
		out = append(out, e.ID)
	}
	return out
}

// writeRegistry lays down a spaces.toml verbatim and returns its data dir, so a
// test can describe a pre-upgrade or hand-edited file exactly as it sits on disk.
func writeRegistry(t *testing.T, body string) string {
	t.Helper()
	dir := t.TempDir()
	if err := os.WriteFile(filepath.Join(dir, "spaces.toml"), []byte(body), 0o644); err != nil {
		t.Fatalf("writing spaces.toml: %v", err)
	}
	return dir
}

func load(t *testing.T, dir string) *registry.Registry {
	t.Helper()
	r, err := registry.Load(dir)
	if err != nil {
		t.Fatalf("loading registry: %v", err)
	}
	return r
}

func equal(a, b []string) bool {
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

// storedOrders reads the orders back out of the file itself, keyed by path — the
// only way to tell a frozen order that was persisted from one merely re-derived
// on the next load.
func storedOrders(t *testing.T, dir string) map[string]int {
	t.Helper()
	data, err := os.ReadFile(filepath.Join(dir, "spaces.toml"))
	if err != nil {
		t.Fatalf("reading spaces.toml back: %v", err)
	}
	var f struct {
		Spaces []struct {
			Path  string `toml:"path"`
			Order *int   `toml:"order"`
		} `toml:"space"`
	}
	if err := toml.Unmarshal(data, &f); err != nil {
		t.Fatalf("re-parsing spaces.toml: %v", err)
	}
	out := map[string]int{}
	for _, s := range f.Spaces {
		if s.Order == nil {
			t.Errorf("entry %s carries no order after a save", s.Path)
			continue
		}
		out[s.Path] = *s.Order
	}
	return out
}

// preUpgrade is a registry as it was written before the order existed: mixed
// pinned and last_active values, no order anywhere, deliberately not in
// sidebar sequence in the file.
//
// preUpgradeOrder is what the old rule made of it, worked through by hand:
// the two pinned spaces lead, most-recently-active first among them (gamma
// 01-03 before alpha 01-01), then the unpinned two the same way (delta 01-09
// before beta 01-05).
const preUpgrade = `
[[space]]
  path = "/repos/beta"
  pinned = false
  last_active = 2026-01-05T09:00:00Z

[[space]]
  path = "/repos/alpha"
  pinned = true
  last_active = 2026-01-01T09:00:00Z

[[space]]
  path = "/repos/delta"
  pinned = false
  last_active = 2026-01-09T09:00:00Z

[[space]]
  path = "/repos/gamma"
  pinned = true
  last_active = 2026-01-03T09:00:00Z
`

var preUpgradeOrder = []string{"/repos/gamma", "/repos/alpha", "/repos/delta", "/repos/beta"}

// The upgrade is invisible: a file with no order loads to exactly the sequence
// the old rule produced — `pinned` included, which is why the migration still
// reads the deleted key — and that sequence survives the save that writes it back.
func TestMigrationFreezesTodaysOrder(t *testing.T) {
	dir := writeRegistry(t, preUpgrade)
	r := load(t, dir)

	got := paths(r.List())
	if !equal(got, preUpgradeOrder) {
		t.Fatalf("frozen order = %v, want the old rule's %v", got, preUpgradeOrder)
	}
	// Sanity: the file's own row order is not the sidebar's, so the assertion
	// above is testing a sort and not an echo.
	if equal(got, []string{"/repos/beta", "/repos/alpha", "/repos/delta", "/repos/gamma"}) {
		t.Fatal("frozen order is the file's row order; the old rule was not applied")
	}

	// Any write persists the freeze. Reloading must read the order off disk, not
	// re-derive it.
	first := r.List()[0]
	if err := r.SetTrackerDismissed(first.ID, true); err != nil {
		t.Fatalf("forcing a save: %v", err)
	}
	orders := storedOrders(t, dir)
	for i, p := range got {
		if orders[p] != i {
			t.Errorf("stored order for %s = %d, want %d", p, orders[p], i)
		}
	}

	if reloaded := paths(load(t, dir).List()); !equal(reloaded, got) {
		t.Errorf("order after save and reload = %v, want the frozen %v", reloaded, got)
	}
}

// A file that already carries an order is never re-derived: recency and pin lose
// against it, which is the whole point of storing it.
func TestStoredOrderBeatsPinAndRecency(t *testing.T) {
	dir := writeRegistry(t, `
[[space]]
  path = "/repos/alpha"
  order = 2
  pinned = true
  last_active = 2026-01-09T09:00:00Z

[[space]]
  path = "/repos/beta"
  order = 0
  pinned = false
  last_active = 2026-01-01T09:00:00Z

[[space]]
  path = "/repos/gamma"
  order = 1
  pinned = false
  last_active = 2026-01-05T09:00:00Z
`)
	got := paths(load(t, dir).List())
	want := []string{"/repos/beta", "/repos/gamma", "/repos/alpha"}
	if !equal(got, want) {
		t.Errorf("order = %v, want the stored %v", got, want)
	}
}

// `pinned` is deleted, and a file that still carries it is not an error: the key
// is ignored on read — every space present, in its stored order, the flag moving
// nothing — and stops being written on the next save. An operator who upgrades
// never has to touch the file, and never sees a warning about a key they cannot
// act on.
func TestPinnedKeyIsIgnoredAndDroppedOnSave(t *testing.T) {
	dir := writeRegistry(t, `
[[space]]
  path = "/repos/alpha"
  order = 0
  pinned = false

[[space]]
  path = "/repos/beta"
  order = 1
  pinned = true

[[space]]
  path = "/repos/gamma"
  order = 2
  pinned = true
`)
	r := load(t, dir)
	want := []string{"/repos/alpha", "/repos/beta", "/repos/gamma"}
	if got := paths(r.List()); !equal(got, want) {
		t.Fatalf("order = %v, want the stored %v — a stale pinned key must move nothing", got, want)
	}

	if err := r.SetTrackerDismissed(r.List()[0].ID, true); err != nil {
		t.Fatalf("forcing a save: %v", err)
	}
	data, err := os.ReadFile(filepath.Join(dir, "spaces.toml"))
	if err != nil {
		t.Fatalf("reading spaces.toml back: %v", err)
	}
	if strings.Contains(string(data), "pinned") {
		t.Errorf("the saved registry still writes pinned:\n%s", data)
	}
	if got := paths(load(t, dir).List()); !equal(got, want) {
		t.Errorf("order after dropping the key = %v, want %v", got, want)
	}
}

// Gaps are legal input and cost nothing: the sequence is the stored one, and the
// next save closes the holes.
func TestGappedOrdersKeepTheirSequenceAndDensify(t *testing.T) {
	dir := writeRegistry(t, `
[[space]]
  path = "/repos/alpha"
  order = 9

[[space]]
  path = "/repos/beta"
  order = 0

[[space]]
  path = "/repos/gamma"
  order = 5
`)
	r := load(t, dir)
	want := []string{"/repos/beta", "/repos/gamma", "/repos/alpha"}
	if got := paths(r.List()); !equal(got, want) {
		t.Fatalf("order = %v, want %v", got, want)
	}

	if err := r.SetTrackerDismissed(r.List()[0].ID, true); err != nil {
		t.Fatalf("forcing a save: %v", err)
	}
	orders := storedOrders(t, dir)
	for i, p := range want {
		if orders[p] != i {
			t.Errorf("densified order for %s = %d, want %d", p, orders[p], i)
		}
	}
}

// Duplicate orders are malformed: every entry sharing an index is sorted among
// the malformed by the old rule and appended after the well-formed ones. No
// space is lost, and none appears twice.
func TestDuplicateOrdersDegradeAndLoseNothing(t *testing.T) {
	dir := writeRegistry(t, `
[[space]]
  path = "/repos/alpha"
  order = 1
  last_active = 2026-01-01T09:00:00Z

[[space]]
  path = "/repos/beta"
  order = 0
  last_active = 2026-01-02T09:00:00Z

[[space]]
  path = "/repos/gamma"
  order = 1
  pinned = true
  last_active = 2026-01-03T09:00:00Z
`)
	r := load(t, dir)
	// beta alone carries a unique order and leads. alpha and gamma both claim 1,
	// so they fall to the back ordered by the old rule — gamma is pinned, so it
	// leads alpha there.
	want := []string{"/repos/beta", "/repos/gamma", "/repos/alpha"}
	if got := paths(r.List()); !equal(got, want) {
		t.Errorf("degraded order = %v, want %v", got, want)
	}
	assertTotalOrder(t, r, 3)
}

// An order on some entries only is the half-migrated file: the ordered ones keep
// their sequence and the rest append behind them by the old rule.
func TestPartialOrdersKeepTheOrderedAndAppendTheRest(t *testing.T) {
	dir := writeRegistry(t, `
[[space]]
  path = "/repos/alpha"
  last_active = 2026-01-01T09:00:00Z

[[space]]
  path = "/repos/beta"
  order = 1
  last_active = 2026-01-02T09:00:00Z

[[space]]
  path = "/repos/gamma"
  order = 0
  last_active = 2026-01-03T09:00:00Z

[[space]]
  path = "/repos/delta"
  pinned = true
  last_active = 2026-01-04T09:00:00Z
`)
	r := load(t, dir)
	want := []string{"/repos/gamma", "/repos/beta", "/repos/delta", "/repos/alpha"}
	if got := paths(r.List()); !equal(got, want) {
		t.Errorf("degraded order = %v, want %v", got, want)
	}
	assertTotalOrder(t, r, 4)
}

// An order of 0 on a single entry is a real order, not a missing one: a file
// where one entry carries `order = 0` and the rest carry nothing must treat that
// entry as well-formed and append the others behind it.
func TestZeroIsAnOrderNotAnAbsence(t *testing.T) {
	dir := writeRegistry(t, `
[[space]]
  path = "/repos/alpha"
  last_active = 2026-01-09T09:00:00Z

[[space]]
  path = "/repos/beta"
  order = 0
  last_active = 2026-01-01T09:00:00Z
`)
	want := []string{"/repos/beta", "/repos/alpha"}
	if got := paths(load(t, dir).List()); !equal(got, want) {
		t.Errorf("order = %v, want %v — order = 0 must beat the more recent unordered entry", got, want)
	}
}

// assertTotalOrder is the degradation contract: however mangled the file, every
// space appears exactly once and the sequence is dense after the next save.
func assertTotalOrder(t *testing.T, r *registry.Registry, want int) {
	t.Helper()
	list := make([]registry.Entry, 0, want)
	for _, e := range r.List() {
		if !e.Scratch {
			list = append(list, e)
		}
	}
	if len(list) != want {
		t.Fatalf("listed %d spaces, want %d — degradation lost or duplicated one", len(list), want)
	}
	seen := map[string]bool{}
	for i, e := range list {
		if seen[e.Path] {
			t.Errorf("space %s listed twice", e.Path)
		}
		seen[e.Path] = true
		if e.Order != i {
			t.Errorf("entry %s at index %d carries order %d; the list is not densified", e.Path, i, e.Order)
		}
	}
}

// A missing file is still the first-run state: no registered rows, plus the one
// synthetic Scratch entry held in memory.
func TestMissingFileLoadsEmpty(t *testing.T) {
	r := load(t, t.TempDir())
	if got := paths(r.List()); len(got) != 0 {
		t.Errorf("fresh registry lists registered spaces %v, want none", got)
	}
	if scratch, ok := r.Get(registry.ScratchID); !ok || !scratch.Scratch {
		t.Errorf("fresh registry has no flagged Scratch entry: entry=%+v present=%v", scratch, ok)
	}
}

// The Scratch slot at the file seam. It is the one thing about the synthetic
// entry that is persisted, and it persists as a scalar beside the rows rather
// than as a row: a [[space]] row goes on meaning "a folder the operator
// registered", which is what keeps deleting this file and re-adding those
// folders a complete recovery.

const threeRegistered = `
[[space]]
  path = "/repos/alpha"
  order = 0

[[space]]
  path = "/repos/beta"
  order = 1

[[space]]
  path = "/repos/gamma"
  order = 2
`

// Drag Scratch into the middle and it stays there across a save and a reload.
// The file records the move as one scalar, leaves the registered rows carrying
// the gapped orders densification gave them, and still writes no row for Scratch.
func TestScratchSlotRoundTripsThroughSaveAndReload(t *testing.T) {
	dir := writeRegistry(t, threeRegistered)
	r := load(t, dir)

	if got := slots(r.List()); !equal(got, []string{"/repos/alpha", "/repos/beta", "/repos/gamma", "scratch"}) {
		t.Fatalf("a file with no recorded slot loads as %v, want Scratch appended last", got)
	}

	seq := idsOf(r.List())
	if err := r.Reorder([]string{seq[0], seq[3], seq[1], seq[2]}); err != nil {
		t.Fatalf("seating Scratch second: %v", err)
	}
	want := []string{"/repos/alpha", "scratch", "/repos/beta", "/repos/gamma"}
	if got := slots(r.List()); !equal(got, want) {
		t.Fatalf("order after the move = %v, want %v", got, want)
	}

	data, err := os.ReadFile(filepath.Join(dir, "spaces.toml"))
	if err != nil {
		t.Fatalf("reading spaces.toml back: %v", err)
	}
	if !strings.Contains(string(data), "scratch_order = 1") {
		t.Errorf("the saved registry does not record Scratch's slot:\n%s", data)
	}
	if got := strings.Count(string(data), "[[space]]"); got != 3 {
		t.Errorf("the saved registry carries %d rows, want only the 3 registered:\n%s", got, data)
	}
	// Densification ran over the whole arrangement, so the rows are written with a
	// hole at index 1 where Scratch sits. That hole is the contract the load path
	// has to put back, not an accident to be tidied away.
	orders := storedOrders(t, dir)
	for p, i := range map[string]int{"/repos/alpha": 0, "/repos/beta": 2, "/repos/gamma": 3} {
		if orders[p] != i {
			t.Errorf("stored order for %s = %d, want %d — the row orders must leave Scratch's gap", p, orders[p], i)
		}
	}

	if got := slots(load(t, dir).List()); !equal(got, want) {
		t.Errorf("order after save and reload = %v, want the stored %v", got, want)
	}
}

// The gap is legal input on its own terms: a file whose registered rows skip the
// index Scratch sat at loads back into exactly that sequence, without the
// existing compaction disturbing what surrounds it.
func TestGappedRowsLoadScratchBackIntoItsSlot(t *testing.T) {
	dir := writeRegistry(t, `
scratch_order = 1

[[space]]
  path = "/repos/alpha"
  order = 0

[[space]]
  path = "/repos/beta"
  order = 2

[[space]]
  path = "/repos/gamma"
  order = 3
`)
	want := []string{"/repos/alpha", "scratch", "/repos/beta", "/repos/gamma"}
	if got := slots(load(t, dir).List()); !equal(got, want) {
		t.Errorf("order = %v, want %v", got, want)
	}
}

// A slot of 0 is a real slot, not a missing one — the same distinction the rows'
// own order draws, and the reason the recorded index is read through a pointer.
func TestScratchSlotZeroIsASlotNotAnAbsence(t *testing.T) {
	dir := writeRegistry(t, `
scratch_order = 0

[[space]]
  path = "/repos/alpha"
  order = 1

[[space]]
  path = "/repos/beta"
  order = 2
`)
	want := []string{"scratch", "/repos/alpha", "/repos/beta"}
	if got := slots(load(t, dir).List()); !equal(got, want) {
		t.Errorf("order = %v, want %v — scratch_order = 0 must seat it first", got, want)
	}
}

// The upgrade is invisible. A file written before this feature carries no
// recorded slot, so the operator's arrangement is exactly what it was and Scratch
// appends behind it — whether the file already stored an order or predates that
// too. An operator who upgrades and never opens a scratch shell cannot tell.
func TestNoRecordedSlotLeavesTheArrangementUntouched(t *testing.T) {
	t.Run("a file that stores an order", func(t *testing.T) {
		dir := writeRegistry(t, `
[[space]]
  path = "/repos/alpha"
  order = 2

[[space]]
  path = "/repos/beta"
  order = 0

[[space]]
  path = "/repos/gamma"
  order = 1
`)
		want := []string{"/repos/beta", "/repos/gamma", "/repos/alpha", "scratch"}
		if got := slots(load(t, dir).List()); !equal(got, want) {
			t.Errorf("order = %v, want the stored arrangement with Scratch last, %v", got, want)
		}
	})

	t.Run("a file that predates the order too", func(t *testing.T) {
		dir := writeRegistry(t, preUpgrade)
		want := append(append([]string(nil), preUpgradeOrder...), "scratch")
		if got := slots(load(t, dir).List()); !equal(got, want) {
			t.Errorf("order = %v, want the old rule's arrangement with Scratch last, %v", got, want)
		}
	})
}

// A hand-edited slot the sequence cannot hold degrades the way every other
// mangled order here does: the arrangement suffers, the list never does.
func TestOutOfRangeScratchSlotAppends(t *testing.T) {
	dir := writeRegistry(t, `
scratch_order = 99

[[space]]
  path = "/repos/alpha"
  order = 0

[[space]]
  path = "/repos/beta"
  order = 1
`)
	r := load(t, dir)
	want := []string{"/repos/alpha", "/repos/beta", "scratch"}
	if got := slots(r.List()); !equal(got, want) {
		t.Errorf("order = %v, want %v", got, want)
	}
	assertTotalOrder(t, r, 2)
}

// Recency is still recorded — it is a useful fact beside last_agent — it just
// stops sorting anything.
func TestLastActiveIsStillRead(t *testing.T) {
	dir := writeRegistry(t, `
[[space]]
  path = "/repos/alpha"
  order = 0
  last_active = 2026-01-01T09:00:00Z
`)
	e := load(t, dir).List()[0]
	if want := time.Date(2026, 1, 1, 9, 0, 0, 0, time.UTC); !e.LastActive.Equal(want) {
		t.Errorf("last_active = %v, want %v", e.LastActive, want)
	}
}
