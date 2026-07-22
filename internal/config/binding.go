// Package config resolves the chartr's layered configuration. Ticket 02 owns
// the role→agent bindings: what a role runs as, merged across three layers.
//
// A role binds to {adapter, args?, prompt?} (ADR 0009). Three layers stack —
// shipped built-in defaults ‹ committed workspace config ‹ local user config —
// and merge field by field, with the user layer winning: the reconciling rule
// is that content the project ships wins (prompts, a later ticket) while
// execution choices the operator makes win (bindings, here). Because a user
// override may set one field and inherit the rest, the effective binding
// records where each field came from so silent inheritance stays visible
// (story 39).
package config

import (
	"fmt"
	"os/exec"
	"sort"

	"github.com/BurntSushi/toml"

	"github.com/rengwu/chartr/internal/adapter"
	"github.com/rengwu/chartr/internal/model"
)

// WorkspaceConfigName is the committed workspace config file in a space's repo
// (ADR 0009): the shared, versioned, portable layer holding role bindings and
// map kinds. It lives under `.chartr/` alongside the space's committed
// prompt overlays, so everything the chartr commits into a space sits in one
// directory rather than a loose file one keystroke away from that directory's
// name. Config owns the filename because it owns the file's shape — the server
// reads and writes it through this package.
const WorkspaceConfigName = ".chartr/config.toml"

// Role is one of the closed set of things a session is spawned to do (ADR
// 0002). The set is fixed here; config that names anything outside it is
// surfaced as a warning, never silently honoured.
type Role string

const (
	RoleGrill     Role = "grill"
	RolePrototype Role = "prototype"
	RoleResearch  Role = "research"
	RoleImplement Role = "implement"
)

// Roles is the closed role set in a stable display order.
var Roles = []Role{RoleGrill, RolePrototype, RoleResearch, RoleImplement}

// RolesForKind returns the roles a map of the given kind offers a session for
// (spec, Sessions and roles): a planning map grills, prototypes, and researches;
// an implementation map implements. An unclassified map (any other
// value, including the empty string) offers none, so it stays inert until a human
// declares its kind (ADR 0007) — the chartr never spawns on a heuristic.
func RolesForKind(kind string) []Role {
	switch kind {
	case model.KindPlanning:
		return []Role{RoleGrill, RolePrototype, RoleResearch}
	case model.KindImplementation:
		return []Role{RoleImplement}
	default:
		return nil
	}
}

// RoleIsAFK reports whether a session in this role runs unattended — the operator
// kicks it off and walks away — as opposed to a human-in-the-loop role that is
// *supposed* to sit idle waiting on its human. Only this split earns a session the
// "quiet" hint: an AFK session silent past a threshold may be stuck, while an idle
// HITL session is simply waiting and must show nothing (spec, Sessions and
// adapters; stories 34–35).
//
// `grill` is the human-in-the-loop role — a grilling session is a dialogue, and
// story 35 names it as the one that must never wear a quiet badge. Every other
// role (prototype, research, implement) runs to completion on its own, so
// silence from one is a signal worth surfacing. An unrecognised name is treated as
// AFK: a stray session shows the hint rather than swallowing a possible stall.
func RoleIsAFK(role string) bool {
	return role != string(RoleGrill)
}

// KindOffersRole reports whether a map of this kind offers the named role — the
// gate the spawn path checks so an unclassified map (which offers nothing) and a
// role that belongs to the other lifecycle are both refused.
func KindOffersRole(kind, role string) bool {
	for _, r := range RolesForKind(kind) {
		if string(r) == role {
			return true
		}
	}
	return false
}

func isRole(name string) bool {
	for _, r := range Roles {
		if string(r) == name {
			return true
		}
	}
	return false
}

// Layer identifies where a binding field was resolved from, so field-level
// inheritance is rendered rather than guessed.
type Layer string

const (
	LayerBuiltin   Layer = "built-in"
	LayerWorkspace Layer = "workspace"
	LayerUser      Layer = "user"
)

// builtins are the shipped default bindings — the starting point every layer
// above may override.
// The model rides `args` like every other flag: which flag carries a model — and
// whether a harness has the concept at all — is exactly the per-CLI knowledge the
// chartr refuses to hold (ADR 0002).
var builtins = map[Role]Binding{
	RoleGrill:     {Adapter: "claude", Args: []string{"--model", "opus"}},
	RolePrototype: {Adapter: "claude", Args: []string{"--model", "sonnet"}},
	RoleResearch:  {Adapter: "claude", Args: []string{"--model", "sonnet"}},
	RoleImplement: {Adapter: "claude", Args: []string{"--model", "sonnet"}},
}

