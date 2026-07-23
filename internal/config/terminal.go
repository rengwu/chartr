package config

import (
	"fmt"
	"regexp"
	"sort"
	"strings"

	"github.com/BurntSushi/toml"
)

// Terminal customization: a single per-machine `terminal.toml`, beside the agent
// library in the operator's own config, that fully customizes every terminal
// island. It is the single source of truth — never committed, never per space —
// and it follows the same philosophy as the agent library: a pure parse produces
// a resolved value plus warnings, the terminal always runs, and a bad value falls
// back to a default with a warning rather than breaking the terminal.
//
// This is Seam 1 of the effort (spec, Testing Decisions): file contents in, a
// resolved TerminalPrefs plus warnings out. The client resolve seam in
// `web/src/lib/tokens.ts` turns these prefs into the concrete xterm options and
// theme the island consumes; an unset colour slot resolves against the live
// design tokens over there, which is why the colours here stay strings the parse
// only validates, never token-resolves. Ticket 01 lands the spine — font family,
// font size, and background/foreground colour — and every later ticket widens the
// struct without changing its shape.

// TerminalPrefs is the resolved terminal customization for this machine, as it
// rides the model snapshot. Every field is a pref the file set; a field left at
// its zero value is *unset*, and the client falls it through to the app default
// (a colour to the live design token, the font to the bundled family). This is
// how a partial file still looks intentional — the resolve happens once, at the
// token seam, exactly as `buildTheme` resolves colours today.
//
// Theme layering (ticket 02, spec Theme layering) lives entirely on the client:
// this side carries a validated preset *name* plus whatever explicit per-slot
// colours the file set, and the resolve seam stacks tokens → named preset →
// explicit slots. The palettes themselves are client data (tokens.ts PRESETS), so
// here Preset is only a name the parse checks against the bundled set.
type TerminalPrefs struct {
	// FontFamily is the CSS font-family string the terminal renders in. Empty
	// falls through to the bundled default; a non-bundled family is the operator's
	// own risk (it depends on their OS), so the parse does not police it.
	FontFamily string `json:"fontFamily,omitempty"`
	// FontSize is the cell font size in px. Zero is unset; a non-positive value is
	// refused with a warning and left unset.
	FontSize float64 `json:"fontSize,omitempty"`

	// Preset is the bundled theme preset the operator named (normalised to its
	// lower-case key). Empty is unset; an unknown name is refused with a warning and
	// left unset so the default theme stands. The palette it selects is resolved on
	// the client.
	Preset string `json:"preset,omitempty"`

	// The theme slots. Each is empty (unset — falls through to the preset then the
	// token-derived default at the resolve seam) or a validated `#hex`; a value that
	// is not a colour is refused with a warning and left unset. Background,
	// Foreground, Cursor, CursorAccent and Selection are the five base slots;
	// Black…BrightWhite are the sixteen ANSI slots. Selection drives xterm's
	// selectionBackground at the seam.
	Background   string `json:"background,omitempty"`
	Foreground   string `json:"foreground,omitempty"`
	Cursor       string `json:"cursor,omitempty"`
	CursorAccent string `json:"cursorAccent,omitempty"`
	Selection    string `json:"selection,omitempty"`

	Black   string `json:"black,omitempty"`
	Red     string `json:"red,omitempty"`
	Green   string `json:"green,omitempty"`
	Yellow  string `json:"yellow,omitempty"`
	Blue    string `json:"blue,omitempty"`
	Magenta string `json:"magenta,omitempty"`
	Cyan    string `json:"cyan,omitempty"`
	White   string `json:"white,omitempty"`

	BrightBlack   string `json:"brightBlack,omitempty"`
	BrightRed     string `json:"brightRed,omitempty"`
	BrightGreen   string `json:"brightGreen,omitempty"`
	BrightYellow  string `json:"brightYellow,omitempty"`
	BrightBlue    string `json:"brightBlue,omitempty"`
	BrightMagenta string `json:"brightMagenta,omitempty"`
	BrightCyan    string `json:"brightCyan,omitempty"`
	BrightWhite   string `json:"brightWhite,omitempty"`
}

