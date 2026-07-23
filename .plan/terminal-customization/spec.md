# Terminal customization

## Problem Statement

The terminal islands are hard-coded. The operator gets one font, one size, one
theme (the token-derived palette with six literal ANSI hues baked into
`buildTheme`), a fixed cursor, fixed scrollback, no padding control, no scrollbar
control, and no say over keybindings. Enter always submits — there is no
Ghostty-style Shift+Enter newline. There is no way to click a URL in output, no
in-terminal find, no ligatures, and no GPU-accelerated renderer. An operator who
lives in these terminals all day cannot make them feel like their own terminal.

## Solution

The operator gets a per-machine `terminal.toml` that fully customizes every
terminal island: font, a layered theme (app defaults → a named preset → their own
per-slot overrides), cursor, scrolling, scrollbar appearance, padding, and
keybindings including Shift+Enter for a newline. New capabilities ship on top — a
GPU (WebGL) renderer with a safe fallback, clickable links, an in-terminal find,
optional ligatures, correct wide-glyph widths, and an accessibility contrast
floor.

The file is the single source of truth, and it follows the existing config
philosophy exactly: it is one more per-machine file the operator edits in their
own editor, surfaced in the Settings surface as read-value-plus-open-file — never
a second config store, and never committed. Editing the file and saving it
re-applies the settings to every open terminal. A malformed file never breaks the
terminal: the affected values fall back to defaults and the operator is told what
was wrong, through the same warnings surface spaces already use.

## User Stories

1. As an operator, I want to set the terminal font family, so that my terminals
   use the typeface I read code in all day.
2. As an operator, I want to choose from a curated list of fonts the app can
   render offline, so that I am not guessing which fonts actually work.
3. As an operator, I want to type in a custom font family string, so that I can
   use a font installed on my own machine, understanding it depends on my OS.
4. As an operator, I want to set font size, weight, bold weight, line height, and
   letter spacing, so that the text is comfortable to read.
5. As an operator, I want to pick a named theme preset (e.g. Dracula, Solarized,
   Nord, Gruvbox), so that I can re-theme the terminal in one line.
6. As an operator, I want to override individual colour slots on top of a preset,
   so that I can tweak just the background or cursor without hand-authoring a
   whole palette.
7. As an operator, I want any colour slot I leave unset to fall through to the
   app's own token-derived default, so that a partial theme still looks
   intentional.
8. As an operator, I want to author a full palette from scratch with no preset, so
   that I can paste in a theme I already have.
9. As an operator, I want to set cursor style (block, bar, underline), blink, the
   inactive-cursor style, and width, so that the cursor behaves the way I expect.
10. As an operator, I want to set scrollback length, so that I can keep more (or
    less) history in a terminal.
11. As an operator, I want to tune scroll sensitivity, fast-scroll modifier and
    sensitivity, and smooth-scroll duration, so that scrolling feels right on my
    hardware.
12. As an operator, I want to control the scrollbar's width, thumb and track
    colour, and whether it auto-hides, so that it matches my aesthetic.
13. As an operator, I want to set padding around the terminal grid, so that the
    text is not jammed against the pane edges.
14. As an operator, I want Shift+Enter to insert a newline instead of submitting,
    so that I can compose multi-line input the way Ghostty allows.
15. As an operator, I want copy-on-select, right-click-selects-word, and
    macOS-option-is-meta behaviours to be configurable, so that selection and
    keys work how I am used to.
16. As an operator, I want a minimum-contrast-ratio floor, so that low-contrast
    theme text is automatically nudged readable.
17. As an operator, I want correct widths for wide glyphs and emoji, so that
    alignment does not break on modern output.
18. As an operator, I want a GPU-accelerated renderer by default, so that heavy
    output stays fast.
19. As an operator, I want the terminal to keep working if the GPU context is
    lost (backgrounded tab, driver reset), so that it never goes blank.
20. As an operator, I want to enable ligatures, so that my coding font's ligatures
    render — accepting that this switches that terminal off the GPU renderer and
    only works for a bundled font.
