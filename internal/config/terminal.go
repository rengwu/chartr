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
type TerminalPrefs struct {
	// FontFamily is the CSS font-family string the terminal renders in. Empty
	// falls through to the bundled default; a non-bundled family is the operator's
	// own risk (it depends on their OS), so the parse does not police it.
	FontFamily string `json:"fontFamily,omitempty"`
	// FontSize is the cell font size in px. Zero is unset; a non-positive value is
	// refused with a warning and left unset.
	FontSize float64 `json:"fontSize,omitempty"`
	// Background and Foreground are the two base theme slots. Empty falls through
	// to the token-derived default; a value that is not a colour is refused with a
	// warning and left unset.
	Background string `json:"background,omitempty"`
	Foreground string `json:"foreground,omitempty"`
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
	Background string `toml:"background"`
	Foreground string `toml:"foreground"`
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

	prefs.Background = validColour("theme.background", raw.Theme.Background, &warnings)
	prefs.Foreground = validColour("theme.foreground", raw.Theme.Foreground, &warnings)

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