type rawTerminal struct {
	Font  rawTermFont  `toml:"font"`
	Theme rawTermTheme `toml:"theme"`
}

type rawTermFont struct {
	Family string  `toml:"family"`
	Size   float64 `toml:"size"`
}

type rawTermTheme struct {
	Preset string `toml:"preset"`

	Background   string `toml:"background"`
	Foreground   string `toml:"foreground"`
	Cursor       string `toml:"cursor"`
	CursorAccent string `toml:"cursorAccent"`
	Selection    string `toml:"selection"`

	Black   string `toml:"black"`
	Red     string `toml:"red"`
	Green   string `toml:"green"`
	Yellow  string `toml:"yellow"`
	Blue    string `toml:"blue"`
	Magenta string `toml:"magenta"`
	Cyan    string `toml:"cyan"`
	White   string `toml:"white"`

	BrightBlack   string `toml:"brightBlack"`
	BrightRed     string `toml:"brightRed"`
	BrightGreen   string `toml:"brightGreen"`
	BrightYellow  string `toml:"brightYellow"`
	BrightBlue    string `toml:"brightBlue"`
	BrightMagenta string `toml:"brightMagenta"`
	BrightCyan    string `toml:"brightCyan"`
	BrightWhite   string `toml:"brightWhite"`
}

// terminalPresets is the set of bundled theme presets an operator may name in
// `theme.preset`. The palettes themselves are client data — tokens.ts PRESETS owns
// the colours because the resolve seam owns colour (ADR 0010); this side only
// validates the name so an unknown one warns instead of silently doing nothing.
// Keep this set in lockstep with PRESETS in web/src/lib/tokens.ts.
var terminalPresets = map[string]struct{}{
	"dracula":         {},
	"nord":            {},
	"gruvbox":         {},
	"solarized-dark":  {},
	"solarized-light": {},
}

// reColor matches the colour strings the theme slots accept — a `#rgb`, `#rrggbb`,
// or `#rrggbbaa` hex. Named colours and oklch are deliberately out: xterm's theme
// wants concrete colour strings, and the operator authors the file in hex the way
// every bundled preset is written.
var reColor = regexp.MustCompile(`^#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$`)

