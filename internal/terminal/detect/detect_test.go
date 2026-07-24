package detect

import (
	"strings"
	"testing"
	"testing/fstest"
)

// The shipped manifests, exercised as the table the ticket asks for: every rule
// each one ships, plus the absence case that carries no rule at all. The engine is
// a pure function from (agent, evidence) to a state, so this is the whole
// contract — everything above it (hysteresis, sampling) reads only this verdict.
func TestShippedManifestRules(t *testing.T) {
	e := Builtin()

	for _, agent := range []string{"claude", "codex", "grok", "kimi", "opencode", "pi"} {
		if !e.Known(agent) {
			t.Fatalf("no manifest shipped for %q", agent)
		}
	}

	// Representative reconstructed screens, structured like the real captures
	// (see .plan/agent-state-detection/assets and the recording-driven tests in the
	// terminal package): flat rules frame the panels, the spinner leads its own line.
	rule := strings.Repeat("─", 40)
	claudeBlocked := strings.Join([]string{
		rule,
		" Bash command",
		"   rm -rf /tmp/x",
		" Do you want to proceed?",
		" ❯ 1. Yes",
		"   2. Yes, and always allow",
		"   3. No",
		" Esc to cancel · Tab to amend · ctrl+e to explain",
	}, "\n")
	claudeIdle := strings.Join([]string{rule, "❯ count to 100 slowly", rule, "  ⏸ manual mode on"}, "\n")
	kimiWorking := strings.Join([]string{
		"● Ran a command",
		"⠦ thinking...",
		"  reasoning about the next step",
		"╭─────────────────────────────────────╮",
		"│ >                                   │",
		"╰─────────────────────────────────────╯",
		"K2.7 Coding thinking  ~",
	}, "\n")
	kimiBlocked := strings.Join([]string{
		"● Ran a command",
		"  Approved: Running: curl earlier", // the transcript echo the region must not read as live
		rule,
		"  ▶ Run this command?",
		"  cwd: /Users/rengwu",
		"  $ curl -sL https://example.com",
		"  ▶ 1. Approve once",
		"    2. Approve for this session",
		"    3. Reject",
		"  ↑/↓ select · 1/2/3/4 choose · ↵ confirm",
		rule,
		"K2.7 Coding thinking  ~",
	}, "\n")
	spinnerScreen := strings.Join([]string{"⠋ Working (12s · esc to interrupt)", "  building the answer"}, "\n")

	for _, tc := range []struct {
		name     string
		agent    string
		ev       Evidence
		want     string
		wantVeto bool
	}{
		// Claude: measured on this machine. A braille frame means generating, the ✳
		// marker means present-but-not-generating, and an empty title matches nothing.
		{"claude braille frame is working", "claude", Evidence{Title: "⠂ Count to 10 slowly"}, "working", false},
		{"claude other braille frame is working", "claude", Evidence{Title: "⠐ Claude Code"}, "working", false},
		{"claude ✳ is a positive idle", "claude", Evidence{Title: "✳ Count to 10 slowly"}, "idle", false},
		{"claude ✳ at the prompt is idle", "claude", Evidence{Title: "✳ Claude Code"}, "idle", false},
		{"claude empty title matches nothing", "claude", Evidence{Title: ""}, "", false},
		{"claude plain title matches nothing", "claude", Evidence{Title: "some shell title"}, "", false},

		// Codex: the blocked signal in the title is the one this ticket's Done-when
		// names, and it outranks working.
		{"codex Action Required is blocked", "codex", Evidence{Title: "Action Required — approve edit"}, "blocked", false},
		{"codex working title", "codex", Evidence{Title: "Working (50s • esc to interrupt)"}, "working", false},
		{"codex blocked wins over working", "codex", Evidence{Title: "Working — Action Required"}, "blocked", false},
		{"codex quiet title matches nothing", "codex", Evidence{Title: "codex"}, "", false},

		// Grok: the same blocked title, plus an OSC 9;4 progress pulse for working.
		{"grok Action Required is blocked", "grok", Evidence{Title: "Action Required"}, "blocked", false},
		{"grok active progress is working", "grok", Evidence{Progress: "4;1;40"}, "working", false},
		{"grok indeterminate progress is working", "grok", Evidence{Progress: "4;3;0"}, "working", false},
		{"grok cleared progress is a positive idle", "grok", Evidence{Progress: "4;0;0"}, "idle", false},
		{"grok blocked title outranks a working pulse", "grok",
			Evidence{Title: "Action Required", Progress: "4;1;40"}, "blocked", false},
		{"grok no evidence matches nothing", "grok", Evidence{}, "", false},

		// Claude's screen rules (ticket 02). Blocked comes off the screen because the
		// title cannot carry it (a permission prompt paints ✳, byte-identical to idle),
		// and it must win over that idle title.
		{"claude blocked from the permission dialog", "claude", Evidence{Screen: claudeBlocked}, "blocked", false},
		{"claude blocked wins over an idle ✳ title", "claude",
			Evidence{Title: "✳ working", Screen: claudeBlocked}, "blocked", false},
		{"claude idle from the ❯ prompt box", "claude", Evidence{Screen: claudeIdle}, "idle", false},
		{"claude working title still wins over an idle prompt box", "claude",
			Evidence{Title: "⠂ counting", Screen: claudeIdle}, "working", false},

		// Kimi reads entirely off the screen — it writes nothing to its title.
		{"kimi working from the ⠋ spinner", "kimi", Evidence{Screen: kimiWorking}, "working", false},
		{"kimi blocked from the approval panel", "kimi", Evidence{Screen: kimiBlocked}, "blocked", false},
		{"kimi status bar alone is not working", "kimi",
			Evidence{Screen: "K2.7 Coding thinking  ~\ncontext: 15%"}, "", false},
		{"kimi empty screen matches nothing", "kimi", Evidence{}, "", false},

		// opencode and pi have no title either; a braille spinner near the foot of the
		// screen reads working, its absence reads nothing (idle by absence).
		{"opencode working from a spinner", "opencode", Evidence{Screen: spinnerScreen}, "working", false},
		{"opencode idle has no spinner", "opencode", Evidence{Screen: "❯ ready\n(no spinner here)"}, "", false},
		{"pi working from a spinner", "pi", Evidence{Screen: spinnerScreen}, "working", false},
		{"pi idle has no spinner", "pi", Evidence{Screen: "❯ ready"}, "", false},

		// An agent with no manifest is nobody's business: the caller falls back to
		// the shell grammar.
		{"unknown agent matches nothing", "nosuch", Evidence{Title: "⠂ working"}, "", false},
	} {
		t.Run(tc.name, func(t *testing.T) {
			got := e.Evaluate(tc.agent, tc.ev)
			if got.State != tc.want || got.Veto != tc.wantVeto {
				t.Errorf("Evaluate(%q, %+v) = {state:%q veto:%v}, want {state:%q veto:%v}",
					tc.agent, tc.ev, got.State, got.Veto, tc.want, tc.wantVeto)
			}
		})
	}
}

