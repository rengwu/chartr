package config_test

import (
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/config"
)

// The agent library (registering agents, assigning them to roles). Every
// assertion is on the bytes the writer leaves in the operator's file and on what
// config.Resolve then makes of them — the same two public seams the binding
// writer is tested through.

func present(string) bool { return true }

func resolveWith(t *testing.T, userTOML, workspaceTOML string) config.Resolution {
	t.Helper()
	return config.Resolve(config.Input{
		WorkspaceTOML: []byte(workspaceTOML),
		UserTOML:      []byte(userTOML),
		SpacePath:     space,
		OnPath:        present,
	})
}

func binding(t *testing.T, res config.Resolution, role string) config.Resolved {
	t.Helper()
	for _, b := range res.Bindings {
		if string(b.Role) == role {
			return b
		}
	}
	t.Fatalf("role %q missing from resolution", role)
	return config.Resolved{}
}

// Registering an agent and assigning a role to it: the agent supplies the whole
// binding — adapter, args and delivery together — because a role runs one
// registered way of driving a harness, not a mix of one and its own leftovers.
func TestRegisterAnAgentAndAssignARoleToIt(t *testing.T) {
	out, err := config.SetUserAgent(nil, "claude-yolo", config.Agent{
		Adapter: "claude",
		Args:    []string{"--model", "sonnet", "--dangerously-skip-permissions"},
		Prompt:  "argv",
	})
	if err != nil {
		t.Fatalf("registering an agent: %v", err)
	}
	for _, want := range []string{
		"[agents.claude-yolo]",
		`adapter = "claude"`,
		`prompt = "argv"`,
		`args = ["--model", "sonnet", "--dangerously-skip-permissions"]`,
	} {
		if !strings.Contains(string(out), want) {
			t.Errorf("registered agent missing %q:\n%s", want, out)
		}
	}

	// Registered but unassigned: the library lists it, and no role has moved.
	res := resolveWith(t, string(out), "")
	if len(res.Agents) != 1 || res.Agents[0].Name != "claude-yolo" {
		t.Fatalf("library = %+v, want one agent named claude-yolo", res.Agents)
	}
	if b := binding(t, res, "implement"); b.Agent != "" || b.AdapterFrom != config.LayerBuiltin {
		t.Errorf("registering an agent moved a role: %+v", b)
	}

	// Assigned: the agent is the binding, every field of it.
	assigned := set(t, string(out), config.BindingEdit{
		SpacePath: space, Role: "implement", Field: config.FieldAgent, Value: "claude-yolo",
	})
	b := binding(t, resolveWith(t, assigned, ""), "implement")
	if b.Agent != "claude-yolo" {
		t.Errorf("implement.agent = %q, want claude-yolo", b.Agent)
	}
	if b.Adapter != "claude" || b.Prompt != "argv" {
		t.Errorf("assigned binding = %+v, want the agent's adapter and delivery", b.Binding)
	}
	if want := []string{"--model", "sonnet", "--dangerously-skip-permissions"}; strings.Join(b.Args, " ") != strings.Join(want, " ") {
		t.Errorf("assigned args = %q, want the agent's %q", b.Args, want)
	}
	// The whole binding came from one place, and says so.
	for _, from := range []config.Layer{b.AdapterFrom, b.ArgsFrom, b.PromptFrom} {
		if from != config.LayerUser {
			t.Errorf("assigned field provenance = %s, want user (the library's layer)", from)
		}
	}
	// Roles nobody assigned are untouched.
	if g := binding(t, resolveWith(t, assigned, ""), "grill"); g.Agent != "" || strings.Join(g.Args, " ") != "--model opus" {
		t.Errorf("assigning implement moved grill: %+v", g)
	}
}

