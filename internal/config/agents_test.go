package config_test

import (
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/config"
)

// The agent library — the only execution config there is (ADR 0009 as
// superseded). Every assertion is on the bytes the writer leaves in the
// operator's file and on what config.ResolveAgents then makes of them: the two
// public seams. The assertions that matter are the ones a curated flag list could
// never make — whatever the operator typed is what launches, whether or not this
// repository has ever heard of the flag.

func present(string) bool { return true }

func resolveAgents(t *testing.T, userTOML string) config.Resolution {
	t.Helper()
	agents, warnings := config.ResolveAgents([]byte(userTOML), present)
	return config.Resolution{Agents: agents, Warnings: warnings}
}

// Registering an agent writes its table and the library reads it back — the whole
// round trip a role no longer sits in the middle of.
func TestRegisterAnAgent(t *testing.T) {
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

	res := resolveAgents(t, string(out))
	if len(res.Agents) != 1 || res.Agents[0].Name != "claude-yolo" {
		t.Fatalf("library = %+v, want one agent named claude-yolo", res.Agents)
	}
	a := res.Agents[0]
	if a.Adapter != "claude" || a.Prompt != "argv" {
		t.Errorf("resolved agent = %+v, want claude with argv delivery", a.Agent)
	}
	if strings.Join(a.Args, " ") != "--model sonnet --dangerously-skip-permissions" {
		t.Errorf("resolved args = %q, want what was registered", a.Args)
	}
}

// An edit rewrites the agent's own keys and nothing else: a field dropped is
// removed rather than written empty, and the operator's comments and unrelated
// tables survive.
func TestEditingAnAgentPreservesTheFile(t *testing.T) {
	const handWrittenAgents = `# my machine
[agents.house]
# pinned while the sonnet build is flaky
adapter = "claude"
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
	// And it still reads as two agents, one of them untouched.
	res := resolveAgents(t, got)
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

// Deleting takes the agent's table and nothing else, and deleting what is not
// there changes nothing rather than erroring.
func TestDeletingAnAgent(t *testing.T) {
	user, _ := config.SetUserAgent(nil, "house", config.Agent{Adapter: "claude", Args: []string{"--model", "opus"}})

	out, err := config.DeleteUserAgent(user, "house")
	if err != nil {
		t.Fatalf("deleting: %v", err)
	}
	if strings.Contains(string(out), "[agents.house]") {
		t.Errorf("the agent's table survived the delete:\n%s", out)
	}
	if got := resolveAgents(t, string(out)); len(got.Agents) != 0 {
		t.Errorf("library = %+v, want it empty", got.Agents)
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

	// One bad table never takes the rest of the library down: an adapterless agent
	// is dropped with a warning, the well-formed one stands.
	res := resolveAgents(t, "[agents.broken]\nprompt = \"argv\"\n\n[agents.fine]\nadapter = \"claude\"\n")
	if len(res.Agents) != 1 || res.Agents[0].Name != "fine" {
		t.Errorf("library = %+v, want only the well-formed agent", res.Agents)
	}
	if !strings.Contains(strings.Join(res.Warnings, "\n"), "names no adapter") {
		t.Errorf("warnings = %v, want one about the adapterless agent", res.Warnings)
	}
}

// A harness with no model at all is the ordinary case, and the reason `model` was
// never a field worth having: an agent is a binary and a list of flags.
func TestAgentIsJustABinaryAndFlags(t *testing.T) {
	out, err := config.SetUserAgent(nil, "pi", config.Agent{Adapter: "pi", Args: []string{"--fast"}, Prompt: "type"})
	if err != nil {
		t.Fatalf("registering: %v", err)
	}
	if strings.Contains(string(out), "model") {
		t.Errorf("registering invented a model key:\n%s", out)
	}
	a := resolveAgents(t, string(out)).Agents[0]
	if a.Adapter != "pi" || strings.Join(a.Args, " ") != "--fast" || a.Prompt != "type" {
		t.Errorf("resolved agent = %+v, want pi --fast with typed delivery", a.Agent)
	}
}

// `model` is not a key chartr reads any more — it is a flag, and flags live in
// args. There are no users to migrate, so a config that still carries one is not
// warned about or rewritten: an old key is simply an unknown key the non-strict
// decoder ignores, and the agent resolves clean.
func TestRetiredModelKeyIsInertNotWarned(t *testing.T) {
	res := resolveAgents(t, "[agents.old]\nadapter = \"claude\"\nmodel = \"opus\"\n")
	if len(res.Agents) != 1 || res.Agents[0].Adapter != "claude" {
		t.Fatalf("agent dropped over a stale key: %+v", res.Agents)
	}
	if len(res.Warnings) != 0 {
		t.Errorf("a stale model key produced a warning: %v", res.Warnings)
	}
}

// DetectAgents is the advisory PATH probe (ticket 04): it reports which of the
// curated known CLIs are present, in curated order, and reports nothing when none
// are — a hint for the registration surface, never a constraint, and it asserts
// only that a binary exists (ADR 0002). The probe is injected here so the test is
// hermetic rather than at the mercy of what the machine happens to have installed.
func TestDetectAgents(t *testing.T) {
	// Nothing on PATH: the probe finds nothing rather than inventing a default.
	if got := config.DetectAgents(func(string) bool { return false }); len(got) != 0 {
		t.Errorf("probe on a bare PATH = %v, want nothing", got)
	}

	// Exactly the ones present, and nothing it was not asked about — a name that is
	// not in the curated list is never reported even if it is on PATH.
	installed := map[string]bool{"claude": true, "goose": true, "totally-unknown-cli": true}
	got := config.DetectAgents(func(name string) bool { return installed[name] })
	want := map[string]bool{"claude": true, "goose": true}
	if len(got) != len(want) {
		t.Fatalf("probe reported %v, want exactly claude and goose", got)
	}
	for _, name := range got {
		if !want[name] {
			t.Errorf("probe reported %q, which was not asked to be detected", name)
		}
	}
}
