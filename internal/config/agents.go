package config

import (
	"fmt"
	"sort"
	"strings"

	"github.com/BurntSushi/toml"

	"github.com/rengwu/chartr/internal/adapter"
)

// The agent library: named launch specs the operator registers once and assigns
// to roles (ADR 0009 as amended).
//
// An agent is a *complete, self-describing way to run a harness* — the binary,
// whatever flags that harness wants, and how it takes its opening prompt. Nothing
// here knows anything about any particular CLI: flags are an opaque list the
// operator types, because the chartr cannot know what `--model sonnet`,
// `--dangerously-skip-permissions`, or `--sandbox danger-full-access` mean to the
// harness that defines them, and pretending to would make the library exactly as
// agent-specific as ADR 0002 refused to be. The model is in that list like
// everything else: it is a flag, and it was never anything more.
//
// The library is **global and local**: one `[agents.<name>]` table set in the
// operator's own config, shared by every space, and never committed. Assignment
// stays per space — a role names an agent, and that name is the whole binding.
// The split is deliberate: which agents exist on this machine is a property of the
// machine (its PATH, its logins, how much rope its operator wants), while which
// role runs which agent is a property of the work. Nothing in a repository can
// therefore hand a teammate a permission-skipping agent on `git pull`.
//
// A role that names no agent resolves exactly as it always has, field by field
// across the three layers. The library is an addition, never a migration.

// Agent is one registered launch spec. Adapter is the only required field:
// everything a harness wants beyond its own name is Args.
type Agent struct {
	Adapter string   `json:"adapter"`
	Args    []string `json:"args,omitempty"`
	// Prompt is how the opener reaches this harness — `argv`, `type`, or a flag
	// name like `--prompt` (adapter.ParseDelivery). Empty leaves the adapter's own
	// default in force.
	Prompt string `json:"prompt,omitempty"`
}

// ResolvedAgent is a registered agent as the surface renders it: the spec, its
// name, and whether its binary is actually on PATH — the same absence badge a
// binding carries, answered once for the library rather than per role.
type ResolvedAgent struct {
	Name string `json:"name"`
	Agent
	Present bool   `json:"present"`
	Missing string `json:"missing,omitempty"`
}

// agentsFile is the global half of the operator's config: the agent library,
// which — unlike everything else in the user layer — is not keyed by space.
type agentsFile struct {
	Agents map[string]rawAgent `toml:"agents"`
}

type rawAgent struct {
	Adapter string   `toml:"adapter"`
	Args    []string `toml:"args"`
	Prompt  string   `toml:"prompt"`
	// Model is retired — it is a flag, and it lives in Args. Still decoded so an
	// agent registered before that change is told it stopped taking effect rather
	// than quietly launching without it.
	Model string `toml:"model"`
}

// ResolveAgents reads the operator's agent library, in name order, with each
// agent's PATH presence probed. It takes the user config alone because the
// library is global: it is the same answer for every space, and for no space at
// all, which is what the settings surface's global scope asks for.
//
// It never errors. A malformed file, an agent with no adapter, or an unreadable
// prompt delivery is dropped with a warning and the rest of the library stands —
// one bad table must not cost the operator every agent they registered.
func ResolveAgents(userTOML []byte, onPath func(string) bool) ([]ResolvedAgent, []string) {
	if onPath == nil {
		onPath = LookPath
	}
	raw, warnings := parseAgents(userTOML)

	names := make([]string, 0, len(raw))
	for name := range raw {
		names = append(names, name)
	}
	sort.Strings(names) // the library reads in a stable order, never map order

	out := make([]ResolvedAgent, 0, len(names))
	for _, name := range names {
		a := raw[name]
		if strings.TrimSpace(a.Adapter) == "" {
			warnings = append(warnings, fmt.Sprintf(
				"agent %q names no adapter, so there is nothing to launch; it is ignored", name))
			continue
		}
		if _, err := adapter.ParseDelivery(a.Prompt); err != nil {
			warnings = append(warnings, fmt.Sprintf(
				"agent %q has an unreadable prompt delivery: %s; the adapter's default stands", name, err))
			a.Prompt = ""
		}
		if a.Model != "" {
			warnings = append(warnings, fmt.Sprintf(
				"agent %q sets model = %q, which the chartr no longer reads; move it into args (for example args = [\"--model\", %q])",
				name, a.Model, a.Model))
		}
		r := ResolvedAgent{
			Name:  name,
			Agent: Agent{Adapter: a.Adapter, Args: a.Args, Prompt: a.Prompt},
		}
		r.Present = onPath(r.Adapter)
		if !r.Present {
			r.Missing = fmt.Sprintf("%q isn't on your PATH; install it or point this agent at a binary that is", r.Adapter)
		}
		out = append(out, r)
	}
	return out, warnings
}

// parseAgents decodes the library out of the user config. A file too malformed to
// decode is already surfaced by the binding resolver reading the same bytes, so
// this one stays quiet about it rather than doubling the warning.
func parseAgents(userTOML []byte) (map[string]rawAgent, []string) {
	if len(userTOML) == 0 {
		return nil, nil
	}
	var af agentsFile
	if _, err := toml.Decode(string(userTOML), &af); err != nil {
		return nil, nil
	}
	var warnings []string
	for name := range af.Agents {
		if err := ValidAgentName(name); err != nil {
			warnings = append(warnings, fmt.Sprintf("agent %q is ignored: %s", name, err))
			delete(af.Agents, name)
		}
	}
	sort.Strings(warnings)
	return af.Agents, warnings
}

// ValidAgentName reports whether a name is one the library can hold. The rule is
// the intersection of what reads well in a picker and what needs no quoting as a
// TOML key: letters, digits, hyphen, underscore. Refusing here is what keeps the
// writer from ever having to escape a name into a table header.
func ValidAgentName(name string) error {
	if name == "" {
		return fmt.Errorf("an agent needs a name")
	}
	if len(name) > 64 {
		return fmt.Errorf("agent names are at most 64 characters")
	}
	for _, r := range name {
		switch {
		case r >= 'a' && r <= 'z', r >= 'A' && r <= 'Z', r >= '0' && r <= '9', r == '-', r == '_':
		default:
			return fmt.Errorf("agent names take letters, digits, hyphen and underscore only (%q is not one)", string(r))
		}
	}
	return nil
}

// ValidAgent checks a spec the surface is about to write: an adapter to launch,
// and a prompt delivery the adapter seam can read. Everything else — which flags
// a harness wants, whether it has a model at all — is the operator's business and
// deliberately unchecked. A flag this package has never heard of is the normal
// case, not an error.
func ValidAgent(a Agent) error {
	if strings.TrimSpace(a.Adapter) == "" {
		return fmt.Errorf("an agent needs an adapter — the CLI to launch")
	}
	if strings.ContainsAny(a.Adapter, " \t") {
		return fmt.Errorf("the adapter is one binary name; put flags in args instead")
	}
	if _, err := adapter.ParseDelivery(a.Prompt); err != nil {
		return err
	}
	return nil
}

// decodeTOML decodes into v, reporting success. A file too malformed to decode
// declares nothing — it is already surfaced as malformed on resolve, and the
// writers treat "declares nothing" as "safe to append a well-formed table to".
func decodeTOML(data []byte, v any) bool {
	_, err := toml.Decode(string(data), v)
	return err == nil
}

// sortStrings is sort.Strings, named here so the writers do not each import sort.
func sortStrings(s []string) { sort.Strings(s) }
