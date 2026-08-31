# 0005 — Spaces follow Zed's multi-workspace ownership

## Decision

One window owns several spaces. Tabbed chrome presents the active space;
sidebar chrome can present either the active space or every space at once. The
ownership shape follows the pinned Zed revision's `workspace::MultiWorkspace`:

- the root owns an ordered `Vec<Entity<Space>>` and the active entity;
- each `Space` owns its pane tree, items, session collection, and active item;
- the root observes every child and partitions backend snapshots between them;
- session actions carry stable pane ids, not positions in the currently drawn
  list; and
- blocking backend calls run on GPUI's background executor and return owned
  answers to the entity context.

The correspondence is structural, not a dependency on Zed's `workspace`
crate. ADR 0002's dependency decision still holds: importing that crate would
also import the editor, project, collaboration, database, language, remote, and
node-runtime systems that zeddy does not use.

## Persistence

The folder registry lives at `$XDG_CONFIG_HOME/chartr-zeddy/spaces.toml`, with
platform fallbacks, file order as display order, duplicate suppression, and
unknown TOML keys preserved. Window bounds, chrome choice, pane trees, item
ownership, and restorable plugin state live in Chartr's SQLite state store. The
rewrite deliberately does not import or mutate older Chartr registries.

Ad-hoc sessions are the one synthetic space. They use the operator's home
directory and have no registry row. A registered home-directory row is not
drawn beside it because herdr has one workspace per directory; two labels over
one backend workspace would pretend to be independent state when they are not.

## Chrome

The sketches choose where items appear: grouped vertically in sidebar mode and
horizontally for the active space in tabs mode. Both reuse Zed `ui` components
for tabs, buttons, labels, icons, colors, focus tracking, and scroll containers.
There is no custom popup, menu state machine, or parallel widget kit.

## Consequence

Switching spaces is an entity-selection change. It cannot reparent a session,
reuse another space's selected index, or recreate backend work. Adding a third
chrome arrangement likewise cannot change the space model: it can only draw
the active child's entries somewhere else.
