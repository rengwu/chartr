# 0002 — The Zed layer, and what it costs

## Decision

zeddy depends on four crates from one pinned Zed revision: `gpui`,
`gpui_platform`, `ui`, and `theme`. It does **not** depend on Zed's `workspace`
crate. The sidebar, the tab strip, and the pane layout are zeddy's own, about
three hundred lines between them.

zeddy is therefore GPL-3.0-or-later.

## Why not `workspace`

`workspace` is where Zed's docks, pane groups, splits, and tab bar live, and
taking it would have meant not writing any of the chrome. Its transitive closure
inside Zed is **88 crates** — `client`, `project`, `language`, `remote`, `db`,
`telemetry`, `node_runtime` among them. Constructing a `Workspace` needs an
`AppState` carrying a collab client, a user store, a language registry, and a
sqlite database, none of which zeddy has any use for. The two modes zeddy
actually wants are a fixed-width column and a horizontal strip.

`ui` + `theme` close over 21 crates instead, and that closure is worth it: it is
what makes zeddy's buttons, labels, tabs, and colours Zed's own rather than a
re-implementation that looks almost right.

## The licence, stated plainly

`gpui` is Apache-2.0. `ui`, `theme`, and `workspace` are all GPL-3.0-or-later,
and zeddy links `ui` and `theme` directly. zeddy is GPL-3.0-or-later as a
result, and so is any native plugin, which links the same objects. This is a
consequence of the decision above, not an independent choice — dropping `ui` and
`theme` for a hand-written kit over Apache-2.0 `gpui` is the whole of what it
would take to change it.

## Two things this pins

- **Only `zeddy` may name `gpui_platform`.** A plugin that linked the platform
  backend would register a second application with the window server.
- **`gpui_platform` must be built with `font-kit`.** It is not in that crate's
  default features, and without it macOS silently gets a text system that
  rasterises nothing: every quad, icon, and border paints correctly and not one
  glyph appears, with the explanation behind a `log::warn!` that an app with no
  logger never sees. `fonts::text_renders` checks the result at startup and
  refuses to open a window that cannot show text.
