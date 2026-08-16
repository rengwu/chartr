package env

import (
	"slices"
	"testing"
)

// lookupMap drives restoreHost the way os.LookupEnv drives HostEnviron.
func lookupMap(vars map[string]string) func(string) (string, bool) {
	return func(name string) (string, bool) {
		v, ok := vars[name]
		return v, ok
	}
}

func TestRestoreHostNotBundled(t *testing.T) {
	env := []string{"LD_LIBRARY_PATH=/opt/mine/lib", "HOME=/home/op"}
	got := restoreHost(env, lookupMap(nil))
	if !slices.Equal(got, env) {
		t.Fatalf("no snapshot marker must leave the environment untouched, got %v", got)
	}
}

func TestRestoreHostRestoresSnapshots(t *testing.T) {
	env := []string{
		"LD_LIBRARY_PATH=/tmp/.mount/usr/lib:",
		"XDG_DATA_DIRS=/usr/local/share:/usr/share:/tmp/.mount/usr/share",
		"GIO_MODULE_DIR=/tmp/.mount/usr/lib/gio/modules",
		"HOME=/home/op",
		snapshotMarker + "=1",
		snapshotPrefix + "LD_LIBRARY_PATH=",
		snapshotPrefix + "XDG_DATA_DIRS=/usr/local/share:/usr/share",
	}
	got := restoreHost(env, lookupMap(map[string]string{
		snapshotMarker:                     "1",
		snapshotPrefix + "LD_LIBRARY_PATH": "",
		snapshotPrefix + "XDG_DATA_DIRS":   "/usr/local/share:/usr/share",
	}))
	// GIO_MODULE_DIR had no snapshot: the operator never set it, so it comes
	// back unset rather than as the bundle's value or an empty string.
	want := []string{
		"HOME=/home/op",
		"LD_LIBRARY_PATH=",
		"XDG_DATA_DIRS=/usr/local/share:/usr/share",
	}
	if !slices.Equal(got, want) {
		t.Fatalf("got %v, want %v", got, want)
	}
}

func TestRestoreHostEmptySnapshotStaysSet(t *testing.T) {
	// Empty is not unset: an empty XDG_DATA_DIRS tells GTK there are
	// definitively no data dirs, and restoring it as absent would silently
	// widen the search.
	got := restoreHost([]string{"XDG_DATA_DIRS=/bundle"}, lookupMap(map[string]string{
		snapshotMarker:                   "1",
		snapshotPrefix + "XDG_DATA_DIRS": "",
	}))
	if !slices.Equal(got, []string{"XDG_DATA_DIRS="}) {
		t.Fatalf("an empty snapshot must round-trip as set-but-empty, got %v", got)
	}
}

func TestRestoreHostStripsBookkeeping(t *testing.T) {
	got := restoreHost([]string{snapshotMarker + "=1", snapshotPrefix + "HOME=/x", "HOME=/home/op"},
		lookupMap(map[string]string{snapshotMarker: "1"}))
	if !slices.Equal(got, []string{"HOME=/home/op"}) {
		t.Fatalf("marker and snapshot variables must not leak into children, got %v", got)
	}
}

func TestHostEnviron(t *testing.T) {
	t.Setenv(snapshotMarker, "1")
	t.Setenv(snapshotPrefix+"LD_LIBRARY_PATH", "/host/lib")
	t.Setenv("LD_LIBRARY_PATH", "/bundle/lib")
	t.Setenv("CHARTR_TEST_UNTOUCHED", "yes")
	got := HostEnviron()
	if !slices.Contains(got, "LD_LIBRARY_PATH=/host/lib") {
		t.Fatalf("snapshot not restored: %v", got)
	}
	if !slices.Contains(got, "CHARTR_TEST_UNTOUCHED=yes") {
		t.Fatalf("unrelated variable dropped: %v", got)
	}
	if slices.Contains(got, snapshotMarker+"=1") {
		t.Fatalf("marker leaked: %v", got)
	}
}