// Binding is a fully-resolved role→agent binding: which adapter to launch, with
// which args (ADR 0009), and how that agent takes its opening prompt. There is no
// model field — a model is a flag, and it lives in Args with every other flag.
type Binding struct {
	Adapter string   `json:"adapter"`
	Args    []string `json:"args,omitempty"`
	// Prompt overrides how the opener reaches this agent — `argv`, `type`, or a
	// flag name like `--prompt` (see adapter.ParseDelivery). Empty means the
	// adapter's own default stands, which is what nearly every binding wants; it
	// is the hatch that lets an operator drive a harness the chartr ships no
	// knowledge of without waiting for an adapter row.
	Prompt string `json:"prompt,omitempty"`
}

// Resolved is one role's effective binding, plus per-field provenance and
// whether the adapter's binary is actually on PATH.
type Resolved struct {
	Role Role `json:"role"`
	Binding
	// Each *From names the layer that last set that field, so a user override
	// of just `args` shows args←user, adapter←workspace, and nothing is a
	// surprise (story 39).
	AdapterFrom Layer `json:"adapterFrom"`
	ArgsFrom    Layer `json:"argsFrom"`
	PromptFrom  Layer `json:"promptFrom"`
	// Agent is the registered agent this role is assigned to, empty when the role
	// is bound field by field the older way. When it is set and registered, the
	// agent *is* the binding — every field above came from it wholesale — so the
	// surface shows one name instead of four values with four provenances.
	Agent string `json:"agent,omitempty"`
	// AgentMissing is set when Agent names nothing in the library: the assignment
	// is shown as it stands, with the fields beneath it saying what actually runs,
	// rather than the name silently disappearing.
	AgentMissing string `json:"agentMissing,omitempty"`
	// Present is whether the adapter binary was found on PATH at resolve time.
	// Missing carries the absence badge — empty when Present is true.
	Present bool   `json:"present"`
	Missing string `json:"missing,omitempty"`
}

// Resolution is the whole outcome for one space: its effective bindings in role
// order, the committed map-kind declarations (slug → kind, ADR 0007), any
// warnings to surface (an unknown role, an unrecognised kind, a malformed
// config file).
type Resolution struct {
	Bindings []Resolved
	Kinds    map[string]string
	// Agents is the operator's registered agent library, in name order. It is
	// global rather than per space (agents.go), and is carried on the resolution so
	// a caller reads bindings and the library they may name out of one pass.
	Agents   []ResolvedAgent
	Warnings []string
}

// Input is everything Resolve needs for one space. The TOML byte slices are the
// raw file contents (nil or empty when the file is absent); SpacePath is the
// key into the user layer, which is keyed by space; OnPath probes the PATH and
// defaults to exec.LookPath when nil.
type Input struct {
	WorkspaceTOML []byte
	UserTOML      []byte
	SpacePath     string
	OnPath        func(binary string) bool
}

// rawBinding is one layer's view of a binding: a pointer per field so an unset
// field (nil) inherits the layer below while a set field overrides it. Args
// follows the same rule via nil-vs-non-nil, so an explicit `args = []` clears
// inherited args.
type rawBinding struct {
	Adapter *string  `toml:"adapter"`
	Args    []string `toml:"args"`
	// Model is no longer a binding field — it is a flag like any other, and lives
	// in Args. It is still decoded so a config written before that change is
	// *told* it stopped taking effect rather than silently launching a different
	// model (retiredModelWarnings).
	Model  *string `toml:"model"`
	Prompt *string `toml:"prompt"`
	// Agent assigns this role to a registered agent by name, which supplies the
	// whole binding. It is the field the surface writes now; the four above remain
	// for a role bound the older way, and for anyone who prefers the file.
	Agent *string `toml:"agent"`
}

// setsFields reports whether this layer sets any of the four execution fields —
// what an agent assignment would override wholesale, and therefore what is worth
// telling the operator about when both are present in one table.
func (b rawBinding) setsFields() bool {
	return b.Adapter != nil || b.Args != nil || b.Prompt != nil
}

type workspaceFile struct {
	Roles map[string]rawBinding `toml:"roles"`
	Maps  map[string]rawMap     `toml:"maps"`
}

// rawMap is one map's committed chartr config, keyed by map slug (ADR 0007).
// Kind is the only field today; the table exists so per-map committed state has
// a home to grow into without another top-level key.
type rawMap struct {
	Kind string `toml:"kind"`
}

