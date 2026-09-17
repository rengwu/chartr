# 0005 — Spaces follow Zed's multi-workspace ownership

## Decision

One window owns several spaces. Tabbed chrome presents the active space;
sidebar chrome presents all spaces. Inbox also lists conversations from all
spaces, alongside the selected original terminal. The
ownership shape follows the pinned Zed revision's `workspace::MultiWorkspace`:

- the root owns an ordered `Vec<Entity<Space>>` and the active entity;
- each `Space` owns an ordered outer workspace-tab collection, items, session
  collection, and active outer tab;
- each outer workspace tab owns one Zed-style pane tree and is presented as a
  standalone tab when it has one item or a grouped tab when it has several;
- the root observes every child and partitions backend snapshots between them;
- session actions carry stable outer-tab and pane ids, not positions in the
  currently drawn list; and
- blocking backend calls run on GPUI's background executor and return owned
  answers to the entity context.

The correspondence is structural: chartr does not construct a Zed `Workspace`.
The complete terminal and editor stack now brings that crate into the transitive
dependency graph, as clarified in [ADR 0002](0002-the-zed-layer.md).

## Persistence

The folder registry lives at `$XDG_CONFIG_HOME/chartr/spaces.toml`, with
platform fallbacks, file order as display order, duplicate suppression, and
unknown TOML keys preserved. A committed sidebar reorder atomically rewrites
that file order; a failed write restores the previous registry and drawn order.
Window bounds, chrome choice, complete space order (including the synthetic and
recovered spaces), ordered outer tabs, pane trees, item ownership, and restorable
plugin state live in chartr's SQLite state store. A pre-outer-tab pane tree
migrates to one grouped outer entry. The rewrite deliberately does not import or
mutate older chartr registries.

Free sessions are the one synthetic space. They use the operator's home
directory and have no registry row. A registered home-directory row is not
drawn beside it because herdr has one workspace per directory; two labels over
one backend workspace would pretend to be independent state when they are not.

## Chrome

Both chromes project the same outer collection: every standalone item and every
pane group is one entry. Sidebar mode draws those entries beneath each visible
space; tabbed mode draws the active space's entries beside its name. Selecting a
group reveals the pane-local Zed tab bars, while a standalone item has no
duplicate inner bar. Both reuse Zed `ui` components for tabs, buttons, labels,
icons, colors, focus tracking, and scroll containers. Shared host popup/modal
adapters keep those controls above native child webviews; their platform patches
are documented in [vendor/zed-platform](../../vendor/zed-platform/README.md).

All-Spaces sidebar cards have one focused sorter owned by the root window rather
than a general drag-and-drop framework. A heading drag carries the whole card on
the Y axis even after horizontal overdrag, compares final pointer Y against
measured variable-height midpoints, and uses the tracked Zed scroll handle for
edge autoscroll. Reordered neighbours and the released card use an interruptible
150 ms quintic FLIP; GPUI's application-wide reduced-motion flag removes those
animations without changing direct manipulation.

Standalone terminal labels are live backend presentation: detected agent,
non-shell foreground process, then Herdr's persistent tab label or number. They
are refreshed on the same two-second cadence as session discovery and are not
persisted locally. A collapsed pane group has an optional persisted name and
otherwise uses its item count, such as `5 tabs`; its children retain their
individual live labels in the pane-local tab bars.

## Consequence

Switching spaces is an entity-selection change. It cannot reparent a session,
reuse another space's selected index, or recreate backend work. Moving a
standalone outer tab into a selected pane changes ownership once and removes its
emptied outer entry; changing chrome never does. Adding a third chrome
arrangement likewise cannot change the space model: it can only draw the active
child's content through the same ownership boundary. Inbox maintains a separate
durable history index and mounts the selected session's existing terminal.
