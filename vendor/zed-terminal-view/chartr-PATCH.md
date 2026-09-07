# chartr patch

This crate's source is an otherwise unchanged copy of Zed's `terminal_view`
crate at commit `1ea16c1ab9dd6d36649e002dc60995634da04daf`.

chartr adds three host policies:

- `TerminalVerticalAlignment` and `TerminalView::set_vertical_alignment` let a
  non-Zed host choose whether spare sub-row pixels are placed above or below
  the terminal grid.
- Zed's existing `BottomWhenFull` behavior remains the default.
- chartr selects `Top`, keeping the grid origin stable while a pane is resized.
- `TerminalView::set_grid_padding` gives a standalone grid one cell-width of
  base padding on every edge. chartr enables it; right and bottom may retain
  fractional space because terminal dimensions use whole columns and rows.
- The vertical scrollbar overlays the terminal instead of reserving a stable
  track, so an idle scrollbar does not create a permanent right-hand inset.

When updating the pinned Zed revision, replace this directory from upstream
first, then reapply only the alignment and padding fields, setters and layout
branches, the overlay-scrollbar render change, and their tests.

`Cargo.toml` declares the upstream workspace dependencies explicitly so this
crate can build from chartr's workspace without pretending to be part of Zed's.
