package adapter

import (
	"reflect"
	"testing"
)

const (
	testHome = "/home/operator"
	testDir  = "/work/repo"
)

func TestStateRootVars(t *testing.T) {
	if got, want := StateRootVars("claude"), []string{"CLAUDE_CONFIG_DIR"}; !reflect.DeepEqual(got, want) {
		t.Errorf("StateRootVars(claude) = %v, want %v", got, want)
	}
	// A provider whose variable names a parent still declares just the variable:
	// the allowlist is what may be *read*, and the provider's own path
	// arithmetic is applied after reading it.
	if got, want := StateRootVars("opencode"), []string{"XDG_DATA_HOME"}; !reflect.DeepEqual(got, want) {
		t.Errorf("StateRootVars(opencode) = %v, want %v", got, want)
	}
	if got, want := StateRootVars("pi"), []string{"PI_CODING_AGENT_SESSION_DIR", "PI_CODING_AGENT_DIR"}; !reflect.DeepEqual(got, want) {
		t.Errorf("StateRootVars(pi) = %v, want %v", got, want)
	}
	if got := StateRootVars("nothing-chartr-knows"); got != nil {
		t.Errorf("StateRootVars(unknown) = %v, want nil", got)
	}
}

// The allowlist a reader is handed is a copy: a caller that sorts or truncates
// it cannot narrow what the next lookup is allowed to read.
func TestStateRootVarsIsACopy(t *testing.T) {
	got := StateRootVars("claude")
	got[0] = "MUTATED"
	if again := StateRootVars("claude"); again[0] != "CLAUDE_CONFIG_DIR" {
		t.Fatalf("table mutated through the returned slice: %v", again)
	}
}

