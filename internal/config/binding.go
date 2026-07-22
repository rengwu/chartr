// Package config resolves the harness's layered configuration. Ticket 02 owns
// the role→agent bindings: what a role runs as, merged across three layers.
//
// A role binds to {adapter, model, args?} (ADR 0009). Three layers stack —
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

	"github.com/rengwu/wayfinder-harness/internal/model"
)

// WorkspaceConfigName is the committed workspace config file in a space's repo
// (ADR 0009): the shared, versioned, portable layer holding role bindings and
// map kinds. It lives under `.wayfinder-harness/` alongside the space's committed
// prompt overlays, so everything the harness commits into a space sits in one
// directory rather than a loose file one keystroke away from that directory's
// name. Config owns the filename because it owns the file's shape — the server
// reads and writes it through this package.
const WorkspaceConfigName = ".wayfinder-harness/config.toml"

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
// declares its kind (ADR 0007) — the harness never spawns on a heuristic.
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
var builtins = map[Role]Binding{
	RoleGrill:     {Adapter: "claude", Model: "opus"},
	RolePrototype: {Adapter: "claude", Model: "sonnet"},
	RoleResearch:  {Adapter: "claude", Model: "sonnet"},
	RoleImplement: {Adapter: "claude", Model: "sonnet"},
}

// Binding is a fully-resolved role→agent binding: which adapter to launch, on
// which model, with any extra args the adapter does not model (ADR 0009).
type Binding struct {
	Adapter string   `json:"adapter"`
	Model   string   `json:"model"`
	Args    []string `json:"args,omitempty"`
}

// Resolved is one role's effective binding, plus per-field provenance and
// whether the adapter's binary is actually on PATH.
type Resolved struct {
	Role Role `json:"role"`
	Binding
	// Each *From names the layer that last set that field, so a user override
	// of just `model` shows model←user, adapter←workspace, and nothing is a
	// surprise (story 39).
	AdapterFrom Layer `json:"adapterFrom"`
	ModelFrom   Layer `json:"modelFrom"`
	ArgsFrom    Layer `json:"argsFrom"`
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
	Model   *string  `toml:"model"`
	Args    []string `toml:"args"`
}

type workspaceFile struct {
	Roles map[string]rawBinding `toml:"roles"`
	Maps  map[string]rawMap     `toml:"maps"`
}

// rawMap is one map's committed harness config, keyed by map slug (ADR 0007).
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

	bindings := make([]Resolved, 0, len(Roles))
	for _, role := range Roles {
		r := Resolved{Role: role, Binding: builtins[role]}
		r.AdapterFrom, r.ModelFrom, r.ArgsFrom = LayerBuiltin, LayerBuiltin, LayerBuiltin

		if b, ok := ws[string(role)]; ok {
			apply(&r, b, LayerWorkspace)
		}
		if b, ok := us[string(role)]; ok {
			apply(&r, b, LayerUser)
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

	return Resolution{Bindings: bindings, Kinds: kinds, Warnings: warnings}
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
				"committed config declares map %q as kind %q, which the harness does not recognise (want planning or implementation); the map stays unclassified",
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
	if b.Model != nil {
		r.Model, r.ModelFrom = *b.Model, layer
	}
	if b.Args != nil {
		r.Args, r.ArgsFrom = b.Args, layer
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

// unknownRoleWarnings flags config that binds a name outside the closed role
// set — a typo or a role the harness does not offer — rather than honouring it
// silently.
func unknownRoleWarnings(roles map[string]rawBinding) []string {
	var out []string
	for name := range roles {
		if !isRole(name) {
			out = append(out, fmt.Sprintf("config binds unknown role %q, which the harness ignores", name))
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
