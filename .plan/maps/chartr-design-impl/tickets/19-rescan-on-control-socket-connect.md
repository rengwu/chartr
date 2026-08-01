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

## Answer

One line of behaviour, as specified. `handleControl` (`internal/server/control.go`)
now calls `s.rebuild()` **before** `s.hub.subscribe()`. `rebuild` is synchronous
and ends in `s.hub.setModel(...)`, which writes `modelJSON` under the same lock
`subscribe` reads it under, so the snapshot the connect already sends *is* the
fresh one — no second push, no waiting, no new plumbing. No timer, no refresh
button, no watch-health surface; the by-notice path is untouched.

An unlooked-for second benefit, worth a reader knowing: `rebuild` also calls
`watch.setRoots`, which re-attempts any directory `Add` that previously failed.
So a connect repairs a *partially* degraded watch, not only a dead one.

**The test seam.** fsnotify's init cannot be made to fail portably, so the
degraded path needed a deliberate door: `Options.NoWatch` starts the server with
the same nothing-watching watcher `newWatcher` already falls back to. That
fallback got a name — `deadWatcher(pinned)` in `watch.go` — and `newWatcher` now
builds on it, so there is one shape for "no OS watcher behind this" rather than
two. `chartrtest.WithoutWatch()` is the rig option. `setRoots` and `close` were
already nil-safe; nothing else needed guarding.

**Tests** (`internal/server/rescan_test.go`, process-boundary like ticket 03's):

- `TestConnectRescansWhenWatchIsDead` — with `WithoutWatch()`, assert the space
  starts mapless, drop a map in from outside, then dial and read the *first*
  snapshot. No `WaitFor`: no notice is ever coming, so waiting would only mask a
  failure as a timeout. Confirmed to fail (`maps = []`) with the `s.rebuild()`
  line commented out, so it is testing the fix and not the fixture.
- `TestWatchStillDiscoversForAConnectedBrowser` — the live watch unregressed: an
  already-connected browser receives a map by notice without reconnecting.

**Verified:** `go vet ./...` and `go test ./...` pass (the full server suite,
including ticket 03's existing `TestMapAppearsByNoticeBothLayouts`). No frontend
change was needed, and the frontend gates pass unchanged anyway —
`svelte-check` 0 errors, `vitest` 211/211, `npm run build` clean.
