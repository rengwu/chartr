package config_test

import (
	"os"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/config"
)

// An agent's environment: the half of a launch spec that is not a flag. The
// assertions here are the same shape as the args ones — whatever the operator
// typed is what the process gets — plus the one interpretation this package does
// make, a leading `~/`, which exists because nothing else in the launch path is a
// shell and an unexpanded tilde fails silently.

// setHome points tilde expansion at a directory the test owns, and returns what
// os.UserHomeDir will now answer, so the assertions read the same on every
// platform rather than hard-coding one OS's home variable.
func setHome(t *testing.T) string {
	t.Helper()
	dir := t.TempDir()
	t.Setenv("HOME", dir)        // unix
	t.Setenv("USERPROFILE", dir) // windows
	home, err := os.UserHomeDir()
	if err != nil {
		t.Skipf("no home directory to expand against: %v", err)
	}
	return home
}

// The round trip an operator's `CLAUDE_CONFIG_DIR=~/.claude2 claude` becomes: the
// file keeps the tilde they typed, and the launch gets the path it means.
func TestAgentEnvRoundTripsTypedAndLaunchesExpanded(t *testing.T) {
	home := setHome(t)

	out, err := config.SetUserAgent(nil, "claude-alt", config.Agent{
		Adapter: "claude",
		Env:     []string{"CLAUDE_CONFIG_DIR=~/.claude2"},
	})
	if err != nil {
		t.Fatalf("registering an agent with an environment: %v", err)
	}
	// Stored as typed. Writing the expanded path here would bake one machine's home
	// into a config the operator reads and edits by hand.
	if !strings.Contains(string(out), `env = ["CLAUDE_CONFIG_DIR=~/.claude2"]`) {
		t.Errorf("the env was not written as typed:\n%s", out)
	}

	a := resolveAgents(t, string(out)).Agents[0]
	if got := strings.Join(a.Env, " "); got != "CLAUDE_CONFIG_DIR=~/.claude2" {
		t.Errorf("resolved Env = %q, want what the operator typed (the editing surface round trips it)", got)
	}
	if want := "CLAUDE_CONFIG_DIR=" + home + "/.claude2"; strings.Join(a.LaunchEnv, " ") != want {
		t.Errorf("LaunchEnv = %q, want %q — an unexpanded tilde makes the agent create a directory named `~`", a.LaunchEnv, want)
	}
}

// Expansion is deliberately narrow: the front of a value, and only when the tilde
// stands for a home. Everything else is an ordinary character, because the moment
// this package interprets more it has to be right about a whole shell grammar.
func TestTildeExpansionIsOnlyALeadingHome(t *testing.T) {
	home := setHome(t)

	for name, tc := range map[string]struct{ in, want string }{
		"leading slash":  {"X=~/.claude2", "X=" + home + "/.claude2"},
		"bare tilde":     {"X=~", "X=" + home},
		"another's home": {"X=~someone/.claude2", "X=~someone/.claude2"},
		"mid value":      {"X=/srv/~/backups", "X=/srv/~/backups"},
		"not a path":     {"X=a~b", "X=a~b"},
		"no expansion":   {"X=$HOME/.claude2", "X=$HOME/.claude2"},
		"empty value":    {"X=", "X="},
	} {
		res := resolveAgents(t, "[agents.a]\nadapter = \"claude\"\nenv = [\""+tc.in+"\"]\n")
		if len(res.Agents) != 1 {
			t.Fatalf("%s: agent dropped: %+v", name, res)
		}
		if got := strings.Join(res.Agents[0].LaunchEnv, " "); got != tc.want {
			t.Errorf("%s: %q expanded to %q, want %q", name, tc.in, got, tc.want)
		}
	}
}

// A variable chartr has never heard of is the ordinary case — the same stance
// args take. Nothing is added, nothing is reordered, nothing is interpreted.
func TestAgentEnvIsOpaque(t *testing.T) {
	setHome(t)

	res := resolveAgents(t, "[agents.a]\nadapter = \"claude\"\n"+
		"env = [\"ANTHROPIC_BASE_URL=https://proxy.internal\", \"NO_COLOR=1\", \"WHATEVER=a=b=c\"]\n")
	a := res.Agents[0]
	want := "ANTHROPIC_BASE_URL=https://proxy.internal NO_COLOR=1 WHATEVER=a=b=c"
	if got := strings.Join(a.LaunchEnv, " "); got != want {
		t.Errorf("LaunchEnv = %q, want %q verbatim and in order", got, want)
	}
	if len(res.Warnings) != 0 {
		t.Errorf("an ordinary environment produced warnings: %v", res.Warnings)
	}
}

