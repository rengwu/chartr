// Package adapter is the agent-agnostic seam between a resolved role binding and
// the process the chartr actually launches (ADR 0002, as amended by the
// interactive-spawn decision and again by prompt delivery). An adapter turns a
// binding's args into the argv that starts *that* agent's own interactive TUI,
// and decides how that agent receives its opening line. There is no headless mode
// and no system-prompt flag — role and context ride the payload body uniformly
// (spec, Sessions and adapters), so every agent is driven through one path.
//
// It models exactly one thing about a CLI: prompt delivery. Everything else an
// agent wants — its model, its permissions, its sandbox — is args the operator
// writes, because those are the flags no two harnesses agree on and the ones this
// package would have to guess at. Delivery is modelled only because the chartr
// itself has to *do* something differently depending on the answer.
//
// # Prompt delivery
//
// Every session opens with one line: read this payload file. How that line
// *reaches* the agent is the one thing agents genuinely disagree about, so it is
// modelled rather than assumed:
//
//   - argv — the line is the CLI's trailing positional argument. The agent starts
//     with it already submitted as the first turn. Nothing is typed, so nothing
//     can race the TUI's startup or be swallowed by its paste heuristics. This is
//     the delivery to prefer whenever an agent offers it.
//   - flag — the line rides a named flag (`--prompt <line>`). Same guarantees as
//     argv, for CLIs whose positional argument means something else.
//   - type — the line is typed into the live TUI as keystrokes, submitted with a
//     carriage return. The universal fallback: it needs nothing of the CLI but a
//     PTY, and it is what an agent the chartr has never heard of gets.
//
// The chartr ships argv delivery for the agents whose command lines it knows
// first-hand, and types for everything else. An operator running any other
// harness upgrades it in one line of config — `prompt = "argv"` or
// `prompt = "--prompt"` on the role binding — without this package learning about
// their CLI first. That hatch is the point: the closed set here is a convenience,
// never a gate on which agents the chartr can drive.
package adapter

import (
	"fmt"
	"strings"
)

// Mode is how an agent receives its opening line.
type Mode string

const (
	// ModeArgv passes the line as the command's trailing positional argument.
	ModeArgv Mode = "argv"
	// ModeFlag passes the line as the value of a named flag.
	ModeFlag Mode = "flag"
	// ModeType types the line into the live TUI as keystrokes.
	ModeType Mode = "type"
)

// Delivery is a resolved prompt-delivery choice: a mode, plus the flag name when
// the mode is ModeFlag.
type Delivery struct {
	Mode Mode
	Flag string
}

// Argv, Type and Flag build the three deliveries.
func Argv() Delivery            { return Delivery{Mode: ModeArgv} }
func Type() Delivery            { return Delivery{Mode: ModeType} }
func Flag(f string) Delivery    { return Delivery{Mode: ModeFlag, Flag: f} }
func (d Delivery) isZero() bool { return d.Mode == "" }

// ParseDelivery reads a binding's `prompt` value — the operator's hatch for an
// agent this package ships no knowledge of. Three shapes, chosen so the common
// case is one word:
//
//	prompt = "argv"       # trailing positional argument
//	prompt = "type"       # keystrokes into the live TUI
//	prompt = "--prompt"   # anything starting with "-" is a flag name
//
// An empty value means "unset": the agent's built-in default stands. Anything
// else is an error the caller surfaces as a config warning rather than honouring
// blind — a mistyped mode must never silently become a flag the CLI rejects.
func ParseDelivery(s string) (Delivery, error) {
	s = strings.TrimSpace(s)
	switch {
	case s == "":
		return Delivery{}, nil
	case s == string(ModeArgv):
		return Argv(), nil
	case s == string(ModeType):
		return Type(), nil
	case strings.HasPrefix(s, "-"):
		return Flag(s), nil
	}
	return Delivery{}, fmt.Errorf(
		"unknown prompt delivery %q; want \"argv\", \"type\", or a flag name like \"--prompt\"", s)
}

// String renders a delivery back into the config value that produced it, so the
// effective-config surface shows the operator their own vocabulary.
func (d Delivery) String() string {
	if d.Mode == ModeFlag {
		return d.Flag
	}
	return string(d.Mode)
}

// Agent is one CLI's command-line conventions — which is to say how it takes an
// opening prompt, the only convention this package models. It is pure data: a new
// agent is a row in the table below, never a change to the spawn path.
type Agent struct {
	// Prompt is how this CLI receives the opening line.
	Prompt Delivery
}