// Clearing the assignment hands the role back to its own fields — an assignment
// is as reversible as any other override (story 42).
func TestClearingAnAssignmentRestoresTheFields(t *testing.T) {
	user, err := config.SetUserAgent(nil, "codex-fast", config.Agent{Adapter: "codex", Args: []string{"-m", "gpt-x"}})
	if err != nil {
		t.Fatalf("registering: %v", err)
	}
	assigned := set(t, string(user), config.BindingEdit{
		SpacePath: space, Role: "implement", Field: config.FieldAgent, Value: "codex-fast",
	})
	if b := binding(t, resolveWith(t, assigned, ""), "implement"); b.Adapter != "codex" {
		t.Fatalf("assignment did not take: %+v", b.Binding)
	}

	cleared := set(t, assigned, config.BindingEdit{
		SpacePath: space, Role: "implement", Field: config.FieldAgent, Clear: true,
	})
	b := binding(t, resolveWith(t, cleared, ""), "implement")
	if b.Agent != "" {
		t.Errorf("implement is still assigned to %q after clearing", b.Agent)
	}
	if b.Adapter != "claude" || strings.Join(b.Args, " ") != "--model sonnet" {
		t.Errorf("cleared binding = %+v, want the built-in fields back", b.Binding)
	}
	// The agent itself is still registered — clearing an assignment is not a delete.
	if len(resolveWith(t, cleared, "").Agents) != 1 {
		t.Errorf("clearing an assignment removed the agent from the library")
	}
}

// An assignment that names nothing registered never blocks a spawn: the role
// falls back to its own fields, and both the binding and the space's warnings say
// what happened rather than the name silently vanishing.
func TestDanglingAssignmentFallsBackAndSaysSo(t *testing.T) {
	user := set(t, "", config.BindingEdit{
		SpacePath: space, Role: "implement", Field: config.FieldAgent, Value: "never-registered",
	})
	res := resolveWith(t, user, "")
	b := binding(t, res, "implement")
	if b.Agent != "never-registered" {
		t.Errorf("agent = %q, want the assignment shown as it stands", b.Agent)
	}
	if b.AgentMissing == "" || !strings.Contains(b.AgentMissing, "never-registered") {
		t.Errorf("agentMissing = %q, want it to name the agent that resolved to nothing", b.AgentMissing)
	}
	if b.Adapter != "claude" || strings.Join(b.Args, " ") != "--model sonnet" {
		t.Errorf("dangling assignment did not fall back to the fields: %+v", b.Binding)
	}
	if !strings.Contains(strings.Join(res.Warnings, "\n"), "never-registered") {
		t.Errorf("warnings = %v, want one naming the unregistered agent", res.Warnings)
	}
}

// An assignment beside the fields it supersedes is surfaced, never silently
// dropped: a value that stopped mattering is exactly what costs an afternoon.
func TestAssignmentBesideFieldsWarns(t *testing.T) {
	user, _ := config.SetUserAgent(nil, "claude-yolo", config.Agent{Adapter: "claude", Args: []string{"--model", "opus"}})
	assigned := set(t, string(user), config.BindingEdit{
		SpacePath: space, Role: "implement", Field: config.FieldAgent, Value: "claude-yolo",
	})
	withField := set(t, assigned, config.BindingEdit{
		SpacePath: space, Role: "implement", Field: config.FieldArgs, Args: []string{"--model", "sonnet"},
	})

	res := resolveWith(t, withField, "")
	if b := binding(t, res, "implement"); strings.Join(b.Args, " ") != "--model opus" {
		t.Errorf("args = %q, want the agent's opus — the agent is the binding", b.Args)
	}
	joined := strings.Join(res.Warnings, "\n")
	if !strings.Contains(joined, "no longer apply") || !strings.Contains(joined, "claude-yolo") {
		t.Errorf("warnings = %v, want one saying the role's own fields no longer apply", res.Warnings)
	}
}

// Editing an agent is what changes every role assigned to it — that is the whole
// purchase of registering one rather than repeating four fields per role.
func TestEditingAnAgentMovesEveryRoleAssignedToIt(t *testing.T) {
	user, _ := config.SetUserAgent(nil, "house", config.Agent{Adapter: "claude", Args: []string{"--model", "sonnet"}})
	assigned := string(user)
	for _, role := range []string{"implement", "research"} {
		assigned = set(t, assigned, config.BindingEdit{
			SpacePath: space, Role: role, Field: config.FieldAgent, Value: "house",
		})
	}

	edited, err := config.SetUserAgent([]byte(assigned), "house", config.Agent{
		Adapter: "claude", Args: []string{"--model", "opus", "--dangerously-skip-permissions"},
	})
	if err != nil {
		t.Fatalf("editing the agent: %v", err)
	}
	res := resolveWith(t, string(edited), "")
	for _, role := range []string{"implement", "research"} {
		b := binding(t, res, role)
		if strings.Join(b.Args, " ") != "--model opus --dangerously-skip-permissions" {
			t.Errorf("%s did not follow its agent: %+v", role, b.Binding)
		}
	}
	// grill was never assigned, so it did not move.
	if g := binding(t, res, "grill"); strings.Join(g.Args, " ") != "--model opus" {
		t.Errorf("grill picked up an agent it was never assigned: %+v", g.Binding)
	}
}

