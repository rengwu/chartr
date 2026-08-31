# 0001 — Sessions live in a private herdr

## Decision

Chartr runs its own Herdr daemon in a namespace it owns: its own socket, saved
shape, and log under `$XDG_CONFIG_HOME/chartr-zeddy/herdr` (normally
`~/.config/chartr-zeddy/herdr`). This matches the proven Chartr-rs namespace
shape. The executable is the sidecar vendored beside Chartr's binary and is
resolved by path. Chartr never discovers, attaches to, stops, upgrades, or
writes the user's own Herdr.

## Why

The backend is infrastructure zeddy hides, not a feature zeddy exposes. Sharing
the user's daemon would mean zeddy's sessions and the user's sessions in one
list, zeddy's version pin constraining the user's upgrades, and a `herdr server
stop` typed in one of zeddy's own terminals killing the window's backend.

Resolving by path rather than through `PATH` follows from the same thing: a
herdr the user installed is theirs, and picking it up would make zeddy's backend
version depend on the machine. The frame stream rides herdr's *command line*,
which carries no compatibility promise, so the version is pinned exactly rather
than as a floor.

`Namespace::env` sets only Herdr's config root and exact socket, then clears
`HERDR_SESSION`, `HERDR_PANE_ID`, and their siblings rather than merely
overriding what it sets. Chartr is frequently launched *from* a Herdr pane, and
an inherited selector would otherwise point a frame stream at a daemon the
control plane is not talking to.

## What this rules out

- Attaching zeddy to a session the user started in their own herdr. That is a
  real thing to want, and it is not free: it means two version pins, two
  lifetimes, and a shared list. It would be a new decision, not an extension of
  this one.
- A backend administration surface. There is nothing here for a user to
  configure, so there is no page for configuring it.

## Revisit if

Sharing a daemon with the user's own herdr becomes a request rather than a
hypothesis — at which point the namespace stays and gains a second, adopted
member rather than being removed.