func TestStateRoot(t *testing.T) {
	for _, tc := range []struct {
		name    string
		adapter string
		env     map[string]string
		dir     string
		noHome  bool // the OS would not say where home is
		want    string
		wantOK  bool
	}{{
		name:    "documented default when the variable is unset",
		adapter: "claude",
		want:    testHome + "/.claude",
		wantOK:  true,
	}, {
		name:    "an absolute value is taken as given",
		adapter: "claude",
		env:     map[string]string{"CLAUDE_CONFIG_DIR": "/srv/roots/one"},
		want:    "/srv/roots/one",
		wantOK:  true,
	}, {
		// The motivating alias: `CLAUDE_CONFIG_DIR=~/.claude2 claude`. Nothing
		// but a shell expands a tilde, and the value reaches the process raw
		// when it was set by chartr's own launch env, so the reader must.
		name:    "a user-relative value expands against home",
		adapter: "claude",
		env:     map[string]string{"CLAUDE_CONFIG_DIR": "~/.claude2"},
		want:    testHome + "/.claude2",
		wantOK:  true,
	}, {
		name:    "a bare tilde is home itself",
		adapter: "claude",
		env:     map[string]string{"CLAUDE_CONFIG_DIR": "~"},
		want:    testHome,
		wantOK:  true,
	}, {
		name:    "a relative value resolves against the process working directory",
		adapter: "claude",
		env:     map[string]string{"CLAUDE_CONFIG_DIR": ".claude2"},
		dir:     testDir,
		want:    testDir + "/.claude2",
		wantOK:  true,
	}, {
		name:    "a dot-dot value is cleaned",
		adapter: "claude",
		env:     map[string]string{"CLAUDE_CONFIG_DIR": "../shared/.claude"},
		dir:     testDir,
		want:    "/work/shared/.claude",
		wantOK:  true,
	}, {
		name:    "an untidy absolute value is cleaned",
		adapter: "claude",
		env:     map[string]string{"CLAUDE_CONFIG_DIR": "/srv//roots/./one/"},
		want:    "/srv/roots/one",
		wantOK:  true,
	}, {
		// Set-but-empty is how an operator unsets a variable in a launch env;
		// an empty path is no path, so the documented default stands.
		name:    "a set-but-empty value falls back to the default",
		adapter: "claude",
		env:     map[string]string{"CLAUDE_CONFIG_DIR": ""},
		want:    testHome + "/.claude",
		wantOK:  true,
	}, {
		// ~user needs a passwd lookup this package will not do. Guessing would
		// be a false positive; unavailable is the cheap failure.
		name:    "another user's home is not interpreted",
		adapter: "claude",
		env:     map[string]string{"CLAUDE_CONFIG_DIR": "~someone/.claude"},
		wantOK:  false,
	}, {
		name:    "a relative value with no working directory is unavailable",
		adapter: "claude",
		env:     map[string]string{"CLAUDE_CONFIG_DIR": ".claude2"},
		dir:     "",
		wantOK:  false,
	}, {
		name:    "no home and no variable is unavailable rather than guessed",
		adapter: "claude",
		noHome:  true,
		wantOK:  false,
	}, {
		name:    "kimi carries its own variable and default",
		adapter: "kimi",
		want:    testHome + "/.kimi-code",
		wantOK:  true,
	}, {
		name:    "kimi's variable selects its root",
		adapter: "kimi",
		env:     map[string]string{"KIMI_CODE_HOME": "~/.kimi-alt"},
		want:    testHome + "/.kimi-alt",
		wantOK:  true,
	}, {
		// One adapter's variable never selects another's root.
		name:    "a foreign adapter's variable is ignored",
		adapter: "claude",
		env:     map[string]string{"KIMI_CODE_HOME": "/srv/kimi"},
		want:    testHome + "/.claude",
		wantOK:  true,
	}, {
		name:    "codex names its root directly",
		adapter: "codex",
		env:     map[string]string{"CODEX_HOME": "/srv/codex"},
		want:    "/srv/codex",
		wantOK:  true,
	}, {
		// The suffix case: XDG_DATA_HOME names the parent OpenCode appends its
		// own name to, so the variable's value is never the root itself.
		name:    "opencode appends its own segment to the variable",
		adapter: "opencode",
		env:     map[string]string{"XDG_DATA_HOME": "/srv/share"},
		want:    "/srv/share/opencode",
		wantOK:  true,
	}, {
		name:    "opencode's default is the whole documented path",
		adapter: "opencode",
		want:    testHome + "/.local/share/opencode",
		wantOK:  true,
	}, {
		// A user-relative value expands before the suffix is applied, so both
		// halves land where the provider itself would have put them.
		name:    "a suffix applies after normalization",
		adapter: "opencode",
		env:     map[string]string{"XDG_DATA_HOME": "~/share"},
		want:    testHome + "/share/opencode",
		wantOK:  true,
	}, {
		name:    "pi's first variable names the sessions directory outright",
		adapter: "pi",
		env:     map[string]string{"PI_CODING_AGENT_SESSION_DIR": "/srv/pi-sessions"},
		want:    "/srv/pi-sessions",
		wantOK:  true,
	}, {
		name:    "pi's second variable names the directory sessions sit under",
		adapter: "pi",
		env:     map[string]string{"PI_CODING_AGENT_DIR": "/srv/pi"},
		want:    "/srv/pi/sessions",
		wantOK:  true,
	}, {
		name:    "pi's precedence prefers the session directory",
		adapter: "pi",
		env: map[string]string{
			"PI_CODING_AGENT_SESSION_DIR": "/srv/pi-sessions",
			"PI_CODING_AGENT_DIR":         "/srv/pi",
		},
		want:   "/srv/pi-sessions",
		wantOK: true,
	}, {
		name:    "pi's default is its documented sessions path",
		adapter: "pi",
		want:    testHome + "/.pi/agent/sessions",
		wantOK:  true,
	}, {
		name:    "grok carries its own variable and default",
		adapter: "grok",
		want:    testHome + "/.grok",
		wantOK:  true,
	}, {
		name:    "an adapter with no state-root knowledge is unavailable",
		adapter: "nothing-chartr-knows",
		wantOK:  false,
	}} {
		t.Run(tc.name, func(t *testing.T) {
			home := testHome
			if tc.noHome {
				home = ""
			}
			got, ok := StateRoot(tc.adapter, tc.env, tc.dir, home)
			if ok != tc.wantOK {
				t.Fatalf("ok = %v, want %v (got %q)", ok, tc.wantOK, got)
			}
			if ok && got != tc.want {
				t.Fatalf("StateRoot = %q, want %q", got, tc.want)
			}
		})
	}
}

// The isolation guarantee two parallel Claude accounts depend on: same adapter,
// same executable, same working directory, different configuration directories
// — and two roots that share nothing.
func TestStateRootSeparatesTwoConcurrentClaudeRoots(t *testing.T) {
	first, ok := StateRoot("claude", map[string]string{"CLAUDE_CONFIG_DIR": "~/.claude"}, testDir, testHome)
	if !ok {
		t.Fatal("first root unavailable")
	}
	second, ok := StateRoot("claude", map[string]string{"CLAUDE_CONFIG_DIR": "~/.claude2"}, testDir, testHome)
	if !ok {
		t.Fatal("second root unavailable")
	}
	if first == second {
		t.Fatalf("two custom roots collapsed onto %q", first)
	}
}