// An edit rewrites the agent's own keys and nothing else: a field dropped is
// removed rather than written empty, the retired `model` key is cleared out on
// the way through, and the operator's comments and unrelated tables survive.
func TestEditingAnAgentPreservesTheFile(t *testing.T) {
	const handWrittenAgents = `# my machine
[agents.house]
# pinned while the sonnet build is flaky
adapter = "claude"
model = "opus"
args = ["--verbose"]

[agents.other]
adapter = "codex"
`
	out, err := config.SetUserAgent([]byte(handWrittenAgents), "house", config.Agent{
		Adapter: "claude", Args: []string{"--add-dir", "."}, Prompt: "--prompt",
	})
	if err != nil {
		t.Fatalf("editing: %v", err)
	}
	got := string(out)
	for _, want := range []string{
		"# my machine",
		"# pinned while the sonnet build is flaky",
		`args = ["--add-dir", "."]`,
		`prompt = "--prompt"`,
		"[agents.other]",
		`adapter = "codex"`,
	} {
		if !strings.Contains(got, want) {
			t.Errorf("edit lost %q:\n%s", want, got)
		}
	}
	if strings.Contains(got, "model = ") {
		t.Errorf("editing an agent left its retired model key behind:\n%s", got)
	}
	// And it still reads as two agents, one of them untouched.
	res := resolveWith(t, got, "")
	if len(res.Agents) != 2 {
		t.Fatalf("library = %+v, want both agents", res.Agents)
	}
	if res.Agents[0].Name != "house" || res.Agents[0].Prompt != "--prompt" {
		t.Errorf("edited agent = %+v", res.Agents[0])
	}
	if res.Agents[1].Name != "other" || res.Agents[1].Adapter != "codex" {
		t.Errorf("unrelated agent changed: %+v", res.Agents[1])
	}
}

// Deleting takes the agent and nothing else. The assignment is deliberately left
// standing — a delete here must not quietly rewrite a space's bindings — and
// AssignedRoles is what lets the surface say so before it happens.
func TestDeletingAnAgentLeavesAssignmentsToDangleVisibly(t *testing.T) {
	user, _ := config.SetUserAgent(nil, "house", config.Agent{Adapter: "claude", Args: []string{"--model", "opus"}})
	assigned := set(t, string(user), config.BindingEdit{
		SpacePath: space, Role: "implement", Field: config.FieldAgent, Value: "house",
	})

	if got := config.AssignedRoles([]byte(assigned), "house"); len(got) != 1 || !strings.Contains(got[0], "implement") {
		t.Errorf("AssignedRoles = %v, want the one implement assignment", got)
	}

	out, err := config.DeleteUserAgent([]byte(assigned), "house")
	if err != nil {
		t.Fatalf("deleting: %v", err)
	}
	if strings.Contains(string(out), "[agents.house]") {
		t.Errorf("the agent's table survived the delete:\n%s", out)
	}
	res := resolveWith(t, string(out), "")
	if len(res.Agents) != 0 {
		t.Errorf("library = %+v, want it empty", res.Agents)
	}
	b := binding(t, res, "implement")
	if b.AgentMissing == "" || b.Adapter != "claude" || strings.Join(b.Args, " ") != "--model sonnet" {
		t.Errorf("the stranded role did not fall back visibly: %+v / %q", b.Binding, b.AgentMissing)
	}

	// Deleting what is not there changes nothing rather than erroring.
	again, err := config.DeleteUserAgent(out, "house")
	if err != nil || string(again) != string(out) {
		t.Errorf("deleting an absent agent changed the file (%v):\n%s", err, again)
	}
}

