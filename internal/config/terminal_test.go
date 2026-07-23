package config_test

import (
	"reflect"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/config"
)

// Seam 1 — the pure parse of terminal.toml into a resolved TerminalPrefs value
// plus human-readable warnings. Every assertion is on what ResolveTerminalPrefs
// makes of the file's bytes: a valid value lands, an invalid one keeps the
// default and warns, a missing file is all defaults and silent. The client resolve
// seam (tokens.ts) turns these prefs into concrete xterm options; here we only
// prove the file becomes the value.

func TestTerminalPrefsMissingFileIsAllDefaults(t *testing.T) {
	prefs, warnings := config.ResolveTerminalPrefs(nil)
	if prefs != (config.TerminalPrefs{}) {
		t.Errorf("a missing file resolved to %+v, want the zero (all-default) prefs", prefs)
	}
	if len(warnings) != 0 {
		t.Errorf("a missing file warned: %v", warnings)
	}
}

func TestTerminalPrefsReadsFontAndColours(t *testing.T) {
	prefs, warnings := config.ResolveTerminalPrefs([]byte(`
[font]
family = "IBM Plex Mono"
size = 15

[theme]
background = "#1e2530"
foreground = "#e6e6e6"
`))
	if len(warnings) != 0 {
		t.Fatalf("a valid file warned: %v", warnings)
	}
	if prefs.FontFamily != "IBM Plex Mono" {
		t.Errorf("font family = %q, want the set value", prefs.FontFamily)
	}
	if prefs.FontSize != 15 {
		t.Errorf("font size = %v, want 15", prefs.FontSize)
	}
	if prefs.Background != "#1e2530" || prefs.Foreground != "#e6e6e6" {
		t.Errorf("colours = %q/%q, want the set values", prefs.Background, prefs.Foreground)
	}
}

func TestTerminalPrefsBadValueKeepsDefaultAndWarns(t *testing.T) {
	// A negative size and a colour that is not a colour: each falls back to its
	// unset default and the operator is told what was ignored.
	prefs, warnings := config.ResolveTerminalPrefs([]byte(`
[font]
size = -3

[theme]
background = "not-a-colour"
`))
	if prefs.FontSize != 0 {
		t.Errorf("a negative size resolved to %v, want the default (unset)", prefs.FontSize)
	}
	if prefs.Background != "" {
		t.Errorf("an invalid colour resolved to %q, want the default (unset)", prefs.Background)
	}
	if !hasSub(warnings, "size") {
		t.Errorf("no warning named the bad font size: %v", warnings)
	}
	if !hasSub(warnings, "background") {
		t.Errorf("no warning named the bad colour: %v", warnings)
	}
}

func TestTerminalPrefsUnknownKeyWarns(t *testing.T) {
	prefs, warnings := config.ResolveTerminalPrefs([]byte(`
[font]
family = "IBM Plex Mono"
kerning = "loose"
`))
	if prefs.FontFamily != "IBM Plex Mono" {
		t.Errorf("a known key beside an unknown one was dropped: %+v", prefs)
	}
	if !hasSub(warnings, "kerning") {
		t.Errorf("an unknown key produced no warning: %v", warnings)
	}
}

func TestTerminalPrefsReadsPresetAndSlots(t *testing.T) {
	// A named preset (case-insensitive) plus a spread of explicit slot overrides:
	// the preset name normalises to its bundled key and every slot lands, silently.
	prefs, warnings := config.ResolveTerminalPrefs([]byte(`
[theme]
preset = "Dracula"
blue = "#0000ff"
brightBlack = "#333333"
selection = "#222222"
cursor = "#abcdef"
`))
	if len(warnings) != 0 {
		t.Fatalf("a valid preset + slots warned: %v", warnings)
	}
	if prefs.Preset != "dracula" {
		t.Errorf("preset = %q, want the normalised bundled key %q", prefs.Preset, "dracula")
	}
	if prefs.Blue != "#0000ff" || prefs.BrightBlack != "#333333" ||
		prefs.Selection != "#222222" || prefs.Cursor != "#abcdef" {
		t.Errorf("per-slot overrides did not all land: %+v", prefs)
	}
}

func TestTerminalPrefsUnknownPresetWarnsAndFallsBack(t *testing.T) {
	prefs, warnings := config.ResolveTerminalPrefs([]byte(`
[theme]
preset = "mauve-dream"
`))
	if prefs.Preset != "" {
		t.Errorf("an unknown preset resolved to %q, want unset (the default theme stands)", prefs.Preset)
	}
	if !hasSub(warnings, "mauve-dream") {
		t.Errorf("an unknown preset produced no naming warning: %v", warnings)
	}
}

