# 0004 — alacritty's VT core, not libghostty

## Decision

`zeddy-vt` wraps `alacritty_terminal` — Zed's fork, at the revision Zed's own
terminal uses. Repaint bytes and optional ANSI host history go in; a `Screen`
comes out.

## Why

libghostty-vt is the faster parser and is what a terminal built for raw speed
would reach for. It also needs an exact Zig version and, on macOS, Xcode's Metal
toolchain, before `cargo build` does anything. zeddy's renderer is built on Zed's
frontend, and taking Zed's parser means the grid semantics the renderer assumes
and the grid semantics the parser produces already agree.

The traffic zeddy parses is also not what that speed is for. herdr's frame
stream is a *re-render of its own emulated grid* — cell-addressed writes with
normalised SGR, at herdr's repaint rate — not the raw output of the program in
the PTY. The parser is not the bottleneck on that path.

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