type userFile struct {
	// Spaces is keyed by absolute space path — the user layer is keyed by
	// space (ADR 0009). Quoted TOML keys carry the path verbatim.
	Spaces map[string]userSpace `toml:"spaces"`
}

type userSpace struct {
	Roles map[string]rawBinding `toml:"roles"`
}

// Resolve merges the three layers for one space and reports the effective
// bindings and warnings. It never errors: a malformed config file degrades to a
// warning and the layers below it, because adoption is never gated on config
// lint.
func Resolve(in Input) Resolution {
	onPath := in.OnPath
	if onPath == nil {
		onPath = LookPath
	}

	var warnings []string

	wf := parseWorkspace(in.WorkspaceTOML, &warnings)
	ws := wf.Roles
	kinds := resolveKinds(wf.Maps, &warnings)
	us := parseUser(in.UserTOML, in.SpacePath, &warnings)

	warnings = append(warnings, unknownRoleWarnings(ws)...)
	warnings = append(warnings, unknownRoleWarnings(us)...)
	warnings = append(warnings, retiredModelWarnings(LayerWorkspace, ws)...)
	warnings = append(warnings, retiredModelWarnings(LayerUser, us)...)

	// The agent library is global and local (agents.go), so it resolves once for
	// every role here, out of the same user bytes.
	library, libWarnings := ResolveAgents(in.UserTOML, onPath)
	warnings = append(warnings, libWarnings...)

	bindings := make([]Resolved, 0, len(Roles))
	for _, role := range Roles {
		r := Resolved{Role: role, Binding: builtins[role]}
		r.AdapterFrom, r.ArgsFrom, r.PromptFrom = LayerBuiltin, LayerBuiltin, LayerBuiltin

		if b, ok := ws[string(role)]; ok {
			apply(&r, b, LayerWorkspace)
		}
		if b, ok := us[string(role)]; ok {
			apply(&r, b, LayerUser)
		}

		// An assignment supersedes the field merge entirely: a registered agent is
		// one indivisible way to run a harness, so taking three of its fields and a
		// fourth from somewhere else would launch something nobody registered.
		if r.Agent != "" {
			warnings = append(warnings, assignmentWarnings(role, r.Agent, ws, us)...)
			if a, ok := findAgent(library, r.Agent); ok {
				r.Binding = Binding{Adapter: a.Adapter, Args: a.Args, Prompt: a.Prompt}
				// Every field now comes from one place — the library, which lives in the
				// user layer — so the provenance says so rather than pointing at three.
				r.AdapterFrom, r.ArgsFrom, r.PromptFrom = LayerUser, LayerUser, LayerUser
			} else {
				// A dangling assignment falls back to the fields beneath it. The role is
				// still spawnable, and the surface shows the name that resolved to
				// nothing rather than quietly dropping it.
				r.AgentMissing = fmt.Sprintf("no agent named %q is registered; this role falls back to its own fields", r.Agent)
				warnings = append(warnings, fmt.Sprintf(
					"role %s is assigned to agent %q, which is not registered; it falls back to its own fields", role, r.Agent))
			}
		}

		// A prompt delivery the adapter seam cannot read is dropped to a warning and
		// the agent's default stands, in the same spirit as an unrecognised map kind:
		// config that says something unreadable never silently changes how a session
		// is launched, and never blocks one either. The library validates its own on
		// the way through, so this catches the field-bound form.
		if r.Prompt != "" {
			if _, err := adapter.ParseDelivery(r.Prompt); err != nil {
				warnings = append(warnings, fmt.Sprintf(
					"%s config binds role %s with an unreadable prompt delivery: %s; the agent's default stands",
					r.PromptFrom, role, err,
				))
				r.Prompt, r.PromptFrom = "", LayerBuiltin
			}
		}

		r.Present = onPath(r.Adapter)
		if !r.Present {
			r.Missing = fmt.Sprintf(
				"%q isn't on your PATH (%s → %s config); install it or set a local override",
				r.Adapter, role, r.AdapterFrom,
			)
		}
		bindings = append(bindings, r)
	}

	return Resolution{Bindings: bindings, Kinds: kinds, Agents: library, Warnings: warnings}
}

// findAgent picks one agent out of the resolved library by name.
func findAgent(library []ResolvedAgent, name string) (ResolvedAgent, bool) {
	for _, a := range library {
		if a.Name == name {
			return a, true
		}
	}
	return ResolvedAgent{}, false
}

