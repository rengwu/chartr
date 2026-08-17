package prompts_test

import (
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/prompts"
)

func write(t *testing.T, dir, body string) {
	t.Helper()
	if err := os.WriteFile(filepath.Join(dir, "prompts.toml"), []byte(body), 0o600); err != nil {
		t.Fatalf("writing the catalog: %v", err)
	}
}

func ids(list []prompts.Prompt) []string {
	out := make([]string, 0, len(list))
	for _, p := range list {
		out = append(out, p.ID)
	}
	return out
}

// A machine with no catalog has an empty one — the first-run state, not an
// error, and nothing is written until the operator creates a preset.
func TestMissingFileIsAnEmptyCatalog(t *testing.T) {
	dir := t.TempDir()
	c := prompts.Load(dir)

	if got := c.List(); len(got) != 0 {
		t.Errorf("a missing catalog listed %v, want nothing", ids(got))
	}
	if got := c.Warnings(); len(got) != 0 {
		t.Errorf("a missing catalog warned %v, want silence", got)
	}
	if _, err := os.Stat(filepath.Join(dir, "prompts.toml")); !os.IsNotExist(err) {
		t.Error("loading an absent catalog wrote a file")
	}
}

// The operator's own file reads back in file order, which is creation order.
func TestValidFileReadsInFileOrder(t *testing.T) {
	dir := t.TempDir()
	write(t, dir, `
[[prompt]]
id = "brief"
name = "Keep answers brief"
body = "Answer in as few words as the question allows."

[[prompt]]
id = "commit-convention"
name = "Commit convention"
body = "Follow the repository's commit convention."
`)

	c := prompts.Load(dir)
	if got := ids(c.List()); len(got) != 2 || got[0] != "brief" || got[1] != "commit-convention" {
		t.Fatalf("catalog = %v, want file order", got)
	}
	if len(c.Warnings()) != 0 {
		t.Errorf("a valid catalog warned %v", c.Warnings())
	}
	p, ok := c.Get("brief")
	if !ok || p.Name != "Keep answers brief" || !strings.HasPrefix(p.Body, "Answer in") {
		t.Errorf("Get(brief) = %+v, %v", p, ok)
	}
}

// Every way the file can be unreadable — a parse failure, a row missing its
// body, two rows claiming one id — yields no presets and a warning naming the
// file. Nothing is half-read: a catalog chartr cannot fully understand executes
// nothing rather than executing the part it happened to parse.
func TestMalformedFileYieldsNoPresetsAndWarns(t *testing.T) {
	for _, tc := range []struct{ name, body string }{
		{"unparseable", "[[prompt]\nid = "},
		{"empty body", "[[prompt]]\nid = \"a\"\nname = \"A\"\nbody = \"\"\n"},
		{"empty name", "[[prompt]]\nid = \"a\"\nname = \"\"\nbody = \"x\"\n"},
		{"no id", "[[prompt]]\nname = \"A\"\nbody = \"x\"\n"},
		{"duplicate id", "[[prompt]]\nid = \"a\"\nname = \"A\"\nbody = \"x\"\n\n[[prompt]]\nid = \"a\"\nname = \"B\"\nbody = \"y\"\n"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			dir := t.TempDir()
			write(t, dir, tc.body)
			c := prompts.Load(dir)

			if got := c.List(); len(got) != 0 {
				t.Errorf("a malformed catalog listed %v, want nothing", ids(got))
			}
			warns := c.Warnings()
			if len(warns) != 1 || !strings.Contains(warns[0], "prompts.toml") {
				t.Fatalf("warnings = %v, want one naming the file", warns)
			}
		})
	}
}

// A mutation against a catalog chartr could not read is refused, and the
// operator's bytes are left exactly as they wrote them: an unreadable file is
// theirs to fix, never chartr's to overwrite.
func TestMutationsRefuseToOverwriteAMalformedFile(t *testing.T) {
	dir := t.TempDir()
	const original = "[[prompt]\nid = "
	write(t, dir, original)
	c := prompts.Load(dir)

	if _, err := c.Create("Brief", "Keep it short."); !errors.Is(err, prompts.ErrMalformed) {
		t.Errorf("Create on a malformed catalog = %v, want ErrMalformed", err)
	}
	if _, err := c.Update("a", "Brief", "Keep it short."); !errors.Is(err, prompts.ErrMalformed) {
		t.Errorf("Update on a malformed catalog = %v, want ErrMalformed", err)
	}
	if err := c.Delete("a"); !errors.Is(err, prompts.ErrMalformed) {
		t.Errorf("Delete on a malformed catalog = %v, want ErrMalformed", err)
	}

	got, err := os.ReadFile(filepath.Join(dir, "prompts.toml"))
	if err != nil || string(got) != original {
		t.Errorf("the operator's file is now %q (%v), want it untouched", got, err)
	}
}

