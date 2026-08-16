package config

import (
	_ "embed"
	"fmt"
	"sort"

	"github.com/BurntSushi/toml"
)

// Auto-title is machine-wide server behaviour — it spends the operator's own model
// subscriptions in the background — so its one setting lives beside notify.toml as
// a per-machine file, never committed and never scoped to a space. Only a single
// on/off exists; there is nothing else to tune from the file.

// ScaffoldAutoTitleTOML is the self-documenting autotitle.toml the Settings surface
// creates. The one key is commented, so writing it changes nothing while putting
// the file's ownership and default in the operator's editor.
//
//go:embed autotitle.scaffold.toml
var ScaffoldAutoTitleTOML []byte

// DefaultAutoTitleEnabled is the shipped default: on. A fresh machine generates tab
// titles until the operator turns them off, which is the point of a toggle that
// still defaults to the feature working.
const DefaultAutoTitleEnabled = true

// AutoTitlePrefs is the resolved auto-title rule for this machine. Like NotifyPrefs
// and unlike TerminalPrefs, its field has a concrete default: absence means on.
type AutoTitlePrefs struct {
	Enabled bool
}

type rawAutoTitle struct {
	Enabled interface{} `toml:"enabled"`
}

// ResolveAutoTitlePrefs parses one per-machine autotitle.toml. A missing file, a
// missing key, or a wrong type all leave the default in force; only a wrong type
// warns, by name. Nothing here can make the cockpit fail to build its model.
func ResolveAutoTitlePrefs(tomlBytes []byte) (AutoTitlePrefs, []string) {
	prefs := AutoTitlePrefs{Enabled: DefaultAutoTitleEnabled}
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
	prefs.Enabled = resolveAutoTitleEnabled(raw.Enabled, &warnings)
	for _, key := range md.Undecoded() {
		warnings = append(warnings, fmt.Sprintf(
			"autotitle.toml has an unknown setting %q; it is ignored", key.String()))
	}
	sort.Strings(warnings)
	return prefs, warnings
}

func resolveAutoTitleEnabled(raw interface{}, warnings *[]string) bool {
	if raw == nil {
		return DefaultAutoTitleEnabled
	}
	enabled, ok := raw.(bool)
	if !ok {
		*warnings = append(*warnings,
			`autotitle.toml key "enabled" must be true or false; the default true stands`)
		return DefaultAutoTitleEnabled
	}
	return enabled
}