// One bad entry costs only itself — the same rule one bad agent follows in the
// library. An agent with a typo in its third variable still launches with the
// other two rather than losing its whole environment.
func TestUnreadableEnvEntryIsDroppedNotFatal(t *testing.T) {
	setHome(t)

	res := resolveAgents(t, "[agents.a]\nadapter = \"claude\"\n"+
		"env = [\"GOOD=1\", \"JUST_A_WORD\", \"=novalue\", \"ALSO_GOOD=2\"]\n")
	if len(res.Agents) != 1 {
		t.Fatalf("the agent was dropped over a bad entry: %+v", res.Agents)
	}
	if got := strings.Join(res.Agents[0].LaunchEnv, " "); got != "GOOD=1 ALSO_GOOD=2" {
		t.Errorf("LaunchEnv = %q, want the readable entries kept", got)
	}
	joined := strings.Join(res.Warnings, "\n")
	for _, want := range []string{"JUST_A_WORD", "no `=`", "no name before"} {
		if !strings.Contains(joined, want) {
			t.Errorf("warnings %v do not mention %q", res.Warnings, want)
		}
	}
}

// PATH is passed like anything else and warned about, because setting it here
// cannot do the thing an operator would expect: the binary is resolved out of
// chartr's own PATH before the child environment applies (the env package).
// Surface, never enforce.
func TestPATHInAgentEnvIsWarnedNotRefused(t *testing.T) {
	setHome(t)

	res := resolveAgents(t, "[agents.a]\nadapter = \"claude\"\nenv = [\"PATH=/opt/custom/bin\"]\n")
	if got := strings.Join(res.Agents[0].LaunchEnv, " "); got != "PATH=/opt/custom/bin" {
		t.Errorf("LaunchEnv = %q, want PATH passed through anyway", got)
	}
	if !strings.Contains(strings.Join(res.Warnings, "\n"), "does not change which binary") {
		t.Errorf("warnings = %v, want one explaining that PATH cannot redirect the launch", res.Warnings)
	}
}

// The writer refuses at the gate what resolve would have to drop — a malformed
// entry never reaches the operator's file — and drops the key entirely when an
// agent stops carrying an environment, rather than leaving an empty list behind.
func TestAgentEnvWriteRefusalsAndRemoval(t *testing.T) {
	for name, tc := range map[string]struct{ entry, want string }{
		"no equals": {"JUST_A_WORD", "no `=`"},
		"no name":   {"=value", "no name"},
		"spaced":    {"TWO WORDS=1", "not a variable name"},
	} {
		_, err := config.SetUserAgent(nil, "x", config.Agent{Adapter: "claude", Env: []string{tc.entry}})
		if err == nil {
			t.Errorf("%s: SetUserAgent accepted %q, want a refusal", name, tc.entry)
			continue
		}
		if !strings.Contains(err.Error(), tc.want) {
			t.Errorf("%s: refusal %q does not mention %q", name, err, tc.want)
		}
	}

	with, err := config.SetUserAgent(nil, "a", config.Agent{Adapter: "claude", Env: []string{"X=1"}})
	if err != nil {
		t.Fatalf("registering: %v", err)
	}
	without, err := config.SetUserAgent(with, "a", config.Agent{Adapter: "claude"})
	if err != nil {
		t.Fatalf("re-registering without an environment: %v", err)
	}
	if strings.Contains(string(without), "env") {
		t.Errorf("an agent that dropped its environment kept the key:\n%s", without)
	}
}

// An agent with no environment at all is the overwhelmingly common case and must
// stay exactly as clean as it was before the field existed: no key in the file,
// nothing on the wire.
func TestNoEnvWritesNoKey(t *testing.T) {
	out, err := config.SetUserAgent(nil, "plain", config.Agent{Adapter: "claude", Args: []string{"--fast"}})
	if err != nil {
		t.Fatalf("registering: %v", err)
	}
	if strings.Contains(string(out), "env") {
		t.Errorf("registering invented an env key:\n%s", out)
	}
	a := resolveAgents(t, string(out)).Agents[0]
	if len(a.Env) != 0 || len(a.LaunchEnv) != 0 {
		t.Errorf("agent = %+v, want no environment at all", a)
	}
}
