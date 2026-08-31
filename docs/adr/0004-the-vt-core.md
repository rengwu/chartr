# 0004 — alacritty's VT core, not libghostty

## Decision

`zeddy-vt` wraps `alacritty_terminal` — Zed's fork, at the revision Zed's own
terminal uses. Bytes in, a `Screen` out. That is the whole public surface.

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

## No scrollback

`scrolling_history` is zero. herdr's frame stream sends the viewport and has no
way to move it back through history, so a scrollback buffer here would be one
nothing can ever scroll to. History, when zeddy grows it, comes from the control
plane and is a different rendering.

## Colour is not resolved here

A cell carries `Default`, `Indexed(n)`, or `Rgb`. What those *are* belongs to
the theme, and resolving them in this crate would hard-code one. `zeddy::palette`
is where it happens, which makes a theme switch a re-render rather than a
re-parse.