func TestTerminalPrefsBadSlotColourWarns(t *testing.T) {
	// A bad colour on a newly-added slot warns and stays unset, exactly like the two
	// base slots — every slot goes through the same validation.
	prefs, warnings := config.ResolveTerminalPrefs([]byte(`
[theme]
green = "chartreuse"
brightRed = "#00ff00"
`))
	if prefs.Green != "" {
		t.Errorf("a bad slot colour resolved to %q, want the default (unset)", prefs.Green)
	}
	if prefs.BrightRed != "#00ff00" {
		t.Errorf("a good slot beside a bad one was dropped: %+v", prefs)
	}
	if !hasSub(warnings, "green") {
		t.Errorf("a bad slot colour produced no warning naming it: %v", warnings)
	}
}

func TestTerminalPrefsReadsFontCursorScrollingOptions(t *testing.T) {
	// The full ticket-03 spread lands from a valid file, silently: font weight
	// (keyword and number), line height, letter spacing, the cursor block, the
	// scrolling block, the contrast floor, and the unicode11 toggle.
	blinkFalse := false
	uni := true
	prefs, warnings := config.ResolveTerminalPrefs([]byte(`
[font]
weight = "bold"
boldWeight = 800
lineHeight = 1.2
letterSpacing = -0.5

[cursor]
style = "bar"
blink = false
inactiveStyle = "outline"
width = 2

[scrolling]
scrollback = 5000
sensitivity = 2
fastScrollModifier = "ctrl"
fastScrollSensitivity = 8
smoothScrollDuration = 120

[accessibility]
minimumContrastRatio = 4.5

[glyph]
unicode11 = true
`))
	if len(warnings) != 0 {
		t.Fatalf("a valid options file warned: %v", warnings)
	}
	want := config.TerminalPrefs{
		FontWeight:            "bold",
		FontWeightBold:        "800",
		LineHeight:            1.2,
		LetterSpacing:         -0.5,
		CursorStyle:           "bar",
		CursorBlink:           &blinkFalse,
		CursorInactiveStyle:   "outline",
		CursorWidth:           2,
		Scrollback:            5000,
		ScrollSensitivity:     2,
		FastScrollModifier:    "ctrl",
		FastScrollSensitivity: 8,
		SmoothScrollDuration:  120,
		MinimumContrastRatio:  4.5,
		Unicode11:             &uni,
	}
	if !reflect.DeepEqual(prefs, want) {
		t.Errorf("options resolved to\n%+v\nwant\n%+v", prefs, want)
	}
}

func TestTerminalPrefsNumericFontWeight(t *testing.T) {
	// A numeric weight normalises to its integer string, so the wire carries one
	// field the client reads as a keyword or parses as a number.
	prefs, warnings := config.ResolveTerminalPrefs([]byte(`
[font]
weight = 600
`))
	if len(warnings) != 0 {
		t.Fatalf("a valid numeric weight warned: %v", warnings)
	}
	if prefs.FontWeight != "600" {
		t.Errorf("numeric weight = %q, want the normalised %q", prefs.FontWeight, "600")
	}
}

func TestTerminalPrefsOutOfRangeValuesWarnAndDefault(t *testing.T) {
	// One out-of-range value per rule: a non-positive line height, a contrast ratio
	// past the [1,21] range, a negative scrollback, and an unrecognised enum. Each
	// keeps its default (unset) and warns by name.
	prefs, warnings := config.ResolveTerminalPrefs([]byte(`
[font]
lineHeight = 0
weight = 1500

[cursor]
style = "beam"

[scrolling]
scrollback = -100

[accessibility]
minimumContrastRatio = 30
`))
	if prefs.LineHeight != 0 {
		t.Errorf("a zero line height resolved to %v, want unset", prefs.LineHeight)
	}
	if prefs.CursorStyle != "" {
		t.Errorf("an unknown cursor style resolved to %q, want unset", prefs.CursorStyle)
	}
	if prefs.Scrollback != 0 {
		t.Errorf("a negative scrollback resolved to %v, want unset", prefs.Scrollback)
	}
	if prefs.MinimumContrastRatio != 0 {
		t.Errorf("an out-of-range contrast ratio resolved to %v, want unset", prefs.MinimumContrastRatio)
	}
	if prefs.FontWeight != "" {
		t.Errorf("an out-of-range weight resolved to %q, want unset", prefs.FontWeight)
	}
	for _, sub := range []string{"scrollback", "style", "minimumContrastRatio", "weight"} {
		if !hasSub(warnings, sub) {
			t.Errorf("no warning named %q: %v", sub, warnings)
		}
	}
	// A zero line height is "unset", so it is silent — it is the default, not an error.
	if hasSub(warnings, "lineHeight") {
		t.Errorf("a zero (unset) line height warned: %v", warnings)
	}
}

