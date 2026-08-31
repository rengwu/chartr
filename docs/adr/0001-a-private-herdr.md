# 0001 — Sessions live in a private herdr

## Decision

zeddy runs its own herdr daemon in a namespace it owns: its own socket, XDG
directories, session name, and log, all under `~/.local/state/zeddy/herdr`. The
executable is the one vendored beside zeddy's binary, resolved by path. zeddy
never discovers, attaches to, stops, upgrades, or writes the user's own herdr.

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

`Namespace::env` clears `HERDR_SESSION`, `HERDR_PANE_ID`, and their siblings
rather than merely overriding what it sets. zeddy is frequently launched *from*
a herdr pane, and an inherited selector would otherwise point a frame stream at
a daemon the control plane is not talking to.

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
