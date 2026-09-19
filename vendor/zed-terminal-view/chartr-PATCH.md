# chartr patch

This crate's source is an otherwise unchanged copy of Zed's `terminal_view`
crate at commit `1ea16c1ab9dd6d36649e002dc60995634da04daf`.

chartr adds five host policies:

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
- `TerminalView::set_resize_paused` preserves the terminal grid during a host
  layout animation, while its origin and clipping follow the viewport. chartr
  pauses resizing during view-mode transitions and automatic sidebar expansion,
  then applies the final dimensions once the animation ends.
- `TerminalView::set_report_scroll_events` lets a multiplexer host receive SGR
  wheel reports even when its child is not capturing mouse input. It is off by
  default. Chartr enables it for Herdr-owned scrollback, while click/drag selection
  and TUI mouse reporting follow the child's modes. The wheel-only override uses
  Zed's existing scroll accumulator and encoder and restores the mode immediately.

When updating the pinned Zed revision, replace this directory from upstream
first, then reapply only the alignment, padding and resize-pause fields, setters
and layout branches, the overlay-scrollbar render change, the wheel-reporting
host policy, and their tests.

`Cargo.toml` declares the upstream workspace dependencies explicitly so this
crate can build from chartr's workspace without pretending to be part of Zed's.