// The OSC regions serve the retained title/progress values; the screen regions
// slice the reconstructed grid. A rule pointed at a region the engine does not know
// reads empty rather than throwing — which is what lets a manifest name a region at
// the one seam without any rule evaluation needing to change.
func TestRegionsAreTheOnlySeam(t *testing.T) {
	ev := Evidence{Title: "t", Progress: "p", Screen: "s"}
	for _, tc := range []struct{ region, want string }{
		{"osc_title", "t"},
		{"osc_progress", "p"},
		{"screen", "s"},
		{"whole_recent", "s"},
		{"no_such_region", ""},
	} {
		if got := region(tc.region, ev); got != tc.want {
			t.Errorf("region(%q) = %q, want %q", tc.region, got, tc.want)
		}
	}
}

// The screen regions (ticket 02) slice a reconstructed viewport structurally: the
// recent foot of the screen, the body below the last flat rule, and the body framed
// between the last two flat rules. The distinction that makes them reliable is that
// a flat rule (a run of U+2500) is not a cornered box border, so a rounded input box
// is never mistaken for framing.
func TestScreenRegions(t *testing.T) {
	rule := strings.Repeat("─", 20)
	screen := strings.Join([]string{
		"transcript line one",
		"transcript line two",
		rule,
		"Do you want to proceed?",
		"❯ 1. Yes",
		"3. No",
	}, "\n")

	t.Run("bottom_non_empty_lines keeps the last n non-blank rows in order", func(t *testing.T) {
		got := region("bottom_non_empty_lines(2)", Evidence{Screen: screen})
		if want := "❯ 1. Yes\n3. No"; got != want {
			t.Errorf("bottom_non_empty_lines(2) = %q, want %q", got, want)
		}
	})
	t.Run("bottom_non_empty_lines skips blank rows", func(t *testing.T) {
		got := region("bottom_non_empty_lines(1)", Evidence{Screen: "a\n\n\n  \nb\n\n"})
		if got != "b" {
			t.Errorf("bottom_non_empty_lines(1) over trailing blanks = %q, want %q", got, "b")
		}
	})
	t.Run("after_last_horizontal_rule isolates the dialog below the rule", func(t *testing.T) {
		got := region("after_last_horizontal_rule", Evidence{Screen: screen})
		if want := "Do you want to proceed?\n❯ 1. Yes\n3. No"; got != want {
			t.Errorf("after_last_horizontal_rule = %q, want %q", got, want)
		}
	})
	t.Run("after_last_horizontal_rule is empty without a rule", func(t *testing.T) {
		if got := region("after_last_horizontal_rule", Evidence{Screen: "no rules here\njust prose"}); got != "" {
			t.Errorf("after_last_horizontal_rule with no rule = %q, want empty", got)
		}
	})
	t.Run("prompt_box_body reads the body between the last two rules", func(t *testing.T) {
		box := strings.Join([]string{"scrolled off", rule, "❯ type here", rule, " status bar"}, "\n")
		if got := region("prompt_box_body", Evidence{Screen: box}); got != "❯ type here" {
			t.Errorf("prompt_box_body = %q, want %q", got, "❯ type here")
		}
	})
	t.Run("prompt_box_body is empty with only one rule", func(t *testing.T) {
		if got := region("prompt_box_body", Evidence{Screen: screen}); got != "" {
			t.Errorf("prompt_box_body with one rule = %q, want empty", got)
		}
	})
	t.Run("a cornered box border is not a flat rule", func(t *testing.T) {
		// The rounded box kimi draws its idle input in must not read as framing, or
		// its body would be mistaken for a prompt/approval panel.
		rounded := "╭" + strings.Repeat("─", 18) + "╮\n│ >                │\n╰" + strings.Repeat("─", 18) + "╯"
		if got := region("prompt_box_body", Evidence{Screen: rounded}); got != "" {
			t.Errorf("prompt_box_body over a rounded box = %q, want empty (corners are not flat rules)", got)
		}
	})
	t.Run("a short dash run in prose is not a rule", func(t *testing.T) {
		if got := region("after_last_horizontal_rule", Evidence{Screen: "a ── b\nc"}); got != "" {
			t.Errorf("a two-dash run counted as a rule: after = %q, want empty", got)
		}
	})
}