// ResolveTerminalPrefs reads the bytes of `terminal.toml` and produces the
// resolved prefs plus any warnings. A missing (empty) file yields all defaults
// and no warnings — today's look, unchanged. A file too malformed to decode
// yields all defaults and one warning: the terminal must never break on a typo.
// Each individual bad value (a non-positive size, a non-colour slot, an unknown
// key) is dropped with a warning while every good value beside it stands.
func ResolveTerminalPrefs(tomlBytes []byte) (TerminalPrefs, []string) {
	if len(tomlBytes) == 0 {
		return TerminalPrefs{}, nil
	}

	var raw rawTerminal
	md, err := toml.Decode(string(tomlBytes), &raw)
	if err != nil {
		return TerminalPrefs{}, []string{fmt.Sprintf(
			"terminal.toml could not be read: %s; the terminal keeps its defaults", err)}
	}

	var prefs TerminalPrefs
	var warnings []string

	prefs.FontFamily = strings.TrimSpace(raw.Font.Family)

	switch {
	case raw.Font.Size == 0:
		// unset — the client default stands
	case raw.Font.Size <= 0:
		warnings = append(warnings, fmt.Sprintf(
			"terminal font size %g is not a positive number; the default size stands", raw.Font.Size))
	default:
		prefs.FontSize = raw.Font.Size
	}

	prefs.Preset = validPreset(raw.Theme.Preset, &warnings)

	// Every theme slot validates the same way — a `#hex` lands, anything else warns
	// by its dotted key and stays unset for the client to fall through. The order
	// here is documentation only; each slot is independent.
	for _, sl := range []struct {
		key string
		raw string
		dst *string
	}{
		{"theme.background", raw.Theme.Background, &prefs.Background},
		{"theme.foreground", raw.Theme.Foreground, &prefs.Foreground},
		{"theme.cursor", raw.Theme.Cursor, &prefs.Cursor},
		{"theme.cursorAccent", raw.Theme.CursorAccent, &prefs.CursorAccent},
		{"theme.selection", raw.Theme.Selection, &prefs.Selection},
		{"theme.black", raw.Theme.Black, &prefs.Black},
		{"theme.red", raw.Theme.Red, &prefs.Red},
		{"theme.green", raw.Theme.Green, &prefs.Green},
		{"theme.yellow", raw.Theme.Yellow, &prefs.Yellow},
		{"theme.blue", raw.Theme.Blue, &prefs.Blue},
		{"theme.magenta", raw.Theme.Magenta, &prefs.Magenta},
		{"theme.cyan", raw.Theme.Cyan, &prefs.Cyan},
		{"theme.white", raw.Theme.White, &prefs.White},
		{"theme.brightBlack", raw.Theme.BrightBlack, &prefs.BrightBlack},
		{"theme.brightRed", raw.Theme.BrightRed, &prefs.BrightRed},
		{"theme.brightGreen", raw.Theme.BrightGreen, &prefs.BrightGreen},
		{"theme.brightYellow", raw.Theme.BrightYellow, &prefs.BrightYellow},
		{"theme.brightBlue", raw.Theme.BrightBlue, &prefs.BrightBlue},
		{"theme.brightMagenta", raw.Theme.BrightMagenta, &prefs.BrightMagenta},
		{"theme.brightCyan", raw.Theme.BrightCyan, &prefs.BrightCyan},
		{"theme.brightWhite", raw.Theme.BrightWhite, &prefs.BrightWhite},
	} {
		*sl.dst = validColour(sl.key, sl.raw, &warnings)
	}

	// Unknown keys are the operator's typos or settings a later ticket has not
	// taught the parse yet: surfaced, never fatal. Undecoded reports each in a
	// stable dotted path.
	for _, key := range md.Undecoded() {
		warnings = append(warnings, fmt.Sprintf(
			"terminal.toml has an unknown setting %q; it is ignored", key.String()))
	}

	sort.Strings(warnings)
	return prefs, warnings
}

// validPreset normalises a preset name (trim + lower-case) and keeps it if it is
// one of the bundled presets, otherwise warning by name and leaving it unset so
// the default theme stands. The normalised key is what rides the snapshot, so the
// client's PRESETS lookup is a direct hit.
func validPreset(value string, warnings *[]string) string {
	name := strings.TrimSpace(value)
	if name == "" {
		return ""
	}
	key := strings.ToLower(name)
	if _, ok := terminalPresets[key]; ok {
		return key
	}
	*warnings = append(*warnings, fmt.Sprintf(
		"terminal theme preset %q is not one of the bundled presets (%s); the default theme stands",
		name, strings.Join(presetNames(), ", ")))
	return ""
}

// presetNames returns the bundled preset names sorted, for a stable warning line.
func presetNames() []string {
	names := make([]string, 0, len(terminalPresets))
	for n := range terminalPresets {
		names = append(names, n)
	}
	sort.Strings(names)
	return names
}

// validColour keeps a colour slot if it is a hex colour, and otherwise warns by
// its dotted key and leaves the slot unset so the client falls it through to the
// token default.
func validColour(key, value string, warnings *[]string) string {
	v := strings.TrimSpace(value)
	if v == "" {
		return ""
	}
	if !reColor.MatchString(v) {
		*warnings = append(*warnings, fmt.Sprintf(
			"terminal %s %q is not a #hex colour; the default stands", key, v))
		return ""
	}
	return v
}
