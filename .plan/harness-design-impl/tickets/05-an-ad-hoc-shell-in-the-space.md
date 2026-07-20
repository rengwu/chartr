---
type: task
blocked_by: [02]
---

# An ad-hoc shell in the space

## Question

The multiplexer baseline: a "+" by the session tabs opens a real shell in the space's working tree — a PTY built on the cross-platform, ConPTY-capable library (ADR 0006 as amended), streamed over its own binary terminal socket to an xterm.js island the Svelte chrome hosts but never reaches inside. Raw bytes down, keystrokes up, server-side scrollback replayed on attach; the terminal socket never shares a connection with the control socket, so a flooding terminal cannot block map updates. Ad-hoc shells are deliberately outside the session model — no ticket, no lifecycle, ended by the human. Tabs seat the space's terminals in the full-width terminal column.

Done when: in the browser, a shell opens in the working tree, echoes keystrokes, and survives detach/reattach with scrollback replayed; process-boundary tests drive the terminal socket directly (spawn, write, read back, reattach and assert replay); a mapless space is fully usable this way.

## Answer

An ad-hoc shell now opens in a space's working tree and streams over its own binary terminal socket, and a mapless space is fully usable as a plain multiplexer. Ad-hoc shells are deliberately outside the session model (spec, State model): no ticket, no lifecycle, ended only by the human — they share nothing with a real session but the PTY primitive. Layout:

- **`internal/terminal`** — a new package that owns the harness's PTYs, built from day one on `aymanbagabas/go-pty` so the session core never ossifies unix-only (ADR 0006 as amended — ConPTY under `COMSPEC` on Windows, `$SHELL`/`/bin/sh` on unix). A `Terminal` is a running shell on a PTY plus a bounded server-side scrollback (256 KiB) and the set of sockets watching it; its read loop copies raw output into scrollback and fans it out to every attached socket, and `Attach` captures scrollback and registers the socket under one lock so no byte is dropped or duplicated at the seam. A socket that falls behind is killed and left to reattach-and-replay rather than back-pressuring the read loop — the same slow-consumer policy the control hub uses, and the reason the two socket kinds never share a connection (ADR 0010). A `Manager` keys terminals by id, groups them by space in open order, and pushes a fresh model whenever one opens or ends, so a tab appears and disappears by notice.
- **`internal/model`** — `Space` grows `Terminals []Terminal` (id, tab title, `alive`): harness-owned runtime state folded into the snapshot so tabs render and a reconnecting browser rediscovers open shells. The raw bytes never ride this snapshot — they travel on the terminal socket keyed by id. Terminals are explicitly not sessions.
- **`internal/server`** — `POST /api/spaces/{id}/terminals` opens a shell (a plain HTTP action, so a shell that will not start surfaces as a response — ADR 0010); `DELETE …/{termID}` ends one on the human's command; `/ws/terminal/{termID}` is the binary socket that replays scrollback as the first frame, then streams raw PTY bytes down as binary frames, carrying keystrokes up as binary frames and a resize up as a small text-JSON control message. `Serve` drains terminals on shutdown; `deriveSpace` folds a space's terminals into every rebuild.
- **`web/`** — `Terminal.svelte` is the imperative xterm.js island the chrome hosts but never reaches inside (ADR 0010): raw bytes written straight into xterm, `onData` keystrokes sent as binary, a fit addon reflowing the PTY on resize. `SpacePane` grows the terminal column — a tab strip with per-tab close and a "+", the active island keyed so a tab switch remounts cleanly, and an empty-state open button so a mapless space opens its first shell in one click; the role bindings move into a collapsible panel beneath it.

Against Done-when: `internal/server/terminal_test.go` extends the process-boundary rig (a `TerminalConn` with `Send`/`ReadUntil`, plus `OpenTerminal`) and drives the socket directly — a shell runs `echo mark-$((6*7))` and the socket reads back `mark-42`, so a match proves the *command executed* (keystrokes up, bytes down), not merely that the PTY echoed input; detach-then-reattach replays the buffered scrollback on a fresh socket; a mapless space (no `.plan/`) surfaces an alive terminal tab in the snapshot; ending a shell over HTTP drops its tab from the pushed model by notice; the socket refuses an unknown id (404). `go vet ./...`, `go test -race ./...`, `svelte-check` (0 errors), and the Vite build pass.

Two decisions for review to weigh:

- **The up-channel splits by frame kind** — binary frames are keystrokes straight to the PTY, a text frame is a resize control message. ADR 0006/0010 name only "keystrokes up"; resize needs a channel and this keeps the hot path (keystrokes) frameless while giving control messages a typed home, rather than inventing a byte-level framing over the keystroke stream. A malformed control frame is ignored, never wedging the socket.
- **Terminals live in the pushed model** — they are harness runtime state, not derived from disk, so `deriveSpace` folds `Manager.ForSpace` into each rebuild. This is what makes tabs appear/disappear by notice and lets a second browser see them, at the cost of the model no longer being a pure function of the filesystem. The alternative (a terminal list on a side channel) buys purity a single-operator cockpit does not need.

Scope notes for review: browser detach/reattach is exercised as the frontend island (one socket per mount, scrollback replays on remount); the automated detach/reattach assertion is at the socket, which is the same server path. A dead shell's tab greys and is dismissed by the human — this is the ad-hoc analogue of story 37's pinned scrollback, not the session-death halt (a later ticket). Killing a shell kills the shell process; child-process reaping beyond it is the operator's sandbox, out of scope (spec).

Review payload should carry this Done-when and the spec by assembly (spec, Prompts and payload).
