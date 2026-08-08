package config

import (
	_ "embed"
	"fmt"
	"sort"
	"time"

	"github.com/BurntSushi/toml"
)

// The shipped machine-wide notification defaults, defined with the config
// that resolves them and re-exported by terminal beside the clock, so
// there's one value for file, snapshot and machine to share.
const (
	DefaultNotifyAfter  = 60 * time.Second
	DefaultNotifySettle = 10 * time.Second
)

// ScaffoldNotifyTOML is the self-documenting notify.toml the Settings
// surface creates. Every key is commented, so writing it changes nothing
// while putting the file's ownership and exact defaults in the operator's
// editor.
//
//go:embed notify.scaffold.toml
var ScaffoldNotifyTOML []byte

// NotifyPrefs is the resolved notification rule for this machine. Unlike
// TerminalPrefs, every field has a concrete default: absence means 60
// seconds, 10 seconds and enabled. rawNotify keeps all three untyped so a
// wrong TOML type on one key can be dropped independently, not take good
// neighbours down with it.
type NotifyPrefs struct {
	After   time.Duration
	Settle  time.Duration
	Enabled bool
}

type rawNotify struct {
	After   interface{} `toml:"after"`
	Settle  interface{} `toml:"settle"`
	Enabled interface{} `toml:"enabled"`
}

// ResolveNotifyPrefs parses one per-machine notify.toml. Missing keys keep
// their defaults; an invalid key keeps only its own default and warns by
// name. Syntax too malformed to identify a key falls back wholesale with
// one file-level warning — nothing here can make the cockpit fail to build
// its model.
func ResolveNotifyPrefs(tomlBytes []byte) (NotifyPrefs, []string) {
	prefs := NotifyPrefs{
		After:   DefaultNotifyAfter,
		Settle:  DefaultNotifySettle,
		Enabled: true,
	}
	if len(tomlBytes) == 0 {
		return prefs, nil
	}

	var raw rawNotify
	md, err := toml.Decode(string(tomlBytes), &raw)
	if err != nil {
		return prefs, []string{fmt.Sprintf(
			"notify.toml could not be read: %s; notification defaults stand", err)}
	}

	var warnings []string
	prefs.After = resolveNotifyDuration("after", raw.After, DefaultNotifyAfter, &warnings)
	prefs.Settle = resolveNotifyDuration("settle", raw.Settle, DefaultNotifySettle, &warnings)
	prefs.Enabled = resolveNotifyEnabled(raw.Enabled, &warnings)
	for _, key := range md.Undecoded() {
		warnings = append(warnings, fmt.Sprintf(
			"notify.toml has an unknown setting %q; it is ignored", key.String()))
	}
	sort.Strings(warnings)
	return prefs, warnings
}

func resolveNotifyDuration(key string, raw interface{}, fallback time.Duration, warnings *[]string) time.Duration {
	if raw == nil {
		return fallback
	}
	value, ok := raw.(string)
	if !ok {
		*warnings = append(*warnings, fmt.Sprintf(
			"notify.toml key %q must be a positive duration string such as %q; the default %q stands",
			key, fallback, fallback))
		return fallback
	}
	d, err := time.ParseDuration(value)
	if err != nil || d <= 0 {
		*warnings = append(*warnings, fmt.Sprintf(
			"notify.toml key %q value %q is not a positive duration; the default %q stands",
			key, value, fallback))
		return fallback
	}
	return d
}

func resolveNotifyEnabled(raw interface{}, warnings *[]string) bool {
	if raw == nil {
		return true
	}
	enabled, ok := raw.(bool)
	if !ok {
		*warnings = append(*warnings,
			`notify.toml key "enabled" must be true or false; the default true stands`)
		return true
	}
	return enabled
}
