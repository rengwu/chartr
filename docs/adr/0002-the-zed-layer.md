# 0002 — The Zed layer, and what it costs

## Decision

chartr uses `gpui`, `gpui_platform`, `ui`, and `theme` from one pinned Zed
revision while owning its space, tab, and pane models. Subsequent terminal and
editor integration expanded that dependency graph: Zed's `workspace`, project,
and editor crates are now transitive dependencies. chartr still does not construct
a Zed `Workspace` or use it as its application model. See
[ADR 0004](0004-the-zed-terminal-stack.md) for the terminal boundary.

chartr is therefore GPL-3.0-or-later.

## Why not `workspace`

`workspace` is where Zed's docks, pane groups, splits, and tab bar live.
The original decision avoided adopting its application model and associated
`client`, `project`, `language`, `remote`, `db`, `telemetry`, and `node_runtime`
systems. Constructing a `Workspace` needs an
`AppState` carrying a collab client, a user store, a language registry, and a
sqlite database. chartr supplies its own workspace ownership and presentation.

Reusing `ui` and `theme` makes chartr's controls and colors follow the same
framework. The original small dependency-count comparison no longer describes
the build after adopting the complete terminal stack and Markdown editor.

## The licence, stated plainly

`gpui` is Apache-2.0. `ui`, `theme`, and `workspace` are all GPL-3.0-or-later,
and chartr links `ui` and `theme` directly. chartr is GPL-3.0-or-later as a
result, and so is any native plugin, which links the same objects. This is a
consequence of the decision above, not an independent choice — dropping `ui` and
`theme` for a hand-written kit over Apache-2.0 `gpui` is the whole of what it
would take to change it.

## Two things this pins

- **Only `chartr` may name `gpui_platform`.** A plugin that linked the platform
  backend would register a second application with the window server.
- **`gpui_platform` must be built with `font-kit`.** It is not in that crate's
  default features, and without it macOS silently gets a text system that
  rasterises nothing: every quad, icon, and border paints correctly and not one
  glyph appears, with the explanation behind a `log::warn!` that an app with no
  logger never sees. `fonts::text_renders` checks the result at startup and
  refuses to open a window that cannot show text.
