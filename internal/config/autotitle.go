package config

import (
	_ "embed"
	"fmt"
	"sort"

	"github.com/BurntSushi/toml"
)

// Auto-title is machine-wide server behaviour — it spends the operator's own model
// subscriptions in the background — so its settings live beside notify.toml as a
// per-machine file, never committed and never scoped to a space. There are two, and
// they are the two halves of the cost ladder: whether the feature runs at all, and
// whether it is allowed to spend.

// ScaffoldAutoTitleTOML is the self-documenting autotitle.toml the Settings surface
// creates. Every key is commented, so writing it changes nothing while putting the
// file's ownership and defaults in the operator's editor.
//
//go:embed autotitle.scaffold.toml
var ScaffoldAutoTitleTOML []byte

// DefaultAutoTitleEnabled is the shipped default: on. A fresh machine generates tab
// titles until the operator turns them off, which is the point of a toggle that
// still defaults to the feature working.
const DefaultAutoTitleEnabled = true

// DefaultAutoTitleNativeOnly is the shipped default: off, so a fresh machine still
// generates a title for a session whose agent wrote none. Turning it on keeps the
// free half of the ladder — the agent's own title, observed and displayed — and
// drops the paid half.
const DefaultAutoTitleNativeOnly = false

// AutoTitlePrefs is the resolved auto-title rule for this machine. Like NotifyPrefs
// and unlike TerminalPrefs, its fields have concrete defaults: absence means on,
// and free-and-paid rather than free-only.
type AutoTitlePrefs struct {
	Enabled bool
	// NativeOnly keeps titling to what the agent's own session already carries: a
	// tab is titled where its harness wrote a title and left plain where it did
	// not, and no model is ever spent. It says nothing when Enabled is false —
	// the title consumer reads and spends nothing either way.
	NativeOnly bool
}

type rawAutoTitle struct {
	Enabled    interface{} `toml:"enabled"`
	NativeOnly interface{} `toml:"native_only"`
}

// ResolveAutoTitlePrefs parses one per-machine autotitle.toml. A missing file, a
// missing key, or a wrong type all leave the default in force; only a wrong type
// warns, by name. Nothing here can make the cockpit fail to build its model.
func ResolveAutoTitlePrefs(tomlBytes []byte) (AutoTitlePrefs, []string) {
	prefs := AutoTitlePrefs{
		Enabled:    DefaultAutoTitleEnabled,
		NativeOnly: DefaultAutoTitleNativeOnly,
	}
	if len(tomlBytes) == 0 {
		return prefs, nil
	}

	var raw rawAutoTitle
	md, err := toml.Decode(string(tomlBytes), &raw)
	if err != nil {
		return prefs, []string{fmt.Sprintf(
			"autotitle.toml could not be read: %s; the default stands", err)}
	}

	var warnings []string
	prefs.Enabled = resolveAutoTitleBool(
		raw.Enabled, "enabled", DefaultAutoTitleEnabled, &warnings)
	prefs.NativeOnly = resolveAutoTitleBool(
		raw.NativeOnly, "native_only", DefaultAutoTitleNativeOnly, &warnings)
	for _, key := range md.Undecoded() {
		warnings = append(warnings, fmt.Sprintf(
			"autotitle.toml has an unknown setting %q; it is ignored", key.String()))
	}
	sort.Strings(warnings)
	return prefs, warnings
}

// resolveAutoTitleBool reads one of the file's two flags. Every rule this file has
// is the same for both: a missing key leaves the shipped default in force, and a
// value that is not a boolean warns by name and leaves it in force too.
func resolveAutoTitleBool(raw interface{}, key string, def bool, warnings *[]string) bool {
	if raw == nil {
		return def
	}
	value, ok := raw.(bool)
	if !ok {
		*warnings = append(*warnings, fmt.Sprintf(
			"autotitle.toml key %q must be true or false; the default %t stands", key, def))
		return def
	}
	return value
}
