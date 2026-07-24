---
type: task
blocked_by: []
---

# Walking skeleton — the binary serves the cockpit

## Question

One self-contained binary serving the cockpit's shell: the Go backend embeds the Vite-built Svelte 5 SPA (ADR 0010), answers operator actions over plain HTTP, and pushes the whole derived model as a JSON snapshot over the control socket on every change, resending the snapshot on reconnect. The model can be near-empty — the slice is the transport and delivery skeleton every later ticket hangs state on. Wire the dev loop (Vite dev server with HMR, proxying websockets to the running backend) and the repo's static checks and test scripts.

This ticket also establishes the **process-boundary test rig** the whole map's testing leans on (spec, Testing Decisions): a Go test starts the real binary against a temporary directory, connects to the control socket, performs HTTP actions, and asserts on snapshots — plus helpers for fixture git repos that later tickets will reuse.

Done when: running the binary and opening a browser shows the empty cockpit shell served from the embedded build; the rig's first tests are green — a snapshot arrives on connect, and a dropped connection gets the full snapshot again on reconnect.

## Answer

One Go binary serves the cockpit shell and pushes the derived model; the process-boundary rig is stood up. Layout:

- **`cmd/chartr`** — a thin `main` (construct → listen → serve) so tests exercise the operator's exact code path.
- **`internal/server`** — `Server` (routes, graceful shutdown), the control-socket `hub` (broadcaster: snapshot-on-connect, whole-snapshot on change, slow-consumer drop), and the `/ws/control` handler. Transport per ADR 0010: JSON control socket (whole-model snapshot, resend on reconnect), operator actions as plain HTTP (`GET /api/health` is the skeleton's one action; register/classify/spawn/approve hang off this mux later). The control socket is push-only — `CloseRead` drains client frames and detects disconnect.
- **`internal/model`** — the derived `Model` pushed to every browser; near-empty (`{"spaces":[]}`) by design, the shape later tickets grow.
- **`web/`** — Svelte 5 + Vite + TS SPA, no SvelteKit (ADR 0010). The chrome (`App.svelte`) is a sidebar / stage / status-bar shell reacting to the pushed model; `control.svelte.ts` owns the one control socket with runes state and reconnect-with-backoff. `go:embed all:dist` folds the Vite build into the binary; a committed empty `web/dist/.gitkeep` keeps `go:embed` compiling on a fresh checkout, `make web` fills in the real assets (postbuild restores the keepfile). Dev loop is Vite HMR proxying `/api` and `/ws` to the backend.
- **`internal/chartrtest`** — the reusable rig: `Start` runs the real server on a random loopback port with a temp `DataDir`; `DialControl`/`ReadSnapshot` and `Get` drive it from outside; git-repo + map fixtures (`NewSpaceRepo`, `Git`, `WriteMap`) are seeded for tickets 02–03. No test-only interface on the product — the seam is the process boundary.

Against Done-when: `internal/server/skeleton_test.go` asserts a snapshot arrives on connect and the whole snapshot is resent on reconnect (plus two-browser fan-out and the HTTP action), green under `-race`; `go vet ./...`, `go test ./...`, and `svelte-check` (0 errors) pass; `make build` produces an 8.9 MB binary that serves the embedded `index.html`, its hashed assets, and SPA-fallback deep links (verified over HTTP) — opening it in a browser shows the empty cockpit shell.

Review payload should carry this Done-when and the spec by assembly (spec, Prompts and payload).
