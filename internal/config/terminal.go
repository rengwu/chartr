package config

import (
	"fmt"
	"regexp"
	"sort"
	"strconv"
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
	// own risk (it depends on their OS), so the parse does not police it. The client
	// resolve seam knows which families are bundled (tokens.ts BUNDLED_FONTS) and
	// stacks a custom family ahead of the system fallback.
	FontFamily string `json:"fontFamily,omitempty"`
	// FontSize is the cell font size in px. Zero is unset; a non-positive value is
	// refused with a warning and left unset.
	FontSize float64 `json:"fontSize,omitempty"`
	// FontWeight and FontWeightBold are the normal- and bold-weight glyph weights,
	// each a normalised string: "normal", "bold", or a numeric "100".."900". Empty is
	// unset; a value that is neither keyword nor an integer in 1..1000 is refused
	// with a warning. Stored as a string so the numeric and keyword forms share one
	// wire field; the client passes a keyword straight through and a numeric string
	// as a number.
	FontWeight     string `json:"fontWeight,omitempty"`
	FontWeightBold string `json:"fontWeightBold,omitempty"`
	// LineHeight multiplies the cell height (1.0 is the default). Zero is unset; a
	// non-positive value is refused with a warning.
	LineHeight float64 `json:"lineHeight,omitempty"`
	// LetterSpacing adds horizontal space between cells in px. Zero is unset and the
	// xterm default; a negative value (tighter) is legitimate, so the parse accepts
	// any number.
	LetterSpacing float64 `json:"letterSpacing,omitempty"`

	// CursorStyle is the active-cursor shape — "block", "bar", or "underline". Empty
	// is unset; any other value warns. CursorBlink is a tri-state (nil unset, so the
	// island's alive-gated default stands); an explicit false stops the blink even on
	// a live shell. CursorInactiveStyle is the shape when the terminal is unfocused —
	// "outline", "block", "bar", "underline", or "none". CursorWidth is the bar
	// cursor's width in px; zero is unset, a non-positive value warns.
	CursorStyle         string  `json:"cursorStyle,omitempty"`
	CursorBlink         *bool   `json:"cursorBlink,omitempty"`
	CursorInactiveStyle string  `json:"cursorInactiveStyle,omitempty"`
	CursorWidth         float64 `json:"cursorWidth,omitempty"`

	// Scrollback is how many lines of history the terminal keeps; zero is unset
	// (the xterm default stands), a negative value warns. ScrollSensitivity and
	// FastScrollSensitivity scale a wheel tick and a fast-scroll wheel tick; each is
	// unset at zero and warns when non-positive. FastScrollModifier is the key that
	// engages fast scroll — "alt", "ctrl", "shift", or "none"; any other value warns.
	// SmoothScrollDuration is the smooth-scroll animation length in ms; zero is unset
	// (no smoothing), a negative value warns.
	Scrollback            float64 `json:"scrollback,omitempty"`
	ScrollSensitivity     float64 `json:"scrollSensitivity,omitempty"`
	FastScrollModifier    string  `json:"fastScrollModifier,omitempty"`
	FastScrollSensitivity float64 `json:"fastScrollSensitivity,omitempty"`
	SmoothScrollDuration  float64 `json:"smoothScrollDuration,omitempty"`

	// MinimumContrastRatio nudges low-contrast glyph/background pairs apart until they
	// clear this ratio (1..21, where 1 is the unset default and does nothing). A value
	// outside that range warns and stays unset.
	MinimumContrastRatio float64 `json:"minimumContrastRatio,omitempty"`

	// Unicode11 gates the unicode11 addon, which the island lazily imports and
	// activates at mount for correct wide-glyph and emoji widths. Tri-state: nil is
	// unset (the addon stays off, today's behaviour), an explicit value turns it on
	// or off.
	Unicode11 *bool `json:"unicode11,omitempty"`

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
	Font          rawTermFont          `toml:"font"`
	Theme         rawTermTheme         `toml:"theme"`
	Cursor        rawTermCursor        `toml:"cursor"`
	Scrolling     rawTermScrolling     `toml:"scrolling"`
	Accessibility rawTermAccessibility `toml:"accessibility"`
	Glyph         rawTermGlyph         `toml:"glyph"`
}

