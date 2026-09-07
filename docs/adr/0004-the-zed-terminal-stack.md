# 0004 — Zed's complete terminal stack

## Decision

chartr uses Zed's pinned `terminal` model and `terminal_view::TerminalView`
together, without a local emulator, renderer, input encoder, scrollback model,
or input path. A Zed-created local PTY runs Herdr's native interactive
`terminal attach <terminal-id> --takeover` client. Herdr continues to own the
persistent PTY and process lifetime.

The view source is vendored at the same exact Zed revision with one narrow host
capability: `TerminalVerticalAlignment`. Upstream behavior remains the default;
chartr selects `Top` so leftover pixels smaller than a terminal row stay below
the grid instead of shifting the grid origin during resize. The patch and its
rebase procedure are recorded beside the vendored crate.

## Why the complete stack

A modern terminal is not a parser followed by a grid. Keyboard protocols,
alternate-screen scrolling, mouse reporting, selection, clipboard, IME,
hyperlinks, resize, scrollback, and rendering share state and edge cases. Using
only a VT library left chartr responsible for the rest of that contract, which
is why individually reasonable fixes still failed to produce Zed-quality
behavior.

The model and view at one Zed revision are already exercised together in Zed.
Keeping them together gives chartr the same hot paths and the same interaction
semantics instead of asking chartr to reproduce them around a parser.

## Why not libghostty-vt

`libghostty-vt` is a capable, high-performance VT engine. It is not a GPUI
terminal view or a complete desktop-terminal integration. chartr would still
own rendering, input dispatch, clipboard, IME, mouse, keymaps, accessibility,
and their synchronization with its state. Choosing it would therefore optimize
one component while retaining the custom frontend this decision removes.

## Persistence boundary

Dropping a Zed terminal closes only the local Herdr attach client. The shell and
its PTY stay in the private Herdr daemon and can be attached again after a
chartr relaunch. Closing a chartr terminal item remains destructive because the
control plane explicitly closes the corresponding Herdr pane.

The Herdr crate returns one complete, namespace-safe attach command. The Zed
host consumes that specification without knowing Herdr flags or environment
rules.

## Host and customization boundary

chartr is not a Zed `Workspace` or `Project`. One `terminal_host` adapter passes
absent weak handles to `TerminalView` and selects its documented
non-workspace-host behavior by hiding workspace-only actions. It does not create
a fake Zed workspace and it does not reimplement any terminal behavior. A
`SpaceEvent::TerminalReady` creates the view with its window and installs it on
the stable session item before rendering; render code only reads and mounts the
existing entity.

Customization remains available through Zed terminal settings, chartr's theme
settings provider, and Zed's pinned default terminal keymap. chartr filters that
keymap by terminal action namespace instead of copying a subset, then adds only
product-level actions such as terminal-buffer search. Product layout and
lifecycle stay in chartr.

The host boundary supplies integrations that cannot exist without product
context: file drops paste shell-quoted paths, filesystem links open through the
desktop, and terminal BEL state appears in chartr chrome. These are callbacks
around public terminal APIs, not alternate input, rendering, or emulation paths.

## Cost

`terminal_view` brings a larger portion of Zed's pinned dependency graph and a
small source-vendoring burden. A rendered regression test covers the maintained
alignment branch at adjacent pane heights. Zed's workspace patches are mirrored
at the same revisions so the external build is reproducible. This is an
intentional build-size tradeoff for sharing the tested terminal implementation
rather than maintaining a parallel one.
