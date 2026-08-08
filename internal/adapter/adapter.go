// Package adapter is the seam between a resolved role binding and the process
// chartr launches (ADR 0002). It turns a binding's args into the argv that
// starts the agent's TUI, and decides how the agent receives its opening
// line. There is no headless mode and no system-prompt flag — role and
// context always ride the payload body (spec, Sessions and adapters), so
// every agent is driven through one path.
//
// It models exactly one thing: prompt delivery. Everything else — model,
// permissions, sandbox — is operator-supplied args, since those flags vary
// per harness and this package has no business guessing at them.
//
// # Prompt delivery
//
// Every session opens with one line: read this payload file. How that line
// reaches the agent varies, so it's modelled explicitly:
//
//   - argv — the line is the CLI's trailing positional argument, already
//     submitted as the first turn. Nothing typed, so nothing can race
//     startup or get swallowed by paste heuristics. Preferred whenever an
//     agent offers it.
//   - flag — the line rides a named flag (`--prompt <line>`). Same
//     guarantees as argv, for CLIs whose positional argument means
//     something else.
//   - type — the line is typed into the live TUI and submitted with a
//     carriage return. The universal fallback: needs nothing but a PTY.
//
// chartr ships argv delivery for agents it knows first-hand, and types for
// everything else. An operator running any other harness can set
// `prompt = "argv"` or `prompt = "--prompt"` on the role binding to upgrade
// it without this package needing to know their CLI first.
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

// agents are CLIs whose command lines chartr knows first-hand. Both take
// their prompt as a trailing positional argument, already submitted
// (`claude [prompt]`, `codex [PROMPT]`) — not their headless flags, which
// chartr never uses.
//
// Kept deliberately short: an agent is listed only once its flags are
// verified, since a wrong guess is worse than the fallback — a bad flag
// refuses to start, typing just takes a beat longer. Anything absent gets
// generic below; operators can override either from config.
var agents = map[string]Agent{
	"claude": {Prompt: Argv()},
	"codex":  {Prompt: Argv()},
}

// generic drives any CLI without a table row — opencode, pi, kimi, whatever's
// next. Its opener is typed into the live TUI, needing nothing from the CLI
// beyond a PTY. A `prompt` on the binding upgrades it to argv or a flag once
// the operator knows which their harness wants.
var generic = Agent{Prompt: Type()}

// For returns the conventions for a binding's adapter name. An unrecognised
// name isn't an error — the operator may bind any CLI on PATH — so it falls
// back to generic rather than refusing the spawn.
func For(name string) Agent {
	if a, ok := agents[name]; ok {
		return a
	}
	return generic
}

// DeliveryFor resolves what actually delivers an agent's opener: the
// operator's override when set and valid, else the adapter's default. The
// settings surface reads this seam so it can show the resolved fact instead
// of re-deriving the table client-side and drifting from it.
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
	// Args is the argv the operator asked for, ahead of the opener — model
	// flag included, since this package doesn't know or care about flag shapes.
	Args []string
	// Prompt is the opening line. Empty launches the agent with nothing
	// said — what an operator opening a bare tab wants.
	Prompt string
	// Deliver overrides the adapter's default delivery. Unparseable values
	// fall back to default — Resolve already surfaced them as a config
	// warning, so a spawn shouldn't die on a typo.
	Deliver string
}

// Launch is the resolved command: the binary to exec, its argv (sans the binary
// name), and anything the caller must type in afterwards. The cwd, environment,
// and PTY are the caller's — this package only decides how the binding and the
// opener map onto one agent's command line.
type Launch struct {
	// Name is the binary to run, resolved against PATH by the caller — the
	// adapter's own name, since the binding's adapter field is the agent's CLI.
	Name string
	// Args is the argv after the binary that starts the agent's interactive TUI.
	Args []string
	// TypeIn is the opener to type into the live TUI once it's up, non-empty
	// only under ModeType. Empty means the opener already rode the argv.
	TypeIn string
}

// Command resolves a spawn onto one agent's command line. Order is fixed:
// prompt flag first, then the operator's args, then a positional prompt
// last — so the operator's args can never push the positional out of the
// final slot every CLI expects.
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

// Opener is the line a freshly launched session opens with: read the
// composed payload off disk (ADR 0005 — one injection path for every agent,
// payload is the whole brief). No line ending — submission by argv or typed
// carriage return is delivery's business. An agent that ignores it is
// visible in its own pane, surfaced rather than enforced.
func Opener(payloadPath string) string {
	return fmt.Sprintf("Read the file %s in full — it is your complete brief for this session — then begin.", payloadPath)
}