type rawTermFont struct {
	Family string  `toml:"family"`
	Size   float64 `toml:"size"`
	// Weight and BoldWeight take either a keyword ("normal"/"bold") or a number, so
	// they decode as an untyped value the resolve normalises — a single TOML key that
	// accepts the two natural spellings without a type-mismatch nuking the file.
	Weight        interface{} `toml:"weight"`
	BoldWeight    interface{} `toml:"boldWeight"`
	LineHeight    float64     `toml:"lineHeight"`
	LetterSpacing float64     `toml:"letterSpacing"`
}

type rawTermCursor struct {
	Style         string  `toml:"style"`
	Blink         *bool   `toml:"blink"`
	InactiveStyle string  `toml:"inactiveStyle"`
	Width         float64 `toml:"width"`
}

type rawTermScrolling struct {
	Scrollback            float64 `toml:"scrollback"`
	Sensitivity           float64 `toml:"sensitivity"`
	FastScrollModifier    string  `toml:"fastScrollModifier"`
	FastScrollSensitivity float64 `toml:"fastScrollSensitivity"`
	SmoothScrollDuration  float64 `toml:"smoothScrollDuration"`
}

type rawTermAccessibility struct {
	MinimumContrastRatio float64 `toml:"minimumContrastRatio"`
}