21. As an operator, I want to click a URL in terminal output and have it open, so
    that I do not have to copy-paste links.
22. As an operator using the macOS shell, I want a clicked link to open in my
    system browser, so that links behave like they do everywhere else.
23. As an operator, I want to press Cmd+F to find text in a terminal, with a match
    count and next/previous and match-case, so that I can locate output in long
    scrollback.
24. As an operator, I want to edit `terminal.toml` and have my open terminals
    re-apply the settings, so that I can iterate without restarting.
25. As an operator, I want to open `terminal.toml` from the Settings surface in my
    own editor, so that I edit it the same way I edit the rest of my config.
26. As an operator, I want to see my current terminal settings reflected in the
    Settings surface, so that I know what is in effect.
27. As an operator, I want a malformed file or a bad value to fall back to
    defaults with a clear warning, so that a typo never leaves me with a broken or
    blank terminal.
28. As an operator, I want these settings to be per-machine and never committed,
    so that my terminal look is mine and does not travel with a repository.
29. As an operator, I want the settings to apply to every terminal island
    uniformly, so that all my terminals look and behave consistently.

## Implementation Decisions

**Storage & ownership.** Terminal customization lives in a single per-machine
`terminal.toml` under the user config (state) root, beside the agent library.
It is never committed and never per-space. It is the single source of truth. This
is a plain on-disk file the operator edits in their own editor — not a write-back
UI. The Settings surface stays read-value-plus-open-file.

