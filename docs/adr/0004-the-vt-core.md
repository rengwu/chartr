# 0004 — Alacritty output and Ghostty input behind one VT boundary

## Decision

`zeddy-vt` wraps two terminal cores for different jobs. Zed's pinned
`alacritty_terminal` parses repaint bytes and optional ANSI host history into a
`Screen`. The pinned safe `libghostty-vt` binding turns normalized key events
into mode-aware terminal input bytes. Neither upstream vocabulary crosses the
crate boundary.

## Why

Zeddy's renderer is built on Zed's frontend, and taking Zed's parser means the
grid semantics the renderer assumes and the grid semantics the parser produces
already agree. The traffic Zeddy parses is not where Ghostty's faster parser is
valuable.

Keyboard encoding is different. Modified navigation, function keys, application
cursor/keypad modes, xterm extensions, fixterms, and the Kitty keyboard protocol
form a stateful protocol rather than a maintainable escape-sequence table.
Ghostty already implements that protocol and is also the encoder used by
chartr-rs. The Zig 0.16.0 build dependency is accepted for input fidelity; the
safe binding, Ghostty commit, and Zig version move as one deliberate pin.

Herdr's frame stream remains a *re-render of its own emulated grid*, not the raw
output of the program in the PTY. Mode-aware encoding therefore uses every mode
the local parser can observe but cannot reconstruct modes Herdr omits. Legacy
Ghostty encoding is authoritative today; fully negotiated Kitty behavior
requires Herdr to carry structured keys or terminal mode state in the future.

## Snapshots, not borrows

`Terminal::screen` copies. A borrowed grid would be faster and would tie the
render pass to the lifetime of an emulator owned by a different thread than the
one painting. At the sizes a terminal runs — a few thousand cells — the copy is
not what makes a frame slow.

## Host-backed scrollback

The live emulator keeps `scrolling_history` at zero because herdr's frame stream
sends only viewport repaints; treating those repaints as raw PTY output creates
duplicate and missing history. On the first upward wheel gesture, zeddy reads
ANSI-styled `recent` history through Herdr's `pane.read` control method on a
background thread. `zeddy-vt` parses that into a separate historical emulator
and moves its display offset. Returning to offset zero renders the live emulator
again. New output marks a bottomed history snapshot stale, and a resize discards
it, so the next upward gesture asks Herdr for an authoritative replacement.

## Colour is not resolved here

A cell carries `Default`, `Indexed(n)`, or `Rgb`. What those *are* belongs to
the theme, and resolving them in this crate would hard-code one. `zeddy::palette`
is where it happens, which makes a theme switch a re-render rather than a
re-parse.