// agents are the CLIs whose command lines the chartr knows first-hand. Both take
// their prompt as a trailing positional argument that starts the interactive TUI
// with it already submitted (`claude [prompt]`, `codex [PROMPT]`) — not to be
// confused with their headless flags, which the chartr never uses.
//
// This table is deliberately short. An agent is only listed once its flags have
// actually been checked, because a wrong guess here is worse than the fallback: a
// bad flag refuses to start, while typing merely takes a beat longer. Everything
// absent gets generic below, and any operator can override either from config.
var agents = map[string]Agent{
	"claude": {Prompt: Argv()},
	"codex":  {Prompt: Argv()},
}

// generic drives any CLI the operator binds that this package ships no row for —
// opencode, pi, kimi, or whatever comes next. Its opener is typed into the live
// TUI, which needs no cooperation from the CLI beyond running in a PTY. A `prompt`
// on the binding upgrades it to argv or a flag the moment the operator knows which
// their harness wants.
var generic = Agent{Prompt: Type()}

// For returns the conventions for a binding's adapter name. An unrecognised name
// is not an error — the operator may bind any CLI on their PATH (the args hatch
// and the agent-agnostic stance) — so it falls back to generic rather than
// refusing the spawn.
func For(name string) Agent {
	if a, ok := agents[name]; ok {
		return a
	}
	return generic
}

// DeliveryFor resolves what will actually deliver an agent's opener: the
// operator's override where they set a readable one, and the adapter's own
// default otherwise. It is the seam the settings surface reads, so the library
// can render "how this harness is told what to do" as a resolved fact rather than
// re-deriving the table client-side and drifting from it.
func DeliveryFor(adapterName, override string) Delivery {
	if d, err := ParseDelivery(override); err == nil && !d.isZero() {
		return d
	}
	return For(adapterName).Prompt
}

// Spawn is everything a launch needs: the resolved binding, the opening line, and
// the operator's delivery override (the binding's raw `prompt` value, empty when
// unset).
type Spawn struct {
	Adapter string
	// Args is the whole argv the operator asked for, ahead of the opener — the
	// model flag among them, since a model is just another flag this package has
	// no business knowing the shape of.
	Args []string
	// Prompt is the opening line — the read-this-file opener. Empty launches the
	// agent with nothing said, which is what an operator opening a bare tab wants.
	Prompt string
	// Deliver overrides the adapter's default delivery. Unparseable values fall
	// back to the default: Resolve has already surfaced them as a config warning,
	// and a spawn must not die on a typo.
	Deliver string
}

// Launch is the resolved command: the binary to exec, its argv (sans the binary
// name), and anything the caller must type in afterwards. The cwd, environment,
// and PTY are the caller's — this package only decides how the binding and the
// opener map onto one agent's command line.
type Launch struct {
	// Name is the binary to run, resolved against PATH by the caller. It is the
	// adapter's own name: the binding's adapter field is the agent's CLI.
	Name string
	// Args is the argv after the binary that starts the agent's interactive TUI.
	Args []string
	// TypeIn is the opener to type into the live TUI once it is up — non-empty
	// only under ModeType. When it is empty the opener already rides the argv, and
	// the caller types nothing at all.
	TypeIn string
}

// Command resolves a spawn onto one agent's command line. Ordering is fixed and
// deliberate: a prompt flag first, then the operator's own args, and last of all a
// positional prompt — so the operator's args can never push the positional out of
// final place, where every CLI expects it.
func Command(s Spawn) Launch {
	a := For(s.Adapter)
	if d, err := ParseDelivery(s.Deliver); err == nil && !d.isZero() {
		a.Prompt = d
	}

	var argv []string
	if s.Prompt != "" && a.Prompt.Mode == ModeFlag {
		argv = append(argv, a.Prompt.Flag, s.Prompt)
	}
	argv = append(argv, s.Args...)

	l := Launch{Name: s.Adapter, Args: argv}
	if s.Prompt == "" {
		return l
	}
	switch a.Prompt.Mode {
	case ModeArgv:
		l.Args = append(l.Args, s.Prompt)
	case ModeType:
		l.TypeIn = s.Prompt
	}
	return l
}

// Opener is the one line a freshly launched session opens with: a plain
// instruction to read the composed payload off disk (ADR 0005 — injection is one
// path for every agent, the payload is the whole brief). It carries no line
// ending: whether it is submitted by argv or by a typed carriage return is
// delivery's business, not the line's. An agent that ignores it is visible in its
// own pane, surfaced rather than enforced.
func Opener(payloadPath string) string {
	return fmt.Sprintf("Read the file %s in full — it is your complete brief for this session — then begin.", payloadPath)
}
