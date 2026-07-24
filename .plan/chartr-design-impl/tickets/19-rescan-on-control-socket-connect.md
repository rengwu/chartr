---
type: task
---

# Rescan on control-socket connect, so a degraded watch still discovers

## Question

Discovery is by notice (ticket 03): an fsnotify watch over each space's `.plan/`
subtree fires a debounced `rebuild`, and a watcher that cannot start "degrades to
action-driven discovery rather than failing chartr." That degradation is the
hole. `newWatcher` falls back to watching *nothing* — silently — when fsnotify
can't init (fd limits, a sandbox, an unusual filesystem), and an individual
`Add`/create event can be missed. In that state an operator sitting on the map
picker, taking no action, never sees a map or ticket created from outside: there
is no notice to rebuild on, and nothing they can click that would.

Close it at the one moment a user is provably present and looking: **when a
browser opens or reopens the control socket.** `s.handleControl` (`/ws/control`)
already re-sends the whole snapshot on every (re)connect — but that snapshot is
only as fresh as the last `rebuild`. Make the connect path run a discovery pass:
call `s.rebuild()` (the same function the watch and every operator action fire) as
the socket comes up, so the first snapshot a reconnecting browser receives
reflects the truth on disk, watch or no watch.

This is the whole fix — a fresh scan at connect. It is not a poll (no timer, no
periodic rescan), not a manual refresh button (discovery stays by notice, story
11 — this adds one more notice, it does not reintroduce ceremony), and not a
watch-health indicator; those were considered and set aside. Reconnect is a
natural, bounded trigger: a browser that lost its socket and comes back is
exactly when a stale snapshot is most likely and a user is most likely to be
staring at it.

Keep it cheap. `rebuild` is already debounced downstream and idempotent, and a
reconnect is not a hot path, so a plain `s.rebuild()` on connect is enough — no
new debounce, no dedupe against a concurrent watch rebuild beyond what `rebuild`
already tolerates.

Tests lead (process-boundary, as ticket 03's discovery tests already are): with
the watcher disabled — the degraded path, a `nil` fsnotify watcher — drop a
fixture map into a registered space from outside, then dial a fresh control
socket and assert the new map appears in the snapshot that connect delivers, with
no operator action and no watch notice behind it. A companion asserts the live
watch case is unregressed: with the watch running, an already-connected browser
still receives the map by notice as before.

Done when: a control-socket connect triggers a discovery rebuild, so a
newly-created map/ticket is in the first snapshot the browser receives even when
the filesystem watch never fired; the existing by-notice discovery is unchanged;
`go vet ./...` and `go test ./...` pass, and the frontend `check`/`build`/`vitest`
pass with no amber in the built CSS.
