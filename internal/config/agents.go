package config

import (
	"fmt"
	"sort"
	"strings"

	"github.com/BurntSushi/toml"

	"github.com/rengwu/chartr/internal/adapter"
)

// The agent library: named launch specs the operator registers once and
// picks from at spawn time. It's the *only* execution config — no
// role→agent bindings, no committed execution layer, so nothing about how
// an agent runs can arrive by `git pull`.
//
// An agent is a complete, self-describing way to run a harness — binary,
// whatever flags it wants, and how it takes its opening prompt. Flags are
// an opaque list the operator types: chartr can't know what `--model
// sonnet` or `--dangerously-skip-permissions` mean to the harness defining
// them, and pretending to would make the library as agent-specific as ADR
// 0002 refused to be. The model is in that list like everything else — just
// a flag.
//
// The library is global and local: one `[agents.<name>]` table in the
// operator's own config, shared by every space, never committed. Which
// agents exist on this machine is a property of the machine, so nothing in
// a repository can hand a teammate a permission-skipping agent on `git
// pull`. An empty library is the starting state, not an error.

// Agent is one registered launch spec. Adapter is the only required field:
// everything a harness wants beyond its own name is Args and Env.
type Agent struct {
	Adapter string   `json:"adapter"`
	Args    []string `json:"args,omitempty"`
	// Env is the environment set on the launch, as `KEY=VALUE` entries —
	// the half of "how I run this harness" not expressible as a flag,
	// since the CLI reads it rather than parsing it (agentenv.go). As
	// opaque as Args, except a leading `~/` in a value expands on resolve.
	Env []string `json:"env,omitempty"`
	// Prompt is how the opener reaches this harness — `argv`, `type`, or a
	// flag name like `--prompt` (adapter.ParseDelivery). Empty leaves the
	// adapter's default in force.
	Prompt string `json:"prompt,omitempty"`
}

// ResolvedAgent is a registered agent as the surface renders it: the spec, its
// name, and whether its binary is actually on PATH — the absence badge answered
// once for the library.
type ResolvedAgent struct {
	Name string `json:"name"`
	Agent
	// LaunchEnv is the environment a spawn actually hands the process: the
	// embedded Agent.Env with tildes expanded. Kept apart deliberately —
	// Env is what the operator typed and what the editing surface must
	// round trip; saving the expanded form back would quietly replace
	// `~/.claude2` with one machine's absolute path.
	LaunchEnv []string `json:"launchEnv,omitempty"`
	Present   bool     `json:"present"`
	Missing   string   `json:"missing,omitempty"`
}

// Resolution is the resolved agent library for one machine plus any warnings —
// what every spawn surface consults to settle and refuse a launch (the library
// is global, so this is the same answer for every space and for none at all).
type Resolution struct {
	// Agents is the operator's registered library in name order.
	Agents []ResolvedAgent
	// Warnings are live problems worth surfacing — an agent with no adapter, an
	// unreadable prompt delivery. Surface, never enforce.
	Warnings []string
}

// agentsFile is the global half of the operator's config: the agent library,
// which — unlike everything else in the user layer — is not keyed by space.
type agentsFile struct {
	Agents map[string]rawAgent `toml:"agents"`
}

type rawAgent struct {
	Adapter string   `toml:"adapter"`
	Args    []string `toml:"args"`
	Env     []string `toml:"env"`
	Prompt  string   `toml:"prompt"`
}

// ResolveAgents reads the operator's agent library, in name order, with
// each agent's PATH presence probed. Takes the user config alone because
// the library is global — the same answer for every space, and for none.
//
// It never errors: a malformed file, an agent with no adapter, or an
// unreadable prompt delivery is dropped with a warning and the rest of the
// library stands.
func ResolveAgents(userTOML []byte, onPath func(string) bool) ([]ResolvedAgent, []string) {
	if onPath == nil {
		onPath = LookPath
	}
	raw, warnings := parseAgents(userTOML)
	home := homeDir()

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
		// Resolved here with tildes expanded, landing beside the spec
		// rather than inside it: launch and command preview read the
		// expanded form, the editing surface reads what was typed.
		env, envWarnings := resolveEnv(name, a.Env, home)
		warnings = append(warnings, envWarnings...)
		r := ResolvedAgent{
			Name:      name,
			Agent:     Agent{Adapter: a.Adapter, Args: a.Args, Env: a.Env, Prompt: a.Prompt},
			LaunchEnv: env,
		}
		r.Present = onPath(r.Adapter)
		if !r.Present {
			// The full path is named because it always works regardless of
			// how chartr was launched: exec.LookPath consults PATH only
			// for a bare name, and takes any name with a separator as the
			// binary itself.
			r.Missing = fmt.Sprintf("%q isn't on your PATH; install it, or give this agent the binary's full path", r.Adapter)
		}
		out = append(out, r)
	}
	return out, warnings
}

// parseAgents decodes the library out of the user config. A file too
// malformed to decode is already surfaced by the binding resolver reading
// the same bytes, so this one stays quiet rather than doubling the warning.
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

// knownAgentCLIs is the curated list the registration surface probes to
// *suggest* binaries the operator likely means — a hint, never a menu. The
// only fact asserted about any name here is "this binary is on your PATH",
// not agent-specific knowledge: chartr claims nothing about what these do
// or what flags they take, any binary can be registered whether or not it
// appears here, and no per-CLI UI is built on this list. It exists only so
// a fresh operator doesn't have to remember exact spellings.
var knownAgentCLIs = []string{
	"claude", "codex", "gemini", "cursor-agent", "aider",
	"goose", "amp", "opencode", "crush", "qwen",
}

// DetectAgents reports which known agent CLIs are resolvable on PATH, in
// curated order, so the surface can render them as helper text beneath the
// adapter input. Advisory only: an empty return means none of the known
// names are installed, not that nothing can be registered.
func DetectAgents(onPath func(string) bool) []string {
	if onPath == nil {
		onPath = LookPath
	}
	var found []string
	for _, name := range knownAgentCLIs {
		if onPath(name) {
			found = append(found, name)
		}
	}
	return found
}

// ValidAgentName reports whether a name is one the library can hold: letters,
// digits, hyphen, underscore — what reads well in a picker and needs no
// quoting as a TOML key. Refusing here keeps the writer from ever having to
// escape a name into a table header.
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

// ValidAgent checks a spec the surface is about to write: an adapter to
// launch, a prompt delivery the adapter seam can read, an environment
// shaped like an environment. Everything else — flags, model, what a
// variable means — is the operator's business and deliberately unchecked.
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
	return ValidAgentEnv(a.Env)
}

// decodeTOML decodes into v, reporting success. A file too malformed to
// decode declares nothing — already surfaced as malformed on resolve, and
// writers treat "declares nothing" as "safe to append a well-formed table to".
func decodeTOML(data []byte, v any) bool {
	_, err := toml.Decode(string(data), v)
	return err == nil
}