// Creation appends, derives a stable kebab-case id from the name, and persists
// atomically — a fresh load sees exactly what the mutations left.
func TestCreateAppendsAndPersists(t *testing.T) {
	dir := t.TempDir()
	c := prompts.Load(dir)

	first, err := c.Create("  Keep answers brief  ", "Answer briefly.\n")
	if err != nil {
		t.Fatalf("Create: %v", err)
	}
	if first.ID != "keep-answers-brief" || first.Name != "Keep answers brief" {
		t.Errorf("created %+v, want a trimmed name and a kebab-case id", first)
	}
	if _, err := c.Create("Commit convention", "Use the repo's convention."); err != nil {
		t.Fatalf("Create: %v", err)
	}

	want := []string{"keep-answers-brief", "commit-convention"}
	for _, got := range [][]string{ids(c.List()), ids(prompts.Load(dir).List())} {
		if len(got) != 2 || got[0] != want[0] || got[1] != want[1] {
			t.Errorf("catalog = %v, want %v in creation order", got, want)
		}
	}
}

// Two presets named the same thing are two presets: the id is uniquified rather
// than the second creation being refused or silently folded into the first.
func TestCreateUniquifiesIDs(t *testing.T) {
	c := prompts.Load(t.TempDir())
	a, _ := c.Create("Brief", "one")
	b, err := c.Create("Brief", "two")
	if err != nil {
		t.Fatalf("second Create: %v", err)
	}
	if a.ID != "brief" || b.ID != "brief-2" {
		t.Errorf("ids = %q, %q, want brief and brief-2", a.ID, b.ID)
	}
}

// A name that kebab-cases to nothing still gets an id, because the id is
// chartr's identity for the row and never the operator's to supply.
func TestCreateNamesTheUnnameable(t *testing.T) {
	c := prompts.Load(t.TempDir())
	p, err := c.Create("!!!", "body")
	if err != nil {
		t.Fatalf("Create: %v", err)
	}
	if p.ID == "" {
		t.Error("a preset was created with no id")
	}
	if _, ok := c.Get(p.ID); !ok {
		t.Errorf("the created id %q does not resolve", p.ID)
	}
}

// A preset is a name and a body; neither may be blank.
func TestCreateRefusesBlankNameOrBody(t *testing.T) {
	c := prompts.Load(t.TempDir())
	if _, err := c.Create("  ", "body"); !errors.Is(err, prompts.ErrInvalid) {
		t.Errorf("blank name = %v, want ErrInvalid", err)
	}
	if _, err := c.Create("Name", "  \n "); !errors.Is(err, prompts.ErrInvalid) {
		t.Errorf("blank body = %v, want ErrInvalid", err)
	}
	if got := c.List(); len(got) != 0 {
		t.Errorf("a refused creation left %v behind", ids(got))
	}
}

// Editing keeps the id, so every space that selected the preset keeps pointing
// at it and future launches simply receive the new text.
func TestUpdateKeepsTheIDAndPosition(t *testing.T) {
	dir := t.TempDir()
	c := prompts.Load(dir)
	first, _ := c.Create("Brief", "one")
	_, _ = c.Create("Second", "two")

	got, err := c.Update(first.ID, "Much briefer", "one, revised")
	if err != nil {
		t.Fatalf("Update: %v", err)
	}
	if got.ID != first.ID || got.Name != "Much briefer" || got.Body != "one, revised" {
		t.Errorf("updated to %+v, want the same id with the new text", got)
	}
	if order := ids(prompts.Load(dir).List()); len(order) != 2 || order[0] != first.ID {
		t.Errorf("after an edit the order is %v, want the edited row where it was", order)
	}
	if _, err := c.Update("nope", "x", "y"); !errors.Is(err, prompts.ErrNotFound) {
		t.Errorf("Update of an unknown id = %v, want ErrNotFound", err)
	}
	if _, err := c.Update(first.ID, "", "y"); !errors.Is(err, prompts.ErrInvalid) {
		t.Errorf("Update to a blank name = %v, want ErrInvalid", err)
	}
}

func TestDeleteRemovesTheRow(t *testing.T) {
	dir := t.TempDir()
	c := prompts.Load(dir)
	first, _ := c.Create("Brief", "one")
	second, _ := c.Create("Second", "two")

	if err := c.Delete(first.ID); err != nil {
		t.Fatalf("Delete: %v", err)
	}
	if got := ids(prompts.Load(dir).List()); len(got) != 1 || got[0] != second.ID {
		t.Errorf("after deletion the catalog is %v, want just %q", got, second.ID)
	}
	if err := c.Delete(first.ID); !errors.Is(err, prompts.ErrNotFound) {
		t.Errorf("deleting twice = %v, want ErrNotFound", err)
	}
}

// A selection is a set: it is resolved and composed in catalog order however the
// ids arrive, and an id the catalog no longer holds is reported rather than
// substituted with another preset.
func TestSelectedComposesInCatalogOrder(t *testing.T) {
	c := prompts.Load(t.TempDir())
	a, _ := c.Create("Alpha", "one")
	b, _ := c.Create("Bravo", "two")
	_, _ = c.Create("Charlie", "three")

	chosen, missing := c.Selected([]string{"gone", b.ID, a.ID})
	if got := ids(chosen); len(got) != 2 || got[0] != a.ID || got[1] != b.ID {
		t.Errorf("selection = %v, want %v in catalog order", got, []string{a.ID, b.ID})
	}
	if len(missing) != 1 || missing[0] != "gone" {
		t.Errorf("missing = %v, want the one unknown id", missing)
	}
}
