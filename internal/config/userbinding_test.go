package config_test

import (
	"strings"
	"testing"

	"github.com/rengwu/wayfinder-harness/internal/config"
)

// The user-layer binding writer (ticket 05, stories 39–42). Every assertion is on
// the bytes the writer leaves in the file — that is the observable contract: the
// specific key changes, and the operator's comments, ordering, spacing, and
// unrelated tables survive it. Resolution of the result is asserted through
// config.Resolve, the public read seam, so "clearing reveals the layer beneath"
// is checked as an effective value rather than as an absence of text.

const space = "/home/op/proj"

// A file the operator wrote by hand: comments, deliberate ordering, spacing, and
// tables that have nothing to do with the edit under test.
const handWritten = `# my machine's overrides
# claude lives in ~/.local/bin here

[spaces."/home/op/other".roles.implement]
adapter = "codex"   # the other project runs on codex

[spaces."/home/op/proj".roles.implement]
# pinned while the sonnet build is flaky
model  = "opus"
args = ["--verbose"]

[spaces."/home/op/proj".roles.grill]
adapter = "claude"
`

func set(t *testing.T, in string, e config.BindingEdit) string {
	t.Helper()
	out, err := config.SetUserBinding([]byte(in), e)
	if err != nil {
		t.Fatalf("SetUserBinding(%+v): %v", e, err)
	}
	return string(out)
}

// resolveUser resolves a space's bindings from a user file alone, so a test can
// assert what the edited bytes actually mean. onPath is stubbed present so the
// PATH probe never colours the result.
func resolveUser(t *testing.T, userTOML, workspaceTOML, role string) config.Resolved {
	t.Helper()
	res := config.Resolve(config.Input{
		WorkspaceTOML: []byte(workspaceTOML),
		UserTOML:      []byte(userTOML),
		SpacePath:     space,
		OnPath:        func(string) bool { return true },
	})
	for _, b := range res.Bindings {
		if string(b.Role) == role {
			return b
		}
	}
	t.Fatalf("role %q missing from resolution", role)
	return config.Resolved{}
}

// Setting an existing key rewrites that one line and nothing else: the comment
// above it, the key order around it, the operator's alignment spacing on other
// lines, and every unrelated table come back byte for byte.
func TestSetExistingKeyPreservesEverythingElse(t *testing.T) {
	got := set(t, handWritten, config.BindingEdit{
		SpacePath: space, Role: "implement", Field: config.FieldModel, Value: "sonnet",
	})

	if !strings.Contains(got, `model = "sonnet"`) {
		t.Errorf("the edited key is not in the result:\n%s", got)
	}
	if strings.Contains(got, `model  = "opus"`) {
		t.Errorf("the old value survived the edit:\n%s", got)
	}

	// Everything but that one line is byte-identical.
	assertOnlyLineChanged(t, handWritten, got, `model  = "opus"`, `model = "sonnet"`)

	// And it means what it says.
	if b := resolveUser(t, got, "", "implement"); b.Model != "sonnet" || b.ModelFrom != config.LayerUser {
		t.Errorf("implement.model resolved %q from %q, want sonnet from user", b.Model, b.ModelFrom)
	}
}

// Setting a key the table does not have yet inserts one line after the keys
// already there — nothing existing moves, and the comment above the table's
// first key stays attached to it.
func TestSetNewKeyInExistingTable(t *testing.T) {
	got := set(t, handWritten, config.BindingEdit{
		SpacePath: space, Role: "grill", Field: config.FieldModel, Value: "opus",
	})

	want := "[spaces.\"/home/op/proj\".roles.grill]\nadapter = \"claude\"\nmodel = \"opus\"\n"
	if !strings.Contains(got, want) {
		t.Errorf("the new key did not land after the table's existing keys:\n%s", got)
	}
	for _, keep := range []string{
		"# pinned while the sonnet build is flaky",
		`model  = "opus"`,
		`adapter = "codex"   # the other project runs on codex`,
	} {
		if !strings.Contains(got, keep) {
			t.Errorf("insertion disturbed %q:\n%s", keep, got)
		}
	}
}

