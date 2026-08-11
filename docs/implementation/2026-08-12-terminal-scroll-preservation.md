# Terminal scroll preservation

**Implemented:** 2026-08-12

## Why this work was needed

Terminal reading position was unreliable in two common situations:

1. A busy agent could eventually evict the history being read above its live
   output.
2. Switching sessions destroyed the active xterm instance and rebuilt it from a
   bounded raw-byte replay, losing the viewport and other client-only state.

We began from an earlier handoff diagnosis, then checked the relevant chartr and
xterm sources independently. Two findings drove the implementation:

- An unset `scrollback` preference really did fall through to xterm's 1,000-line
  default. Once that buffer is full, xterm must evict old lines and can no longer
  hold a reader's anchor indefinitely.
- `SpacePane` keyed the one terminal component on the active terminal ID. Every
  session switch therefore closed its WebSocket, disposed xterm, and created a
  fresh instance. Server replay restores some output, but cannot restore the
  exact client viewport, selection, or find state.

We did not accept the handoff's proposed resize fix. The installed FitAddon
already skips a resize when the calculated rows and columns have not changed,
and the surrounding cockpit header has a fixed height. Adding the same guard in
chartr would have duplicated existing behavior without addressing the reset.

## Decisions

### Use a 10,000-line chartr default

We initially considered 20,000 lines, then chose 10,000 as the safer default
when combined with retained terminals. In xterm's core buffer, the lower-bound
cell storage is approximately `lines × columns × 12 bytes`; 10,000 lines is
about 9.6 MB at 80 columns or 14.4 MB at 120 columns before JavaScript and
renderer overhead.

Ten thousand lines provides a large improvement over xterm's 1,000-line default
without making each filled terminal unnecessarily expensive. An explicit
positive `scrollback` value in `terminal.toml` still overrides the default.

The server's 256 KiB raw replay buffer was not changed. It is a different budget
measured in PTY bytes rather than rendered lines, and increasing it would not by
itself make replay a correct terminal snapshot.

### Retain visited terminals only in the selected space

Visited terminals now remain mounted when the operator selects another session
in the same space. The implementation is deliberately simple:

- A terminal joins the retained pool on first visit, so unvisited sessions do not
  open a renderer or WebSocket.
- Inactive terminals are hidden rather than destroyed.
- Switching spaces clears the pool naturally through component removal.
- Closing a terminal disposes its component through the keyed terminal list.
- There is no LRU, terminal-count heuristic, or global cross-space cache.

This preserves the complete xterm instance—buffer, viewport, selection, find
state, renderer, and socket—during the session switches that caused the reported
problem. It also bounds retained resources to visited terminals in the one space
currently on stage.

### Keep terminal preference remounting broad and simple

The existing `JSON.stringify(terminalPrefs ?? {})` key remains the only terminal
preference identity. Any resolved terminal preference change remounts the whole
retained pool. We did not add per-setting comparisons or hot-update paths.

Terminal preference changes are rare, and a full remount preserves the existing
mount-time configuration contract. Unrelated application preferences do not
affect the pool.

### Fit only when useful

Inactive terminals keep consuming their live PTY stream, but their
`ResizeObserver` does not call FitAddon. Resizing every hidden terminal during a
window drag would resize every PTY and could make several background TUIs repaint
at once.

When a retained terminal becomes active, chartr fits it on the next animation
frame and then focuses it. We deliberately did not pre-fit on session-row hover:
hover is incidental, is unavailable to keyboard users, and a fit can mutate the
PTY geometry and trigger agent redraws.

## Implementation

### Scrollback resolution

[`web/src/lib/tokens.ts`](../../web/src/lib/tokens.ts) now exports
`DEFAULT_TERMINAL_SCROLLBACK = 10_000` and uses it whenever the resolved
preference is unset or zero.

The effective-settings display and scaffold were updated to agree with the
runtime:

- [`web/src/lib/terminalsummary.ts`](../../web/src/lib/terminalsummary.ts)
- [`internal/config/terminal.scaffold.toml`](../../internal/config/terminal.scaffold.toml)
- [`web/src/lib/tokens.test.ts`](../../web/src/lib/tokens.test.ts)
- [`web/src/lib/terminalsummary.test.ts`](../../web/src/lib/terminalsummary.test.ts)

The operator's existing `~/.config/chartr/terminal.toml` was not edited. Its
`scrollback = 1000` example is commented out, so the new chartr default applies
despite the stale local comment.

### Retained terminal pool

[`web/src/lib/SpacePane.svelte`](../../web/src/lib/SpacePane.svelte) now records
which terminal IDs have been visited in the current space. It renders those
terminals as keyed, full-pane layers under the single terminal-preference key.

Only the active layer is visible. Inactive layers are:

- `visibility: hidden` through the existing utility class;
- pointer-inert;
- marked `aria-hidden`;
- given the HTML `inert` attribute.

Using hidden full-size layers instead of `display: none` leaves each retained
terminal with measurable pane dimensions while keeping it out of interaction and
accessibility traversal.

### Terminal activation behavior

[`web/src/lib/Terminal.svelte`](../../web/src/lib/Terminal.svelte) now accepts an
`active` prop. It uses that state to:

- set xterm's `disableStdin` option while inactive;
- blur the old terminal immediately on deactivation;
- skip FitAddon work from hidden-terminal resize observations;
- fit and focus the newly active terminal on the next animation frame;
- focus at initial mount only when the terminal is active;
- clear retained xterm/FitAddon references during disposal.

The WebSocket stays connected and output continues to be parsed while a visited
terminal is hidden. This is necessary to keep its xterm state current when the
operator returns.

## Performance and stability safeguards

- The default is 10,000 rather than 20,000 lines.
- Terminals are mounted lazily on first visit.
- The pool is scoped to the selected space.
- Hidden terminals do not refit during window or pane resizing.
- Hidden terminals cannot receive keyboard input or retain focus.
- Hidden terminal DOM is removed from pointer and accessibility interaction.
- Closing a terminal, switching spaces, or changing terminal preferences follows
  an explicit disposal path.

The expected tradeoff is that every retained, running terminal still uses memory,
a WebGL renderer, one local WebSocket, and CPU to parse its output. This is the
cost of preserving exact live client state. The current-space and lazy-mount
boundaries keep that cost straightforward and observable without adding cache
policy complexity.

## Validation

The completed implementation passed:

- `npm run check` — 0 errors and 0 warnings;
- `npm test` — 17 test files and 191 tests passed;
- `npm run build` — production build completed successfully.

A live browser interaction check was attempted, but no controllable browser
instance was available in the environment. The production build and Svelte
diagnostics verified the DOM attributes, component props, and generated client
bundle, but session-switch interaction should still be exercised manually in the
running cockpit.

## Deliberate non-goals and remaining work

- Scroll state is retained across sessions in the same space, not across space
  switches.
- The server still replays a bounded 256 KiB suffix of raw PTY bytes after a real
  reconnect or remount. That suffix can begin mid-sequence and is not a complete
  terminal snapshot.
- Unexpected WebSocket closure still has no frontend reconnect handler or visible
  disconnected state.
- The server replay cap was not increased.
- No extra resize guard was added beyond FitAddon's existing dimension check.
- No hover-prefit, global terminal pool, LRU, or per-preference remount logic was
  introduced.

These are separate concerns. They should be addressed only with their own
reproduction, protocol design, and resource budget rather than folded into the
scroll-preservation fix.
