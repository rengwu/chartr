package registry_test

import (
	"os"
	"path/filepath"
	"sort"
	"testing"
	"time"

	"github.com/BurntSushi/toml"
	"github.com/rengwu/chartr/internal/registry"
)

// The stored sidebar order at the file seam. The migration is the test that
// matters: a registry written before spaces carried an order must load to
// exactly the sequence the *old* rule produced — pinned first, then recency,
// then path — and keep it once it is written back. The old rule is restated
// here as an independent oracle (oldRule) rather than a hand-copied list, so the
// assertion is against the behaviour being frozen and not against a fixture
// someone would have to re-derive by eye.
//
// The degradation cases follow: a hand-edited or truncated file costs the
// operator their arrangement, never their list of spaces.

// oldRule is the sidebar comparator as it stood before this order existed,
// lifted from the pre-change registry.List: pinned first, then most-recently-
// active, then path as the stable tiebreak.
func oldRule(entries []registry.Entry) []string {
	out := append([]registry.Entry{}, entries...)
	sort.SliceStable(out, func(i, j int) bool {
		if out[i].Pinned != out[j].Pinned {
			return out[i].Pinned
		}
		if !out[i].LastActive.Equal(out[j].LastActive) {
			return out[i].LastActive.After(out[j].LastActive)
		}
		return out[i].Path < out[j].Path
	})
	return paths(out)
}

func paths(entries []registry.Entry) []string {
	out := make([]string, 0, len(entries))
	for _, e := range entries {
		out = append(out, e.Path)
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

// The upgrade is invisible: a file with no order loads to exactly the sequence
// the old rule produced, and that sequence survives the save that writes it back.
func TestMigrationFreezesTodaysOrder(t *testing.T) {
	dir := writeRegistry(t, preUpgrade)
	r := load(t, dir)

	got := paths(r.List())
	want := oldRule(r.List())
	if !equal(got, want) {
		t.Fatalf("frozen order = %v, want the old rule's %v", got, want)
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
	list := r.List()
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

// A missing file is still the first-run state, and an empty registry has nothing
// to order.
func TestMissingFileLoadsEmpty(t *testing.T) {
	r := load(t, t.TempDir())
	if got := len(r.List()); got != 0 {
		t.Errorf("fresh registry lists %d spaces, want 0", got)
	}
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