// assignmentWarnings flags a role table that both assigns an agent and sets the
// fields the agent supplies. The agent wins, and the operator is told which lines
// stopped mattering — a value silently ignored is the kind of thing that costs an
// afternoon when a session launches on the wrong model.
func assignmentWarnings(role Role, agent string, layers ...map[string]rawBinding) []string {
	names := []Layer{LayerWorkspace, LayerUser}
	var out []string
	for i, roles := range layers {
		if b, ok := roles[string(role)]; ok && b.setsFields() {
			out = append(out, fmt.Sprintf(
				"role %s is assigned to agent %q, so the adapter/model/args/prompt set in %s config no longer apply",
				role, agent, names[i],
			))
		}
	}
	return out
}

// resolveKinds turns the committed [maps.<slug>] tables into a slug → kind map,
// keeping only recognised kinds (ADR 0007: kind is planning or implementation).
// A table with no kind declares nothing, and an unrecognised kind is surfaced as
// a warning and dropped — either way the map stays unclassified and inert rather
// than being refused. Kind is committed-layer only: teammates must agree on it
// (story 15), so the user layer never declares it.
func resolveKinds(maps map[string]rawMap, warnings *[]string) map[string]string {
	if len(maps) == 0 {
		return nil
	}
	slugs := make([]string, 0, len(maps))
	for slug := range maps {
		slugs = append(slugs, slug)
	}
	sort.Strings(slugs) // deterministic warning order

	kinds := make(map[string]string)
	for _, slug := range slugs {
		k := maps[slug].Kind
		if k == "" {
			continue
		}
		if !model.ValidKind(k) {
			*warnings = append(*warnings, fmt.Sprintf(
				"committed config declares map %q as kind %q, which the chartr does not recognise (want planning or implementation); the map stays unclassified",
				slug, k,
			))
			continue
		}
		kinds[slug] = k
	}
	return kinds
}

// apply overrides r's fields with those set in b (nil fields inherit), tagging
// each overridden field with the layer it came from.
func apply(r *Resolved, b rawBinding, layer Layer) {
	if b.Adapter != nil {
		r.Adapter, r.AdapterFrom = *b.Adapter, layer
	}
	if b.Args != nil {
		r.Args, r.ArgsFrom = b.Args, layer
	}
	if b.Prompt != nil {
		r.Prompt, r.PromptFrom = *b.Prompt, layer
	}
	if b.Agent != nil {
		r.Agent = *b.Agent
	}
}

func parseWorkspace(data []byte, warnings *[]string) workspaceFile {
	if len(data) == 0 {
		return workspaceFile{}
	}
	var wf workspaceFile
	if _, err := toml.Decode(string(data), &wf); err != nil {
		*warnings = append(*warnings, "committed workspace config is malformed and was ignored: "+err.Error())
		return workspaceFile{}
	}
	return wf
}

func parseUser(data []byte, spacePath string, warnings *[]string) map[string]rawBinding {
	if len(data) == 0 {
		return nil
	}
	var uf userFile
	if _, err := toml.Decode(string(data), &uf); err != nil {
		*warnings = append(*warnings, "local user config is malformed and was ignored: "+err.Error())
		return nil
	}
	return uf.Spaces[spacePath].Roles
}

// retiredModelWarnings flags a binding that still sets `model`. The field was
// retired — a model is a flag, and flags live in `args` — and a key that quietly
// stopped taking effect is exactly the kind of thing that costs an afternoon when
// a session turns out to be running the wrong model. Surfaced, never honoured and
// never guessed at: the chartr will not invent the flag name a harness wants.
func retiredModelWarnings(layer Layer, roles map[string]rawBinding) []string {
	var out []string
	for name, b := range roles {
		if b.Model != nil && isRole(name) {
			out = append(out, fmt.Sprintf(
				"role %s sets model = %q in %s config, which the chartr no longer reads; move it into args (for example args = [\"--model\", %q])",
				name, *b.Model, layer, *b.Model,
			))
		}
	}
	sort.Strings(out)
	return out
}

// unknownRoleWarnings flags config that binds a name outside the closed role
// set — a typo or a role the chartr does not offer — rather than honouring it
// silently.
func unknownRoleWarnings(roles map[string]rawBinding) []string {
	var out []string
	for name := range roles {
		if !isRole(name) {
			out = append(out, fmt.Sprintf("config binds unknown role %q, which the chartr ignores", name))
		}
	}
	sort.Strings(out)
	return out
}

// LookPath reports whether a binary is resolvable on the current PATH.
func LookPath(binary string) bool {
	_, err := exec.LookPath(binary)
	return err == nil
}