// A role with no table yet gets one appended in DeclareMapKind's style — a blank
// line off what precedes it — leaving the whole existing file ahead of it intact.
func TestSetCreatesAbsentTable(t *testing.T) {
	got := set(t, handWritten, config.BindingEdit{
		SpacePath: space, Role: "research", Field: config.FieldAdapter, Value: "codex",
	})

	if !strings.HasPrefix(got, handWritten) {
		t.Errorf("appending a table rewrote what came before it:\n%s", got)
	}
	if !strings.Contains(got, "[spaces.\"/home/op/proj\".roles.research]\nadapter = \"codex\"\n") {
		t.Errorf("the appended table is not in the result:\n%s", got)
	}
	if b := resolveUser(t, got, "", "research"); b.Adapter != "codex" || b.AdapterFrom != config.LayerUser {
		t.Errorf("research.adapter resolved %q from %q, want codex from user", b.Adapter, b.AdapterFrom)
	}

	// An absent file is the same path from empty bytes.
	fresh := set(t, "", config.BindingEdit{
		SpacePath: space, Role: "research", Field: config.FieldAdapter, Value: "codex",
	})
	if strings.HasPrefix(fresh, "\n") {
		t.Errorf("a fresh file starts with a blank line:\n%q", fresh)
	}
}

// Clearing an override deletes exactly that key line and reveals the layer
// beneath it — the whole point of the edit being reversible (story 42).
func TestClearRevealsTheLayerBeneath(t *testing.T) {
	const workspace = `
[roles.implement]
adapter = "claude"
model = "sonnet-ws"
`
	// Before: the user layer wins.
	if b := resolveUser(t, handWritten, workspace, "implement"); b.Model != "opus" || b.ModelFrom != config.LayerUser {
		t.Fatalf("precondition: implement.model resolved %q from %q, want opus from user", b.Model, b.ModelFrom)
	}

	got := set(t, handWritten, config.BindingEdit{
		SpacePath: space, Role: "implement", Field: config.FieldModel, Clear: true,
	})

	if strings.Contains(got, `model  = "opus"`) {
		t.Errorf("the cleared key is still in the file:\n%s", got)
	}
	// After: the workspace layer shows through, and the sibling override stands.
	b := resolveUser(t, got, workspace, "implement")
	if b.Model != "sonnet-ws" || b.ModelFrom != config.LayerWorkspace {
		t.Errorf("after clearing, implement.model resolved %q from %q, want sonnet-ws from workspace", b.Model, b.ModelFrom)
	}
	if len(b.Args) != 1 || b.Args[0] != "--verbose" || b.ArgsFrom != config.LayerUser {
		t.Errorf("clearing model disturbed the args override: %v from %q", b.Args, b.ArgsFrom)
	}
	// The comment that sat above the cleared key stays with its table.
	if !strings.Contains(got, "# pinned while the sonnet build is flaky") {
		t.Errorf("clearing took the operator's comment with it:\n%s", got)
	}
}

// Clearing something that was never overridden is a no-op, not an error and not
// a rewrite: the file comes back byte-identical.
func TestClearingAnAbsentOverrideChangesNothing(t *testing.T) {
	for _, e := range []config.BindingEdit{
		{SpacePath: space, Role: "grill", Field: config.FieldModel, Clear: true},     // table exists, key does not
		{SpacePath: space, Role: "prototype", Field: config.FieldModel, Clear: true}, // no table at all
	} {
		if got := set(t, handWritten, e); got != handWritten {
			t.Errorf("clearing an absent %s override rewrote the file:\n%s", e.Role, got)
		}
	}
}

// args is set and cleared as a whole, and an explicit empty list is a real
// value — it clears inherited args rather than inheriting them (the resolver's
// nil-vs-non-nil rule).
func TestArgsSetClearedAndEmptied(t *testing.T) {
	const workspace = "\n[roles.implement]\nargs = [\"--from-workspace\"]\n"

	set1 := set(t, handWritten, config.BindingEdit{
		SpacePath: space, Role: "implement", Field: config.FieldArgs,
		Args: []string{"--dangerously-skip-permissions", "-p"},
	})
	if !strings.Contains(set1, `args = ["--dangerously-skip-permissions", "-p"]`) {
		t.Errorf("args not written as a TOML array:\n%s", set1)
	}

	empty := set(t, handWritten, config.BindingEdit{
		SpacePath: space, Role: "implement", Field: config.FieldArgs,
	})
	if b := resolveUser(t, empty, workspace, "implement"); len(b.Args) != 0 || b.ArgsFrom != config.LayerUser {
		t.Errorf("an explicit empty args resolved %v from %q, want [] from user", b.Args, b.ArgsFrom)
	}

	cleared := set(t, handWritten, config.BindingEdit{
		SpacePath: space, Role: "implement", Field: config.FieldArgs, Clear: true,
	})
	if b := resolveUser(t, cleared, workspace, "implement"); len(b.Args) != 1 || b.ArgsFrom != config.LayerWorkspace {
		t.Errorf("after clearing, args resolved %v from %q, want the workspace layer", b.Args, b.ArgsFrom)
	}
}

