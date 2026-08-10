package config_test

import (
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/config"
)

// The role bindings' read and write halves. The write is the operator's own file,
// so the assertions are as much about what *survives* an edit — their comments,
// their key order, their unrelated tables — as about what changed.

func qualified(r config.Role) string { return "chartr-skills/" + string(r) }

// A config with no `[roles]` table gets all four, qualified, and nothing else is
// disturbed: the agent library and the operator's comments come through verbatim.
func TestSeedRoleBindingsWritesFourQualifiedRows(t *testing.T) {
	existing := "# my agents\n[agents.claude]\nadapter = \"claude\" # the good one\n"

	out, wrote := config.SeedRoleBindings([]byte(existing), qualified)
	if !wrote {
		t.Fatal("a config with no [roles] table was not seeded")
	}
	got := string(out)
	if !strings.HasPrefix(got, existing) {
		t.Errorf("seeding disturbed what was already there:\n%s", got)
	}
	for _, r := range config.Roles {
		if !strings.Contains(got, string(r)+` `) && !strings.Contains(got, string(r)+" =") {
			t.Errorf("no row for the %s role:\n%s", r, got)
		}
	}

	bindings, present := config.ReadRoleBindings(out)
	if !present || len(bindings) != len(config.Roles) {
		t.Fatalf("seeded config reads %d bindings (present %v), want %d", len(bindings), present, len(config.Roles))
	}
	for _, r := range config.Roles {
		if bindings[r] != qualified(r) {
			t.Errorf("the %s role is bound to %q, want %q", r, bindings[r], qualified(r))
		}
		// Never bare: a bare name would follow whatever source order happens to be.
		if !strings.Contains(bindings[r], "/") {
			t.Errorf("the %s role's binding %q is not source-qualified", r, bindings[r])
		}
	}
}

// Seeding is once, and the presence test is the *table* — so a second startup
// writes nothing, and a table one row short is left one row short. A deleted row
// is a legitimate way to make a role refuse until it is rebound.
func TestSeedRoleBindingsNeverRefills(t *testing.T) {
	seeded, _ := config.SeedRoleBindings(nil, qualified)
	if _, wrote := config.SeedRoleBindings(seeded, qualified); wrote {
		t.Error("a second startup rewrote the role bindings")
	}

	// A table the operator emptied a row out of.
	short := "[roles]\ngrill = \"mine/grill\"\n"
	out, wrote := config.SeedRoleBindings([]byte(short), qualified)
	if wrote || string(out) != short {
		t.Errorf("a half-filled table was refilled:\n%s", out)
	}
	bindings, present := config.ReadRoleBindings(out)
	if !present {
		t.Fatal("a half-filled table does not read as present")
	}
	if bindings[config.RoleGrill] != "mine/grill" {
		t.Errorf("the operator's own binding was not read: %q", bindings[config.RoleGrill])
	}
	if _, ok := bindings[config.RoleImplement]; ok {
		t.Error("a role with no row read as bound")
	}
}

// Rebinding one row is line surgery, not a rewrite: the row changes in place and
// everything around it — including a comment inside the table — is untouched.
func TestSetUserRoleEditsInPlace(t *testing.T) {
	existing := "[roles]\n# picked deliberately\ngrill = \"chartr-skills/grill\"\nimplement = \"chartr-skills/implement\"\n\n[agents.claude]\nadapter = \"claude\"\n"

	out, err := config.SetUserRole([]byte(existing), config.RoleGrill, "mine/interrogate")
	if err != nil {
		t.Fatalf("rebinding: %v", err)
	}
	got := string(out)
	if !strings.Contains(got, `grill = "mine/interrogate"`) {
		t.Errorf("the binding was not rewritten:\n%s", got)
	}
	for _, keep := range []string{"# picked deliberately", `implement = "chartr-skills/implement"`, "[agents.claude]"} {
		if !strings.Contains(got, keep) {
			t.Errorf("rebinding lost %q:\n%s", keep, got)
		}
	}
	if strings.Contains(got, "chartr-skills/grill") {
		t.Errorf("the old binding survived beside the new one:\n%s", got)
	}

	// A role with no row yet is appended into the existing table rather than
	// starting a second one.
	out, err = config.SetUserRole(out, config.RoleResearch, "mine/dig")
	if err != nil {
		t.Fatalf("binding a role with no row: %v", err)
	}
	if n := strings.Count(string(out), "[roles]"); n != 1 {
		t.Errorf("binding a missing role wrote %d [roles] tables:\n%s", n, out)
	}
	if b, _ := config.ReadRoleBindings(out); b[config.RoleResearch] != "mine/dig" {
		t.Errorf("the appended binding does not read back: %q", b[config.RoleResearch])
	}
}

// A bare name is refused at the writer, where the operator can still be told why:
// what a role runs must be readable in the line, not inferred from source order.
func TestSetUserRoleRefusesABareName(t *testing.T) {
	_, err := config.SetUserRole(nil, config.RoleGrill, "grill")
	if err == nil {
		t.Fatal("a bare binding was accepted")
	}
	if !strings.Contains(err.Error(), "Source/grill") {
		t.Errorf("the refusal does not show the qualified form: %v", err)
	}
	if _, err := config.SetUserRole(nil, config.Role("charting"), "mine/chart"); err == nil {
		t.Error("a binding for a role that does not exist was accepted")
	}
}

// "auto" is the one bare word the writer accepts: it names no source because
// following source order is exactly what it asks for. It round-trips like any
// other binding.
func TestSetUserRoleAcceptsAuto(t *testing.T) {
	out, err := config.SetUserRole(nil, config.RoleGrill, config.RoleBindingAuto)
	if err != nil {
		t.Fatalf("the auto sentinel was refused: %v", err)
	}
	if !strings.Contains(string(out), `grill = "auto"`) {
		t.Errorf("the auto binding was not written:\n%s", out)
	}
	if b, _ := config.ReadRoleBindings(out); b[config.RoleGrill] != config.RoleBindingAuto {
		t.Errorf("the auto binding does not read back: %q", b[config.RoleGrill])
	}
}

// An unparseable file reads as no table, which seeds a fresh one rather than
// erroring — the same degradation every other read of this file takes.
func TestReadRoleBindingsToleratesRubbish(t *testing.T) {
	if b, present := config.ReadRoleBindings([]byte("this is not toml [[[")); present || len(b) != 0 {
		t.Errorf("unparseable config read as %d bindings (present %v)", len(b), present)
	}
}