// parseRegion splits an optional integer argument off a region name, so a manifest
// can write bottom_non_empty_lines(6) and the engine reads the 6.
func TestParseRegion(t *testing.T) {
	for _, tc := range []struct {
		in   string
		fn   string
		narg int
	}{
		{"screen", "screen", 0},
		{"bottom_non_empty_lines(6)", "bottom_non_empty_lines", 6},
		{"bottom_non_empty_lines( 3 )", "bottom_non_empty_lines", 3},
		{"after_last_horizontal_rule", "after_last_horizontal_rule", 0},
		{"weird(", "weird(", 0},
		{"weird(x)", "weird", 0},
	} {
		fn, narg := parseRegion(tc.in)
		if fn != tc.fn || narg != tc.narg {
			t.Errorf("parseRegion(%q) = (%q, %d), want (%q, %d)", tc.in, fn, narg, tc.fn, tc.narg)
		}
	}
}

// synthetic builds an engine from an inline manifest — the seam for the cases the
// shipped data deliberately does not carry (a veto rule, an exercise of every
// matcher), so testing them never means inventing agent behaviour.
func synthetic(t *testing.T, toml string) *Engine {
	t.Helper()
	e, err := New(fstest.MapFS{"m/x.toml": &fstest.MapFile{Data: []byte(toml)}}, "m")
	if err != nil {
		t.Fatalf("building synthetic engine: %v", err)
	}
	return e
}

// skip_state_update is the veto: the highest-priority rule that matches wins, and
// when the winner is a veto rule the sample yields no state at all — not idle, not
// working. It is how a transcript viewer or model picker showing stale prompt text
// is kept from being read as blocked.
func TestVetoRuleSuppressesTheSample(t *testing.T) {
	e := synthetic(t, `
agent = "vetoer"

[[rule]]
id = "viewer-open"
priority = 200
region = "osc_title"
contains = ["transcript"]
skip_state_update = true

[[rule]]
id = "blocked"
state = "blocked"
priority = 100
region = "osc_title"
contains = ["Approve?"]
`)

	// The blocked rule alone fires normally.
	if got := e.Evaluate("vetoer", Evidence{Title: "Approve?"}); got.State != "blocked" || got.Veto {
		t.Errorf("blocked evidence = %+v, want state blocked", got)
	}
	// With the viewer open, the same stale text vetoes the sample instead.
	if got := e.Evaluate("vetoer", Evidence{Title: "transcript — Approve?"}); !got.Veto || got.State != "" {
		t.Errorf("veto evidence = %+v, want a veto with no state", got)
	}
}

