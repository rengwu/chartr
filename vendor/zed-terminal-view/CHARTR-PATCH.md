# Chartr patch

This crate's source is an otherwise unchanged copy of Zed's `terminal_view`
crate at commit `1ea16c1ab9dd6d36649e002dc60995634da04daf`.

Chartr adds one host policy:

- `TerminalVerticalAlignment` and `TerminalView::set_vertical_alignment` let a
  non-Zed host choose whether spare sub-row pixels are placed above or below
  the terminal grid.
- Zed's existing `BottomWhenFull` behavior remains the default.
- Chartr selects `Top`, keeping the grid origin stable while a pane is resized.

When updating the pinned Zed revision, replace this directory from upstream
first, then reapply only the alignment enum, field, setter, layout branch, and
their tests.

`Cargo.toml` declares the upstream workspace dependencies explicitly so this
crate can build from Chartr's workspace without pretending to be part of Zed's.
