---
type: task
blocked_by: []
---

# Walking skeleton — the binary serves the cockpit

## Question

One self-contained binary serving the cockpit's shell: the Go backend embeds the Vite-built Svelte 5 SPA (ADR 0010), answers operator actions over plain HTTP, and pushes the whole derived model as a JSON snapshot over the control socket on every change, resending the snapshot on reconnect. The model can be near-empty — the slice is the transport and delivery skeleton every later ticket hangs state on. Wire the dev loop (Vite dev server with HMR, proxying websockets to the running backend) and the repo's static checks and test scripts.

This ticket also establishes the **process-boundary test rig** the whole map's testing leans on (spec, Testing Decisions): a Go test starts the real binary against a temporary directory, connects to the control socket, performs HTTP actions, and asserts on snapshots — plus helpers for fixture git repos that later tickets will reuse.

Done when: running the binary and opening a browser shows the empty cockpit shell served from the embedded build; the rig's first tests are green — a snapshot arrives on connect, and a dropped connection gets the full snapshot again on reconnect.
