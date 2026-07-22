package adapter

import (
	"reflect"
	"strings"
	"testing"
)

// A known agent takes its opener on the argv, so nothing is typed — the delivery
// that cannot race a TUI's startup or be eaten by its paste handling.
func TestKnownAgentCarriesTheOpenerOnArgv(t *testing.T) {
	got := Command(Spawn{Adapter: "claude", Args: []string{"--model", "opus"}, Prompt: "read this"})
	want := Launch{Name: "claude", Args: []string{"--model", "opus", "read this"}}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("claude launch = %+v, want %+v", got, want)
	}
	if got.TypeIn != "" {
		t.Errorf("argv delivery still asks for keystrokes: %q", got.TypeIn)
	}
}

// A positional opener stays last however many args the binding adds, because a
// CLI reading a trailing positional would otherwise swallow one of them instead.
func TestPositionalOpenerStaysLast(t *testing.T) {
	got := Command(Spawn{
		Adapter: "claude",
		Args:    []string{"--model", "sonnet", "--add-dir", "/tmp"},
		Prompt:  "read this",
	})
	want := []string{"--model", "sonnet", "--add-dir", "/tmp", "read this"}
	if !reflect.DeepEqual(got.Args, want) {
		t.Errorf("argv = %q, want %q", got.Args, want)
	}
}

// An agent this package ships no row for still spawns: its flags pass through
// untouched and the opener is typed in, which needs nothing of the CLI.
func TestUnknownAgentFallsBackToTyping(t *testing.T) {
	got := Command(Spawn{Adapter: "some-new-harness", Args: []string{"-m", "big"}, Prompt: "read this"})
	if want := []string{"-m", "big"}; !reflect.DeepEqual(got.Args, want) {
		t.Errorf("argv = %q, want %q — an unknown agent must not be handed a positional it may not take", got.Args, want)
	}
	if got.TypeIn != "read this" {
		t.Errorf("TypeIn = %q, want the opener typed in", got.TypeIn)
	}
}

// The operator's hatch: any harness is upgraded to argv or a flag from config,
// without this package learning about it first.
func TestBindingOverridesDelivery(t *testing.T) {
	argv := Command(Spawn{Adapter: "opencode", Args: []string{"-m", "big"}, Prompt: "read this", Deliver: "argv"})
	if want := []string{"-m", "big", "read this"}; !reflect.DeepEqual(argv.Args, want) {
		t.Errorf("argv override = %q, want %q", argv.Args, want)
	}
	if argv.TypeIn != "" {
		t.Errorf("argv override still types %q", argv.TypeIn)
	}

	flag := Command(Spawn{Adapter: "opencode", Args: []string{"-c", "x"}, Prompt: "read this", Deliver: "--prompt"})
	if want := []string{"--prompt", "read this", "-c", "x"}; !reflect.DeepEqual(flag.Args, want) {
		t.Errorf("flag override = %q, want %q", flag.Args, want)
	}
	if flag.TypeIn != "" {
		t.Errorf("flag override still types %q", flag.TypeIn)
	}

	typed := Command(Spawn{Adapter: "claude", Args: []string{"--model", "opus"}, Prompt: "read this", Deliver: "type"})
	if want := []string{"--model", "opus"}; !reflect.DeepEqual(typed.Args, want) {
		t.Errorf("type override = %q, want %q", typed.Args, want)
	}
	if typed.TypeIn != "read this" {
		t.Errorf("type override did not ask for keystrokes")
	}
}

// A delivery the parser cannot read never reaches the command line: the agent's
// default stands, so a typo in config costs a warning rather than a CLI that
// refuses to start.
func TestUnreadableDeliveryFallsBackToTheDefault(t *testing.T) {
	got := Command(Spawn{Adapter: "claude", Args: []string{"--model", "opus"}, Prompt: "read this", Deliver: "argvv"})
	if want := []string{"--model", "opus", "read this"}; !reflect.DeepEqual(got.Args, want) {
		t.Errorf("argv = %q, want claude's default %q", got.Args, want)
	}
}

func TestParseDelivery(t *testing.T) {
	for _, tc := range []struct {
		in   string
		want Delivery
		err  bool
	}{
		{in: "", want: Delivery{}},
		{in: "argv", want: Argv()},
		{in: "type", want: Type()},
		{in: " argv ", want: Argv()},
		{in: "--prompt", want: Flag("--prompt")},
		{in: "-p", want: Flag("-p")},
		{in: "stdin", err: true},
		{in: "prompt", err: true},
	} {
		got, err := ParseDelivery(tc.in)
		if tc.err {
			if err == nil {
				t.Errorf("ParseDelivery(%q) = %+v, want an error naming the vocabulary", tc.in, got)
			}
			continue
		}
		if err != nil {
			t.Errorf("ParseDelivery(%q): %v", tc.in, err)
			continue
		}
		if got != tc.want {
			t.Errorf("ParseDelivery(%q) = %+v, want %+v", tc.in, got, tc.want)
		}
		if tc.in != "" && got.String() != strings.TrimSpace(tc.in) {
			t.Errorf("ParseDelivery(%q).String() = %q — a delivery must render back as the value that set it", tc.in, got.String())
		}
	}
}

// The opener carries no line ending of its own: submitting it is delivery's job,
// and a trailing newline was what left it sitting unsent in a TUI's composer.
func TestOpenerCarriesNoLineEnding(t *testing.T) {
	got := Opener("/tmp/payload.md")
	if strings.ContainsAny(got, "\r\n") {
		t.Errorf("opener carries a line ending: %q", got)
	}
	if !strings.Contains(got, "/tmp/payload.md") {
		t.Errorf("opener does not name the payload: %q", got)
	}
}

// An empty opener is a launch with nothing said — no stray positional, nothing
// typed.
func TestNoOpenerSaysNothing(t *testing.T) {
	got := Command(Spawn{Adapter: "claude", Args: []string{"--model", "opus"}})
	if want := []string{"--model", "opus"}; !reflect.DeepEqual(got.Args, want) {
		t.Errorf("argv = %q, want %q", got.Args, want)
	}
	if got.TypeIn != "" {
		t.Errorf("TypeIn = %q, want nothing typed", got.TypeIn)
	}
}