// The library refuses what it cannot hold, with the reason in hand — and a
// registration that would need quoting is refused rather than escaped, which is
// what lets the writer emit a bare table header.
func TestAgentRefusals(t *testing.T) {
	for name, tc := range map[string]struct {
		agentName string
		agent     config.Agent
		want      string
	}{
		"no name":            {agentName: "", agent: config.Agent{Adapter: "claude"}, want: "name"},
		"exotic name":        {agentName: "my agent", agent: config.Agent{Adapter: "claude"}, want: "letters"},
		"dotted name":        {agentName: "a.b", agent: config.Agent{Adapter: "claude"}, want: "letters"},
		"no adapter":         {agentName: "x", agent: config.Agent{}, want: "adapter"},
		"adapter with flags": {agentName: "x", agent: config.Agent{Adapter: "claude --yolo"}, want: "one binary name"},
		"bad delivery":       {agentName: "x", agent: config.Agent{Adapter: "claude", Prompt: "stdin"}, want: "argv"},
	} {
		_, err := config.SetUserAgent(nil, tc.agentName, tc.agent)
		if err == nil {
			t.Errorf("%s: SetUserAgent succeeded, want a refusal", name)
			continue
		}
		if !strings.Contains(err.Error(), tc.want) {
			t.Errorf("%s: refusal %q does not mention %q", name, err, tc.want)
		}
	}

	// An agent whose file is unreadable in some way is dropped from the library
	// with a warning, never taking the rest of it down.
	res := resolveWith(t, "[agents.broken]\nmodel = \"x\"\n\n[agents.fine]\nadapter = \"claude\"\n", "")
	if len(res.Agents) != 1 || res.Agents[0].Name != "fine" {
		t.Errorf("library = %+v, want only the well-formed agent", res.Agents)
	}
	if !strings.Contains(strings.Join(res.Warnings, "\n"), "names no adapter") {
		t.Errorf("warnings = %v, want one about the adapterless agent", res.Warnings)
	}
}

// A harness with no model at all is the ordinary case, and the reason `model`
// was never a field worth having: an agent is a binary and a list of flags.
func TestAgentIsJustABinaryAndFlags(t *testing.T) {
	out, err := config.SetUserAgent(nil, "pi", config.Agent{Adapter: "pi", Args: []string{"--fast"}, Prompt: "type"})
	if err != nil {
		t.Fatalf("registering: %v", err)
	}
	if strings.Contains(string(out), "model") {
		t.Errorf("registering invented a model key:\n%s", out)
	}
	assigned := set(t, string(out), config.BindingEdit{
		SpacePath: space, Role: "grill", Field: config.FieldAgent, Value: "pi",
	})
	b := binding(t, resolveWith(t, assigned, ""), "grill")
	if b.Adapter != "pi" || strings.Join(b.Args, " ") != "--fast" || b.Prompt != "type" {
		t.Errorf("binding = %+v, want pi --fast with typed delivery", b.Binding)
	}
}

// `model` was a binding field and is not one any more — it is a flag, and flags
// live in args. A config that still sets it is *told*, in both layers and in the
// library, because a key that quietly stopped taking effect is how a session ends
// up running a model nobody chose. Nothing is migrated automatically: the chartr
// will not invent the flag name a given harness wants.
func TestRetiredModelKeyIsSurfacedNotHonoured(t *testing.T) {
	res := resolveWith(t,
		"[spaces.\""+space+"\".roles.implement]\nmodel = \"opus\"\n",
		"[roles.research]\nmodel = \"haiku\"\n")

	joined := strings.Join(res.Warnings, "\n")
	for _, want := range []string{
		`role implement sets model = "opus" in user config`,
		`role research sets model = "haiku" in workspace config`,
		`args = ["--model", "opus"]`,
	} {
		if !strings.Contains(joined, want) {
			t.Errorf("warnings do not say %q:\n%s", want, joined)
		}
	}
	// And it changed nothing: the built-in args still stand.
	if b := binding(t, res, "implement"); strings.Join(b.Args, " ") != "--model sonnet" {
		t.Errorf("a retired model key took effect: %+v", b.Binding)
	}

	// The library says the same about an agent that still carries one.
	lib := resolveWith(t, "[agents.old]\nadapter = \"claude\"\nmodel = \"opus\"\n", "")
	if len(lib.Agents) != 1 || lib.Agents[0].Adapter != "claude" {
		t.Fatalf("agent dropped over a retired key: %+v", lib.Agents)
	}
	if !strings.Contains(strings.Join(lib.Warnings, "\n"), `agent "old" sets model = "opus"`) {
		t.Errorf("library warnings = %v, want one about the retired key", lib.Warnings)
	}
}
