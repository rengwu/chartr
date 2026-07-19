---
type: task
blocked_by: [02]
---

# An ad-hoc shell in the space

## Question

The multiplexer baseline: a "+" by the session tabs opens a real shell in the space's working tree — a PTY built on the cross-platform, ConPTY-capable library (ADR 0006 as amended), streamed over its own binary terminal socket to an xterm.js island the Svelte chrome hosts but never reaches inside. Raw bytes down, keystrokes up, server-side scrollback replayed on attach; the terminal socket never shares a connection with the control socket, so a flooding terminal cannot block map updates. Ad-hoc shells are deliberately outside the session model — no ticket, no lifecycle, ended by the human. Tabs seat the space's terminals in the full-width terminal column.

Done when: in the browser, a shell opens in the working tree, echoes keystrokes, and survives detach/reattach with scrollback replayed; process-boundary tests drive the terminal socket directly (spawn, write, read back, reattach and assert replay); a mapless space is fully usable this way.