**Server parse seam.** A pure function reads the file's contents and produces a
resolved `TerminalPrefs` value plus a list of human-readable warnings. It applies
the layering (defaults → named preset → explicit per-slot overrides), validates
values (colours, enums, numbers), keeps a default for anything invalid or unset,
and warns on invalid values and unknown keys. A missing file yields all defaults
and no warnings. This resolved value is folded into the pushed model snapshot
alongside its warnings, so every browser receives the effective settings the same
way it receives the rest of the model. (Per the agreed seam split, "the settings
land on the snapshot" is tested together with parsing, not as a separate seam.)

**Warnings surface.** Parse/validation warnings feed the existing config-warnings
surface — the same mechanism spaces already use to report live config problems.
The terminal always runs; warnings explain what was ignored.

**Client resolve seam.** A pure builder (beside the existing token bridge in
`tokens.ts`) turns `TerminalPrefs` into the concrete objects the terminal island
consumes: the xterm options object and the resolved theme. Unset colour slots
resolve against the live design tokens exactly as `buildTheme` does today, so a
partial theme composes with the reskin. A non-bundled font family resolves to a
system fallback. Shift+Enter is a pure `event → action` decision in the same
module.

**Theme layering.** Order is tokens → named preset → explicit slots. A handful of
presets (Dracula, Solarized, Nord, Gruvbox, and similar) are bundled by name. Any
slot named in the file wins over the preset; any slot left unset falls through to
the token-derived default. The six ANSI hues currently hard-coded in `buildTheme`
become the default preset layer rather than literals.

**Island reactivity — remount on change.** The terminal island reacts to a prefs
change by fully remounting. The terminal socket replays scrollback on re-attach,
so nothing is lost. This keeps one code path and avoids in-place addon swapping.
A flicker on each edit is accepted.

**Addon loading — lazy and pref-gated.** Each optional addon is dynamically
imported and instantiated only when its pref enables it, at mount time. Because
the island already remounts on any change, there is no addon hot-swap logic:
each fresh mount imports exactly what the current prefs ask for. All addons are
bundled — no CDN, no runtime fetch (the frontend is embedded into one offline
binary).

**Renderer & the ligatures conflict.** The GPU (WebGL) renderer is the default,
with a DOM-renderer fallback wired to the GPU context-loss event so the terminal
never goes blank. `ligatures = true` forces that terminal onto the canvas
renderer (GPU off), because the ligatures addon and the GPU renderer cannot
coexist. Ligatures only apply to a bundled font, and the ligatures addon is
pointed at the embedded font asset — it never fetches a font over the network.

**Scrollbar & padding.** Scrollbar appearance and terminal padding are driven as
CSS custom properties on the island host (xterm exposes no options for either);
scrollbar styling targets the viewport element. A padding change is followed by a
refit so the shell reflows to the corrected column/row count.

**Links.** The web-links addon makes URLs clickable. Opening prefers a
shell-provided external-open hook when present (mirroring the existing
`__chartrTitleBar` native-shell global pattern), falling back to opening a new
browser tab. This requires a small addition to the macOS webview shell to expose
that hook so links reach the system browser (ADR 0013); until it lands, links
degrade to the browser-tab fallback.

**Find widget.** Cmd+F opens a token-driven find widget hosted by the island
wrapper in the chrome — input, match count, next/previous, match-case — bound to
the search addon. It is designed to sit beside the island at the seam, feeding the
addon, never reaching inside the renderer (ADR 0010). Its transient open/closed
state is UI state, not config.

**Settings surface.** A new "Terminal" section on the Global scope of the Settings
surface renders the current effective settings (read from the snapshot) and an
open-`terminal.toml` row using the existing files-on-disk / open-in-editor
pattern. Per-machine cosmetic settings belong on the Global scope beside the user
config, not per space.

**Design-system compliance.** Terminal content colours are legitimately chromatic
(like the star-map's status hues) and are exempt from the monochrome-chrome rule;
they are fed in at the seam, never inlined into the chrome. The find widget and
the Settings section are built on tokens and vendored primitives. Icons are
Phosphor; fonts are the bundled IBM Plex family.

## Testing Decisions

**What makes a good test here.** Tests observe external behaviour at the two pure
seams only — settings in, resolved values out — never the imperative island. We
do not test that xterm renders, that the GPU fallback fires, that padding looks
right, or that the find widget floats correctly; those live inside the island and
are trusted once the resolve seam hands them the right object. This matches how
the islands are treated today (ADR 0010).

**Seam 1 — server parse (folded with snapshot assembly).** Table-tested: a valid
file resolves to the expected settings; an invalid colour/enum/number keeps the
default and produces a warning; preset-then-override layering resolves correctly;
an unset slot stays default; unknown keys warn; a missing file yields all defaults
and no warnings. The same tests assert the resolved settings and warnings appear
on the pushed model snapshot — the two are tested as one, not split. Prior art:
the existing Go config parsing and the config-warnings behaviour spaces already
have.

**Seam 2 — client resolve.** Table-tested like `tokens.test.ts`: `TerminalPrefs`
builds the expected xterm options and theme object; unset colour slots resolve to
token defaults; a preset applies and explicit slots override it; a non-bundled
font family falls back; the Shift+Enter predicate maps the event to a newline
action while a plain Enter submits. Prior art: `tokens.test.ts` and the other
pure `web/src/lib/*.test.ts` suites.

## Out of Scope

- A write-back settings UI. Editing happens in the operator's editor; the Settings
  surface stays read-value-plus-open-file. Interactive in-panel controls that
  write `terminal.toml` are explicitly not in this spec (they would reintroduce a
  second config store).
- Per-space or committed terminal settings. This is per-machine user config only.
- Theme sync, sharing, or a preset marketplace beyond the bundled named presets.
- Retheming the terminal renderer internals; all customization is fed in at the
  seam.
- The macOS shell's own build/packaging work beyond exposing the external-open
  hook the links story depends on.
- Sixel/image output, and any addon not named here.

## Further Notes

- Cross-team dependency: the links story needs a small macOS webview-shell
  addition exposing an external-open hook, mirroring the `__chartrTitleBar`
  pattern (ADR 0013). Until it exists, links use the browser-tab fallback and the
  story is partially delivered.
- The six literal ANSI hues in today's `buildTheme` are not lost — they become the
  default preset layer, so the current look is the zero-config baseline.
- The `disable-model-invocation` note on this workflow means the spec was written
  from the conversation's decisions; the four grilling rounds that produced them
  (storage, renderer/ligatures, addon loading, live-apply, links, theme format,
  font, bad-config handling) are all reflected in Implementation Decisions.
