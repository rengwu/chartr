# web — the cockpit chrome

The Svelte 5 SPA (Vite, TypeScript, no SvelteKit — ADR 0010). The framework owns
only the chrome; xterm.js terminals and the star-map arrive as imperative
islands in later tickets. The build output in `dist/` is embedded into the Go
binary by the `web` package (`../web/embed.go`).

## Develop

Run the Go backend and the Vite dev server side by side:

```sh
# terminal 1 — the harness backend on :8787
go run ./cmd/harness

# terminal 2 — Vite with HMR, proxying /api and /ws to the backend
cd web && npm install && npm run dev
```

Open the URL Vite prints. `/api` and `/ws` (the control and terminal sockets)
are proxied to the Go backend, so the browser only ever speaks to one origin.
Point at a backend on another port with `HARNESS_BACKEND`.

## Build (what the binary embeds)

```sh
cd web && npm run build   # → web/dist, embedded by go:embed
```

`make build` from the repo root runs this before compiling the Go binary.

## Check

```sh
cd web && npm run check   # svelte-check over the chrome
```

## Toolchain note

`typescript` is pinned to `^5` on purpose: `svelte-check@4` crashes against the
TypeScript 7 native port. Vite transpiles with esbuild/rolldown (not `tsc`), so
this pin only governs type-checking — revisit it when `svelte-check` supports
TS 7.
