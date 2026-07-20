// Package adapter is the agent-agnostic seam between a resolved role binding and
// the process the harness actually launches (ADR 0002, as amended by the
// interactive-spawn decision). An adapter turns a binding's {model, args} into
// the argv that starts *that* agent's own interactive TUI; the harness then runs
// it in a PTY and types a one-line opener in. There is no headless mode and no
// system-prompt flag — role and context ride the payload body uniformly (spec,
// Sessions and adapters), so every agent is driven through one path.
//
// The set is small and closed to the adapters the bindings name (claude, codex);
// anything else resolves to a generic adapter that passes the model through a
// conventional `--model` flag. Getting an agent's flags exactly right is the
// adapter's job and grows per real agent — this package is the place that
// knowledge lives, not the spawn path.
package adapter

import "fmt"

// Launch is the resolved command an adapter produces: the binary to exec and its
// argv (sans the binary name). The cwd, environment, and PTY are the caller's —
// the adapter only decides how the binding maps onto this one agent's command
// line.
type Launch struct {
	// Name is the binary to run, resolved against PATH by the caller. It is the
	// adapter's own name: the binding's adapter field is the agent's CLI.
	Name string
	// Args is the argv after the binary — the model flag and the binding's extra
	// args — that starts the agent's interactive TUI.
	Args []string
}

// Adapter maps a role binding onto one agent's command line. It never launches
// anything: the harness owns the PTY and the process (so liveness, stop, and the
// opener stay uniform across agents). Command is pure so the spawn path — and its
// tests — can assert exactly what will run without a process.
type Adapter interface {
	// Command builds the launch for this agent from the model and the binding's
	// portable extra args.
	Command(model string, args []string) Launch
}

// For returns the adapter for a binding's adapter name. An unrecognised name is
// not an error — the operator may bind any CLI on their PATH (the args hatch and
// agent-agnostic stance) — so it falls back to the generic `--model` adapter
// rather than refusing the spawn. Getting a specific agent's flags right is an
// additive per-agent improvement, never a gate.
func For(name string) Adapter {
	if a, ok := builtins[name]; ok {
		return a
	}
	return generic{name}
}

// modelFlagAdapter is the common shape: the model rides a single flag, then the
// binding's extra args follow. Both agents the harness ships bindings for take
// `--model`; the type keeps the flag one edit away from a per-agent override.
type modelFlagAdapter struct {
	name string
	flag string
}

func (a modelFlagAdapter) Command(model string, args []string) Launch {
	var argv []string
	if model != "" {
		argv = append(argv, a.flag, model)
	}
	argv = append(argv, args...)
	return Launch{Name: a.name, Args: argv}
}

// generic drives any CLI the operator binds that the harness ships no specific
// adapter for: model through `--model`, args appended. It forfeits nothing the
// binding does not already forfeit — the args hatch is where an agent that wants
// a different flag gets it.
type generic struct{ name string }

func (g generic) Command(model string, args []string) Launch {
	return modelFlagAdapter{name: g.name, flag: "--model"}.Command(model, args)
}

// builtins are the adapters for the agents the shipped bindings name. Both take
// `--model`; the map is where a future agent with different conventions is
// taught, without touching the spawn path.
var builtins = map[string]Adapter{
	"claude": modelFlagAdapter{name: "claude", flag: "--model"},
	"codex":  modelFlagAdapter{name: "codex", flag: "--model"},
}

// Opener is the one line typed into a freshly launched session's TUI: a plain
// instruction to read the composed payload off disk (ADR 0005 — injection is one
// path for every agent, the payload is the whole brief). It carries a trailing
// newline so it submits as the session's first turn; an agent that ignores it is
// visible in its own pane, surfaced rather than enforced.
func Opener(payloadPath string) string {
	return fmt.Sprintf("Read the file %s in full — it is your complete brief for this session — then begin.\n", payloadPath)
}