// A multi-line array is replaced and deleted whole — the writer follows the
// value past its first line rather than leaving an orphaned tail.
func TestMultiLineArrayIsEditedWhole(t *testing.T) {
	const src = `[spaces."/home/op/proj".roles.implement]
args = [
  "--one",
  "--two",
]
model = "opus"
`
	got := set(t, src, config.BindingEdit{
		SpacePath: space, Role: "implement", Field: config.FieldArgs, Args: []string{"--three"},
	})
	if strings.Contains(got, "--one") || strings.Contains(got, "--two") {
		t.Errorf("a tail of the old array survived:\n%s", got)
	}
	if !strings.Contains(got, "args = [\"--three\"]\nmodel = \"opus\"\n") {
		t.Errorf("the multi-line array was not replaced whole:\n%s", got)
	}
	if b := resolveUser(t, got, "", "implement"); len(b.Args) != 1 || b.Args[0] != "--three" {
		t.Errorf("replaced args resolved %v, want [--three]", b.Args)
	}
	if b := resolveUser(t, got, "", "implement"); b.Model != "opus" {
		t.Errorf("replacing the array disturbed the key beneath it: model = %q", b.Model)
	}
}

// The writer keys the user layer by space, so an edit for one space never
// reaches another space's table for the same role.
func TestEditIsScopedToItsSpace(t *testing.T) {
	got := set(t, handWritten, config.BindingEdit{
		SpacePath: space, Role: "implement", Field: config.FieldAdapter, Value: "opencode",
	})
	if !strings.Contains(got, `adapter = "codex"   # the other project runs on codex`) {
		t.Errorf("the other space's binding was touched:\n%s", got)
	}
	other := config.Resolve(config.Input{
		UserTOML:  []byte(got),
		SpacePath: "/home/op/other",
		OnPath:    func(string) bool { return true },
	})
	for _, b := range other.Bindings {
		if b.Role == "implement" && b.Adapter != "codex" {
			t.Errorf("the other space's implement.adapter = %q, want codex", b.Adapter)
		}
	}
}

// The writer refuses what it cannot edit surgically rather than corrupting the
// file: an unknown role or field, a value-less set, and a role already bound in
// a shape this editor does not rewrite.
func TestRefusals(t *testing.T) {
	for name, e := range map[string]config.BindingEdit{
		"unknown role":  {SpacePath: space, Role: "review", Field: config.FieldModel, Value: "x"},
		"unknown field": {SpacePath: space, Role: "implement", Field: "autopilot", Value: "true"},
		"empty value":   {SpacePath: space, Role: "implement", Field: config.FieldModel},
		"no space":      {Role: "implement", Field: config.FieldModel, Value: "x"},
	} {
		if _, err := config.SetUserBinding([]byte(handWritten), e); err == nil {
			t.Errorf("%s: SetUserBinding succeeded, want a refusal", name)
		}
	}

	// An inline table binds the role in a shape the line editor does not rewrite.
	const inline = `[spaces."/home/op/proj".roles]
implement = { model = "opus" }
`
	_, err := config.SetUserBinding([]byte(inline), config.BindingEdit{
		SpacePath: space, Role: "implement", Field: config.FieldModel, Value: "sonnet",
	})
	if err == nil {
		t.Fatal("SetUserBinding rewrote an inline table, want a refusal pointing at a hand edit")
	}
	if !strings.Contains(err.Error(), "by hand") {
		t.Errorf("refusal = %q, want it to point at editing by hand", err)
	}
}

// assertOnlyLineChanged checks that from → to is the sole difference between two
// versions of a file, line for line.
func assertOnlyLineChanged(t *testing.T, before, after, from, to string) {
	t.Helper()
	b, a := strings.Split(before, "\n"), strings.Split(after, "\n")
	if len(a) != len(b) {
		t.Fatalf("line count changed: %d → %d\n%s", len(b), len(a), after)
	}
	for i := range b {
		switch {
		case b[i] == a[i]:
		case strings.TrimSpace(b[i]) == strings.TrimSpace(from) && strings.TrimSpace(a[i]) == strings.TrimSpace(to):
		default:
			t.Errorf("line %d changed unexpectedly:\n before %q\n after  %q", i+1, b[i], a[i])
		}
	}
}