// A resolved root reaches a same-profile subprocess as an environment entry, and
// resolving that entry back through the table returns the root it came from —
// which is what makes it safe to keep the root instead of the environment it was
// read out of.
func TestStateRootEnvRoundTrips(t *testing.T) {
	root := testHome + "/.claude2"
	env := StateRootEnv("claude", root)
	if len(env) != 1 || env[0] != "CLAUDE_CONFIG_DIR="+root {
		t.Fatalf("StateRootEnv = %v, want claude's variable set to the root", env)
	}
	back, ok := StateRoot("claude", AllowedEnv(env, StateRootVars("claude")), testDir, testHome)
	if !ok || back != root {
		t.Fatalf("round trip = %q (%v), want %q", back, ok, root)
	}
}

// A variable that names a parent is handed the parent, so the subprocess
// resolves the same root the live tab did rather than one segment below it.
func TestStateRootEnvUnwindsASuffix(t *testing.T) {
	root := "/srv/share/opencode"
	env := StateRootEnv("opencode", root)
	if len(env) != 1 || env[0] != "XDG_DATA_HOME=/srv/share" {
		t.Fatalf("StateRootEnv = %v, want the parent XDG_DATA_HOME names", env)
	}
	back, ok := StateRoot("opencode", AllowedEnv(env, StateRootVars("opencode")), testDir, testHome)
	if !ok || back != root {
		t.Fatalf("round trip = %q (%v), want %q", back, ok, root)
	}
}

func TestStateRootEnvHasNothingToSay(t *testing.T) {
	if got := StateRootEnv("nothing-chartr-knows", "/srv/elsewhere"); got != nil {
		t.Errorf("StateRootEnv(unknown) = %v, want nil — no row, nothing to select", got)
	}
	if got := StateRootEnv("claude", ""); got != nil {
		t.Errorf("StateRootEnv with no root = %v, want nil rather than an invented one", got)
	}
	// A root that is not under a directory the provider would have appended its
	// own segment to cannot be said through that variable at all.
	if got := StateRootEnv("opencode", "/srv/share/somewhere-else"); got != nil {
		t.Errorf("StateRootEnv(opencode) = %v, want nil for a root the variable cannot express", got)
	}
}

func TestAllowedEnv(t *testing.T) {
	got := AllowedEnv([]string{
		"CLAUDE_CONFIG_DIR=/srv/one",
		"ANTHROPIC_API_KEY=sk-secret",
		"malformed-with-no-equals",
		"CLAUDE_CONFIG_DIR=/srv/two", // a repeat keeps its last value, as execve does
	}, []string{"CLAUDE_CONFIG_DIR"})
	if len(got) != 1 || got["CLAUDE_CONFIG_DIR"] != "/srv/two" {
		t.Fatalf("AllowedEnv = %v, want only the allowlisted variable at its last value", got)
	}
	if got := AllowedEnv([]string{"CLAUDE_CONFIG_DIR=/srv/one"}, nil); got != nil {
		t.Errorf("AllowedEnv with no allowlist = %v, want nothing", got)
	}
	if got := AllowedEnv(nil, []string{"CLAUDE_CONFIG_DIR"}); got != nil {
		t.Errorf("AllowedEnv of nothing = %v, want nothing", got)
	}
}

// Precedence is the table's order, not the map's: an adapter that grows a second
// variable must resolve the same way every time it is asked.
func TestStateRootHonoursVariableOrder(t *testing.T) {
	orig := stateRoots
	t.Cleanup(func() { stateRoots = orig })
	stateRoots = map[string]stateRootSpec{
		"two": {vars: []stateRootVar{{name: "FIRST"}, {name: "SECOND"}}, fallback: ".two"},
	}
	env := map[string]string{"FIRST": "/one", "SECOND": "/two"}
	for range 20 {
		if got, _ := StateRoot("two", env, testDir, testHome); got != "/one" {
			t.Fatalf("StateRoot = %q, want the first declared variable to win", got)
		}
	}
	// The second variable stands on its own when the first is unset.
	if got, _ := StateRoot("two", map[string]string{"SECOND": "/two"}, testDir, testHome); got != "/two" {
		t.Fatalf("StateRoot = %q, want /two", got)
	}
}
