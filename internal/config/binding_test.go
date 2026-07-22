package config_test

import (
	"reflect"
	"strings"
	"testing"
)

// Every space registered before the kind cut still has [maps."<slug>"] tables in
// its committed config, and those files ride the repo — a teammate pulling this
// change gets them whether they want to or not. The decoder must simply not care:
// the tables are ignored, the bindings around them resolve exactly as they would
// without them, and nothing is warned about. A key that stopped mattering is not
// a reason to make someone edit a file.
func TestStaleMapTablesAreIgnored(t *testing.T) {
	const stale = `
[roles.implement]
adapter = "codex"

[maps."widget"]
kind = "implementation"

[maps."plan"]
kind = "planning"

[maps."typo"]
kind = "gibberish"
`
	res := resolveWith(t, "", stale)

	if len(res.Warnings) != 0 {
		t.Errorf("a stale [maps.*] table warned: %v", res.Warnings)
	}
	// The bindings beside them still resolve, so the tables were skipped rather
	// than taking the whole file down as malformed.
	if got := binding(t, res, "implement").Adapter; got != "codex" {
		t.Errorf("implement.adapter = %q, want codex — the file decoded past the map tables", got)
	}

	// And the same config with the tables removed is indistinguishable.
	clean := resolveWith(t, "", strings.SplitN(stale, "[maps.", 2)[0])
	if len(clean.Warnings) != len(res.Warnings) || len(clean.Bindings) != len(res.Bindings) {
		t.Error("the stale tables changed the resolution; they must be inert")
	}
	for i := range clean.Bindings {
		if !reflect.DeepEqual(clean.Bindings[i], res.Bindings[i]) {
			t.Errorf("binding %d differs with the stale tables present:\n got %+v\nwant %+v",
				i, res.Bindings[i], clean.Bindings[i])
		}
	}
}
