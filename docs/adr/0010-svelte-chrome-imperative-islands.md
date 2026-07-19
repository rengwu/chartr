# Svelte 5 chrome around imperative islands; embedded dist; two-socket transport

The frontend is a **Svelte 5 SPA over Vite** (no SvelteKit — no routing, no SSR), in TypeScript. The framework owns only the **chrome**: the sidebar, tabs, queue, brief, panes — the dozen small views that react to server-pushed state. The two hard surfaces are **imperative islands** the chrome hosts but never reaches inside: xterm.js terminals, and the star-map — one coherent canvas renderer behind a narrow seam (mount, receive model, emit selection), never decomposed into components.

The star-map renderer is **reimplemented cleanly in TypeScript**, not line-ported: ADR 0001's "copy freely where it elevates" is permission, not obligation, and the harness's renderer diverges too far (pushed state, session moons, pane-aware camera) for the old polling-shaped structure to be a head start. The references stay open on the desk — wayfinder-maps' renderer, `starmap-design.md`, the design map's prototypes — and tuned constants (easing, zoom coupling, parallax, dpr) are cribbed directly, because feel-drift is the one real risk of a rewrite.

The built `dist/` is **embedded in the Go binary** (`go:embed`), keeping distribution a single self-contained file; development uses Vite's dev server with HMR, proxying websockets to the Go backend.

State moves over **two websocket kinds**, both protocols hand-rolled over a standard Go websocket library:

- A **control socket** per browser: JSON, server-authoritative, the whole derived model pushed as a snapshot on every change. The model is small; diffing buys nothing; reconnect is "resend snapshot".
- A **terminal socket** per attached terminal: raw PTY bytes down to xterm.js, keystrokes up, server-buffered scrollback replayed on attach (ADR 0006).

Operator actions (approve, spawn, abandon) default to plain HTTP request/response so failures surface as responses.

## Consequences

- The frontend build step ADR 0006 conceded now has a definite shape: Vite, producing a `dist/` the Go build embeds. Shipping (ticket 13) packages one binary per platform, nothing else.
- Ticket 09's moons, beacon and ticker are written inside the renderer's idiom, not as Svelte components; the seam's pushed model is where session state enters the star-map.
- A terminal that floods cannot head-of-line-block map updates — the streams never share a connection.

## Considered options

- **Plain TypeScript, no framework** — the austere option; loses only on the chrome, where a dozen push-fed live views make hand-written DOM mutation the bug farm.
- **Preact/React** — virtual DOM around imperative islands means refs and effect discipline everywhere, heaviest runtime, no compensating gain.
- **Line-porting or vendoring the renderer** — preserves feel by construction, but shoehorns pushed harness state through polling-era structure; rejected in favour of reimplementation with constants cribbed.
- **Serving the frontend separately** — buys independent frontend releases a single-operator local cockpit doesn't need; costs ticket 13 a second artifact.
- **One multiplexed socket** — hand-built framing plus head-of-line risk when a terminal floods, solving a connection scarcity a local cockpit doesn't have.
- **A sync library (socket.io-style, CRDT)** — reconnection rooms and merge semantics for a client that never writes state through the socket; the server is sole authority.