// Every matcher the ticket names, and the AND between them: a rule narrows as
// fields are added, and a rule with no matchers at all never matches — so a stray
// empty rule cannot swallow every sample.
func TestMatchers(t *testing.T) {
	e := synthetic(t, `
agent = "m"

[[rule]]
id = "contains-all"
state = "working"
priority = 100
region = "osc_title"
contains = ["alpha", "beta"]

[[rule]]
id = "any-of"
state = "blocked"
priority = 90
region = "osc_title"
any = ["yes", "sure"]

[[rule]]
id = "all-but-not"
state = "idle"
priority = 80
region = "osc_title"
all = ["keep"]
not = ["drop"]

[[rule]]
id = "regex"
state = "working"
priority = 70
region = "osc_progress"
regex = ['^4;[13]']

[[rule]]
id = "line-regex"
state = "blocked"
priority = 60
region = "screen"
line_regex = ['^\s*> Approve']

[[rule]]
id = "no-matchers"
state = "working"
priority = 10
region = "osc_title"
`)

	for _, tc := range []struct {
		name string
		ev   Evidence
		want string
	}{
		{"contains needs every substring", Evidence{Title: "alpha beta"}, "working"},
		{"contains fails when one is missing", Evidence{Title: "alpha only"}, ""},
		{"any needs just one", Evidence{Title: "sure thing"}, "blocked"},
		{"any fails when none are present", Evidence{Title: "nope"}, ""},
		{"all with not passes", Evidence{Title: "keep this"}, "idle"},
		{"not vetoes the rule", Evidence{Title: "keep but drop"}, ""},
		{"regex on the progress region", Evidence{Progress: "4;3;0"}, "working"},
		{"regex that does not match", Evidence{Progress: "4;0;0"}, ""},
		// line_regex is anchored per line: it matches a row of a multi-line region,
		// and its ^ does not straddle the line break the way a plain regex would.
		{"line_regex matches one row of a screen", Evidence{Screen: "some output\n  > Approve this?\ntail"}, "blocked"},
		{"line_regex does not match mid-line", Evidence{Screen: "tail > Approve"}, ""},
		{"a rule with no matchers never fires", Evidence{Title: "anything at all"}, ""},
	} {
		t.Run(tc.name, func(t *testing.T) {
			if got := e.Evaluate("m", tc.ev); got.State != tc.want {
				t.Errorf("Evaluate(%+v) = %q, want %q", tc.ev, got.State, tc.want)
			}
		})
	}
}

// Highest priority wins, regardless of the order rules are declared in.
func TestHighestPriorityWins(t *testing.T) {
	e := synthetic(t, `
agent = "p"

[[rule]]
id = "low"
state = "idle"
priority = 1
region = "osc_title"
contains = ["x"]

[[rule]]
id = "high"
state = "blocked"
priority = 99
region = "osc_title"
contains = ["x"]
`)
	if got := e.Evaluate("p", Evidence{Title: "x"}); got.State != "blocked" {
		t.Errorf("Evaluate = %q, want the priority-99 rule's state %q", got.State, "blocked")
	}
}

// Identification reads the whole foreground process group's argv, not just its
// leader's name, and scores so a generic runtime never wins. That is what makes a
// node-launched claude — whose process name is `node` and whose only trace of
// `claude` is an argv path — resolve to claude.
func TestIdentifyScoresPastGenericRuntimes(t *testing.T) {
	e := Builtin()
	for _, tc := range []struct {
		name  string
		names []string
		want  string
	}{
		{"a node-launched claude", []string{"node", "/opt/homebrew/bin/claude", "--resume"}, "claude"},
		{"a shell-script agent on PATH", []string{"/bin/sh", "/tmp/bin/claude"}, "claude"},
		{"an alias from the manifest", []string{"node", "/usr/local/bin/claude-code"}, "claude"},
		{"a bare agent binary", []string{"codex"}, "codex"},
		{"grok", []string{"/usr/local/bin/grok", "chat"}, "grok"},
		{"a plain shell is no agent", []string{"-zsh"}, ""},
		{"a runtime with no agent in sight is no agent", []string{"node", "server.js"}, ""},
		{"an ordinary command is no agent", []string{"sleep", "5"}, ""},
		{"an empty group is no agent", nil, ""},
		// A generic runtime never wins even when it leads the group and an agent
		// trails it — the ordering that broke naive comm-only detection.
		{"python leading an agent", []string{"python3", "-m", "grok"}, "grok"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			if got := e.Identify(tc.names); got != tc.want {
				t.Errorf("Identify(%v) = %q, want %q", tc.names, got, tc.want)
			}
		})
	}
}

// A manifest whose regex will not compile is a build-time defect in shipped data;
// New reports it rather than silently dropping the rule.
func TestBadManifestIsReported(t *testing.T) {
	_, err := New(fstest.MapFS{"m/x.toml": &fstest.MapFile{Data: []byte(`
agent = "bad"
[[rule]]
id = "r"
state = "working"
region = "osc_title"
regex = ["("]
`)}}, "m")
	if err == nil {
		t.Fatal("New accepted a manifest with an uncompilable regex")
	}
}
