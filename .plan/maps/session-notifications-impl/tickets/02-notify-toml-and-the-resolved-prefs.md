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
