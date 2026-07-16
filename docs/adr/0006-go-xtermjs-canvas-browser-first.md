# Go backend, xterm.js panes, canvas star-map, browser-first

The harness is a Go backend — reusing wayfinder-maps' model layer directly, plus `creack/pty` for terminals, `fsnotify` for watching `.plan/`, and process supervision per space — talking over websockets to a web frontend that renders terminals with **xterm.js** and the star-map on **canvas**. It serves a browser by default, with a native webview shell as a second front end: the split wayfinder-maps already proves.

The harness owns PTYs directly rather than delegating to tmux, and renders terminals with xterm.js rather than embedding libghostty or hand-rolling a renderer over a Go VT emulator. Terminal emulation is not this project's value; orchestrating wayfinder maps is.

## Considered Options

- **libghostty embedded, star-map in a side webview** — better terminal rendering, but forces per-platform native composition of a GPU surface beside a webview, kills the browser shell, and does not actually deliver the visual cohesion that motivated it (the seam between the two rendering worlds survives). Rejected once the target was confirmed cross-platform and distributed.
- **A custom canvas renderer over a Go VT emulator** — would unify the cockpit's visual language and preserve the no-build ethos, but signs up for unicode widths, ligatures, scrollback, selection, IME, reflow-on-resize and mouse reporting.
- **Tauri or Electron** — loses direct reuse of the Go model layer.
- **A pure TUI** — a multiplexer is a terminal-native idea, but the star-map is the point of the thing.
- **tmux as the session substrate** — buys crash survival and attaching to a stuck agent from your own terminal. Deferred rather than dismissed; a persistent Go daemon covers most of it.

## Consequences

- xterm.js is an npm dependency, so a frontend build step exists. wayfinder-maps' no-build ethos does not survive.
- Raw PTY output must be buffered server-side so a reconnecting browser can replay scrollback.
