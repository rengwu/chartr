---
type: task
blocked_by: [01]
claimed_by: s2d5179cf39c6
claimed_at: 2026-07-30T18:36:06Z
---

# `notify.toml` and the resolved prefs

## Question

Give the clock its constants from a file the operator owns. `notify.toml` is a
per-machine file beside `terminal.toml` in the chartr data dir, carrying `after`
(*n*), `settle` (*D*) and `enabled`, resolved into the model snapshot and wired
into ticket 01's machine.

**Follow `terminal.toml`'s contract exactly**, because it is already the right
one and a second config philosophy would be the real cost here: never committed,
never per-space, read on every rebuild into the snapshot, and a bad value dropped
with a warning through the same warnings surface spaces already use while its
default stands. A malformed file never breaks the cockpit. Leaving a key out is
distinct from setting it — the tri-state rule `terminal.toml` documents applies.

**It is a new file, deliberately.** Not a section of `terminal.toml`, which is
scoped to terminal *customization*; not `user.toml`, which ADR 0009 reserves for
execution choices. Say so in the file's own header comment, the way
`terminal.toml` explains itself, and reproduce the defaults as commented keys so
copying the file as-is changes nothing.

**Surface it where the others are surfaced.** The existing config surface shows
per-machine files as read-value-plus-open-file; `notify.toml` joins them on the
same terms. It is never a second config store and there is no settings form that
writes it.

**`enabled = false` stops events at the source.** The clock does not run and no
consumer is reached, rather than each consumer checking a flag — one place to turn
it off, and no possibility of the dot and the notification disagreeing.

Tests lead, following the prior art in `internal/config`'s `terminal.toml` tests:
absent file yields the documented defaults; each malformed value (a negative
duration, an unparseable duration, a wrong type) yields the default plus exactly
one warning naming the key; a valid file resolves through to the model snapshot;
`enabled = false` yields a clock that emits nothing. Assert the warning text is
actionable — it names the key and the file — because the operator cannot fix what
the warning does not identify.

Done when: `notify.toml` is read, validated, defaulted and surfaced like
`terminal.toml`; its values reach ticket 01's machine; a malformed file warns and
falls back rather than breaking anything; `enabled = false` silences the clock at
source; `go vet ./...`, `go test ./...` and the frontend `check`, `build` and
`vitest` scripts pass, with no amber in the built CSS. No notification and no dot
in this ticket.

## Answer

`notify.toml` now owns the machine-wide run-clock preferences end to end. The
operator file lives beside `terminal.toml`, is reread on every rebuild, resolves
into the pushed model, and configures the per-tab `runClock` instances that fold
published terminal states.

**What shipped.**

- `internal/config/notify.go` resolves `after`, `settle`, and `enabled` to concrete
  defaults of 60s, 10s, and true. Missing keys stay distinct from explicit values,
  including `enabled = false`. Durations must be positive Go duration strings.
  Invalid values fall back independently, with exactly one actionable warning
  naming `notify.toml` and the bad key; malformed TOML falls back wholesale with a
  file-level warning, and unknown keys warn without breaking known neighbours.
- `notify.scaffold.toml` is the inert starter the config surface creates. Its
  header says why this is neither terminal customization (`terminal.toml`) nor an
  execution choice (`user.toml`), and all three shipped defaults are reproduced as
  commented keys, so creating the file changes nothing.
- The snapshot carries the complete resolved values as `Model.Notify`; parse
  warnings join terminal warnings on each space. The config-root watcher already
  drives rebuilds, and a process-boundary test proves an external save changes the
  pushed value without a refresh.
- `notify-config` joins the named global config layers and create/open endpoints.
  The global Settings surface shows the three parsed values and the owning file,
  using the same read-value-plus-open-file pattern as terminal customization.
  There is no form or second store that writes preference values.
- `terminal.Manager` now owns the resolved constants and seats one clock per tab.
  Every sample folds the state the terminal actually published, downstream of
  detection and publisher hysteresis, into the single `RunFinished` callback seam
  tickets 03 and 04 will consume. Rebuilds with unchanged preferences preserve an
  in-progress clock; a real threshold edit updates it in place. Turning the
  feature off removes the clock itself, and re-enabling starts a fresh one, so no
  consumer needs an enabled check and the two later consumers cannot disagree.
  The 60s/10s defaults are defined once in `internal/config` and re-exported beside
  `runClock`.

**Tests.** Config tests cover the absent file, every valid value, a negative
duration, an unparseable duration, duration and boolean type errors, independent
fallback beside valid neighbours, and the scaffold's inert documented defaults.
Server tests cover snapshot resolution, warning propagation, watch-driven reread,
named-layer open, and scaffold creation. The terminal test proves custom constants
reach a seated clock, unchanged rebuilds do not reset it, and disabled means the
clock is nil at the source.

`go vet ./...`, `go test ./...`, `npm run check`, `npm run build`, and `npm test`
all pass. The built CSS contains no `amber`. No OS notifier, notification content,
attention dot, or seen endpoint was added; those remain tickets 03 and 04.
