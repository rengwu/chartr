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
version depend on the machine. Direct terminal attachment rides Herdr's *command line*,
which carries no compatibility promise, so the version is pinned exactly rather
than as a floor.

The pin may be an immutable upstream source revision when a required backend
fix has not reached a release. In that case the acquisition script gives the
binary a revision-specific build version, and the ordinary version/protocol
handshake rejects released or locally built binaries that merely share the
same package version. This is currently required for semantic direct-attach
mouse forwarding: released Herdr 0.8.2 consumes click reports instead of
forwarding them according to the attached child's active mouse mode.

Because the daemon can outlive the Chartr binary that launched it, a pin change
uses Herdr's live-handoff API when the private socket is occupied by an
incompatible version. The replacement sidecar inherits the live PTYs; Chartr
never stops an old daemon merely to upgrade it.

`Namespace::env` sets only Herdr's config root and exact socket, then clears
`HERDR_SESSION`, `HERDR_PANE_ID`, and their siblings rather than merely
overriding what it sets. Chartr is frequently launched *from* a Herdr pane, and
an inherited selector would otherwise point an attach client at a daemon the
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