func TestTerminalPrefsReadsScrollbarPaddingAndKeys(t *testing.T) {
	// The ticket-04 spread: the scrollbar block, the four padding sides, and the four
	// key/selection behaviours — all landing from a valid file, silently.
	yes := true
	no := false
	prefs, warnings := config.ResolveTerminalPrefs([]byte(`
[scrollbar]
width = 6
thumb = "#3a4034"
track = "#00000000"
autoHide = true

[padding]
top = 8
right = 12
bottom = 8
left = 12

[keys]
shiftEnterNewline = false
copyOnSelect = true
rightClickSelectsWord = true
macOptionIsMeta = true
`))
	if len(warnings) != 0 {
		t.Fatalf("a valid scrollbar/padding/keys file warned: %v", warnings)
	}
	want := config.TerminalPrefs{
		ScrollbarWidth:        6,
		ScrollbarThumb:        "#3a4034",
		ScrollbarTrack:        "#00000000",
		ScrollbarAutoHide:     &yes,
		PaddingTop:            8,
		PaddingRight:          12,
		PaddingBottom:         8,
		PaddingLeft:           12,
		ShiftEnterNewline:     &no,
		CopyOnSelect:          &yes,
		RightClickSelectsWord: &yes,
		MacOptionIsMeta:       &yes,
	}
	if !reflect.DeepEqual(prefs, want) {
		t.Errorf("scrollbar/padding/keys resolved to\n%+v\nwant\n%+v", prefs, want)
	}
}

func TestTerminalPrefsBadScrollbarAndPaddingWarn(t *testing.T) {
	// A non-colour thumb, a non-positive scrollbar width, and a negative padding side
	// each keep the default and warn; a good side beside a bad one still lands. Zero
	// padding is the default, so it is silent.
	prefs, warnings := config.ResolveTerminalPrefs([]byte(`
[scrollbar]
width = -4
thumb = "olive"

[padding]
top = -2
bottom = 0
left = 10
`))
	if prefs.ScrollbarWidth != 0 || prefs.ScrollbarThumb != "" || prefs.PaddingTop != 0 {
		t.Errorf("bad scrollbar/padding values survived: %+v", prefs)
	}
	if prefs.PaddingLeft != 10 {
		t.Errorf("a good padding side beside a bad one was dropped: %+v", prefs)
	}
	for _, sub := range []string{"scrollbar width", "scrollbar.thumb", "padding top"} {
		if !hasSub(warnings, sub) {
			t.Errorf("no warning named %q: %v", sub, warnings)
		}
	}
	if hasSub(warnings, "padding bottom") {
		t.Errorf("a zero (default) padding side warned: %v", warnings)
	}
}

func TestTerminalPrefsReadsLigatures(t *testing.T) {
	// The ticket-05 flag: `font.ligatures` is a tri-state boolean that lands verbatim
	// and silently. The renderer/font gating it drives lives on the client — the parse
	// only carries the flag.
	yes := true
	prefs, warnings := config.ResolveTerminalPrefs([]byte(`
[font]
ligatures = true
`))
	if len(warnings) != 0 {
		t.Fatalf("a valid ligatures file warned: %v", warnings)
	}
	if !reflect.DeepEqual(prefs.Ligatures, &yes) {
		t.Errorf("ligatures resolved to %v, want &true", prefs.Ligatures)
	}
	// Absent stays nil (unset — off), distinguishable from an explicit false.
	none, _ := config.ResolveTerminalPrefs([]byte("[font]\nsize = 13\n"))
	if none.Ligatures != nil {
		t.Errorf("an absent ligatures key resolved to %v, want nil", none.Ligatures)
	}
}

func TestTerminalPrefsMalformedFileWarnsAndDefaults(t *testing.T) {
	prefs, warnings := config.ResolveTerminalPrefs([]byte("this is not = = toml"))
	if prefs != (config.TerminalPrefs{}) {
		t.Errorf("a malformed file resolved to %+v, want the zero prefs", prefs)
	}
	if len(warnings) == 0 {
		t.Errorf("a malformed file produced no warning")
	}
}

func hasSub(warnings []string, sub string) bool {
	for _, w := range warnings {
		if strings.Contains(w, sub) {
			return true
		}
	}
	return false
}