type rawTermGlyph struct {
	Unicode11 *bool `toml:"unicode11"`
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

// The closed enum sets the parse validates a slot against, each mapping the value
// straight to the xterm option name. An unknown value warns and stays unset so the
// island's default stands. Keep these in step with xterm's accepted values.
var (
	cursorStyles         = []string{"block", "bar", "underline"}
	cursorInactiveStyles = []string{"outline", "block", "bar", "underline", "none"}
	fastScrollModifiers  = []string{"alt", "ctrl", "shift", "none"}
)

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

	prefs.FontSize = validPositive("font size", raw.Font.Size, &warnings)
	prefs.FontWeight = validWeight("font weight", raw.Font.Weight, &warnings)
	prefs.FontWeightBold = validWeight("font boldWeight", raw.Font.BoldWeight, &warnings)
	prefs.LineHeight = validPositive("font lineHeight", raw.Font.LineHeight, &warnings)
	// LetterSpacing accepts any number — a negative value tightens the tracking and
	// is legitimate, so there is nothing to validate; zero is unset.
	prefs.LetterSpacing = raw.Font.LetterSpacing

	prefs.CursorStyle = validEnum("cursor style", raw.Cursor.Style, cursorStyles, &warnings)
	prefs.CursorBlink = raw.Cursor.Blink
	prefs.CursorInactiveStyle = validEnum(
		"cursor inactiveStyle", raw.Cursor.InactiveStyle, cursorInactiveStyles, &warnings)
	prefs.CursorWidth = validPositive("cursor width", raw.Cursor.Width, &warnings)

	prefs.Scrollback = validNonNegative("scrolling scrollback", raw.Scrolling.Scrollback, &warnings)
	prefs.ScrollSensitivity = validPositive(
		"scrolling sensitivity", raw.Scrolling.Sensitivity, &warnings)
	prefs.FastScrollModifier = validEnum(
		"scrolling fastScrollModifier", raw.Scrolling.FastScrollModifier, fastScrollModifiers, &warnings)
	prefs.FastScrollSensitivity = validPositive(
		"scrolling fastScrollSensitivity", raw.Scrolling.FastScrollSensitivity, &warnings)
	prefs.SmoothScrollDuration = validNonNegative(
		"scrolling smoothScrollDuration", raw.Scrolling.SmoothScrollDuration, &warnings)

	prefs.MinimumContrastRatio = validRange(
		"accessibility minimumContrastRatio", raw.Accessibility.MinimumContrastRatio, 1, 21, &warnings)

	prefs.Unicode11 = raw.Glyph.Unicode11

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

// validEnum keeps a value if it is one of the allowed options (after a trim), and
// otherwise warns by its key — naming the options — and leaves it unset so the
// island default stands. An empty value is unset and silent.
func validEnum(key, value string, allowed []string, warnings *[]string) string {
	v := strings.TrimSpace(value)
	if v == "" {
		return ""
	}
	for _, a := range allowed {
		if v == a {
			return v
		}
	}
	*warnings = append(*warnings, fmt.Sprintf(
		"terminal %s %q is not one of %s; the default stands", key, v, strings.Join(allowed, ", ")))
	return ""
}

// validPositive keeps a number if it is strictly positive; zero is unset (the
// default stands, silently) and a negative value warns. This is the rule for a
// size, a line height, a cursor width, and a scroll sensitivity — anything where a
// non-positive value is meaningless.
func validPositive(key string, value float64, warnings *[]string) float64 {
	switch {
	case value == 0:
		return 0 // unset
	case value < 0:
		*warnings = append(*warnings, fmt.Sprintf(
			"terminal %s %g is not a positive number; the default stands", key, value))
		return 0
	default:
		return value
	}
}

// validNonNegative keeps a number that is zero-or-more; zero is unset (the default
// stands) and only a negative value warns. This is the rule for a scrollback line
// count and a smooth-scroll duration, where zero is a legitimate-but-default value.
func validNonNegative(key string, value float64, warnings *[]string) float64 {
	if value < 0 {
		*warnings = append(*warnings, fmt.Sprintf(
			"terminal %s %g is not zero or more; the default stands", key, value))
		return 0
	}
	return value
}

// validRange keeps a number inside [lo, hi]; the range's low bound doubles as the
// unset value (e.g. a minimum-contrast-ratio of 1 does nothing), so a value equal to
// lo is treated as unset. A value outside the range warns and stays unset.
func validRange(key string, value, lo, hi float64, warnings *[]string) float64 {
	if value == 0 || value == lo {
		return 0 // unset
	}
	if value < lo || value > hi {
		*warnings = append(*warnings, fmt.Sprintf(
			"terminal %s %g is out of range %g..%g; the default stands", key, value, lo, hi))
		return 0
	}
	return value
}

// validWeight normalises a font weight that may be written as a keyword ("normal" or
// "bold") or as a number (an integer 1..1000). It returns the canonical string — the
// keyword as-is, or the integer rendered back to a string — so the wire carries one
// field the client can read as a keyword or parse as a number. An out-of-range or
// unrecognised value warns and stays unset.
func validWeight(key string, value interface{}, warnings *[]string) string {
	warn := func(shown string) string {
		*warnings = append(*warnings, fmt.Sprintf(
			"terminal %s %q is not \"normal\", \"bold\", or an integer 1..1000; the default stands",
			key, shown))
		return ""
	}
	switch v := value.(type) {
	case nil:
		return "" // unset
	case string:
		s := strings.TrimSpace(v)
		if s == "" {
			return ""
		}
		if s == "normal" || s == "bold" {
			return s
		}
		if n, err := strconv.Atoi(s); err == nil {
			return validWeightNumber(float64(n), s, key, warnings)
		}
		return warn(s)
	case int64:
		return validWeightNumber(float64(v), fmt.Sprintf("%d", v), key, warnings)
	case float64:
		return validWeightNumber(v, fmt.Sprintf("%g", v), key, warnings)
	default:
		return warn(fmt.Sprintf("%v", v))
	}
}

// validWeightNumber keeps a numeric weight if it is a whole number in 1..1000,
// returning it as its integer string; anything else warns through the caller.
func validWeightNumber(n float64, shown, key string, warnings *[]string) string {
	if n == float64(int(n)) && n >= 1 && n <= 1000 {
		return strconv.Itoa(int(n))
	}
	*warnings = append(*warnings, fmt.Sprintf(
		"terminal %s %q is not \"normal\", \"bold\", or an integer 1..1000; the default stands",
		key, shown))
	return ""
}
