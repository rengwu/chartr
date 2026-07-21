# wayfinder-harness

A cockpit for driving [wayfinder](https://github.com/) maps to completion: switch
between project spaces, read a map as a star-map, and spawn agent sessions against
its frontier — with implementation work gated behind review.

The harness runs as one self-contained binary that serves its interface in your
browser. Point it at a directory of git repositories, open the cockpit, and drive
maps to done.

```
harness            # serves the cockpit on http://127.0.0.1:8787
harness -addr :9000
harness -version
```

## What you get: the support tiers

The harness ships **one supported artifact**, and everything else is a
**best-effort tier** that may be absent without anything being wrong. This is a
deliberate boundary ([ADR 0011](docs/adr/0011-one-supported-artifact-tiered-extras.md)),
not an accident of what happened to build.

### Supported — the browser-serving binary

The one supported artifact is the **pure-Go binary that serves the browser
frontend** from its embedded frontend build. It is cross-compiled for **macOS,
Linux, and Windows** from a single **cgo-free** CI job — nothing in it requires
cgo — and every release is **checksummed** (`checksums.txt`, SHA-256).

"Supported" means: this is the artifact the release pipeline must produce,
green, for all three operating systems before a tag ships. If you want the
harness, this is what you download.

### Best-effort — the native webview shells

Each platform can also have a **native webview shell** — a desktop window around
the same cockpit instead of a browser tab (cgo + WebKitGTK on Linux, cgo on
macOS, cgo-free `go-webview2` on Windows). It is a second binary,
`wayfinder-harness-shell_<version>_<os>_<arch>`, that runs the same server
in-process on a random loopback port and points a real window at it: a dock icon,
a minimal native menu (Quit, Reload, the edit items), and one window per
`--data-dir` — a second launch raises the running one. If the native runtime is
missing it says so and points you back at `harness`; it never silently opens a
browser. Build it yourself with `make webview`
([ADR 0013](docs/adr/0013-webview-shell-architecture.md)).

These are **best-effort**: they are built in a separate, non-blocking CI lane and
**attached to a release only for the platforms whose toolchains built them**,
each with its own `.sha256` sidecar rather than an entry in the supported
`checksums.txt`. A missing shell asset for your platform means its toolchain did
not build that release — the supported binary for that platform is unaffected. A
shell build failure never fails a release.

### Windows: native is best-effort, WSL2 is the sure path

Native Windows is a **best-effort tier by decision**, not an afterthought: the
supported binary is built for Windows and its ConPTY-backed PTY layer is
smoke-tested in CI on every change ([ADR 0006](docs/adr/0006-go-xtermjs-canvas-browser-first.md)
as amended), but native Windows is not driven daily. **WSL2 is the documented
sure path** — if you want the smoothest Windows experience, run the Linux binary
under WSL2.

## Distribution

**GitHub releases only.** Download the checksummed binary for your platform from
the [releases page](https://github.com/rengwu/wayfinder-harness/releases), verify
it against `checksums.txt`, and run it.

There is deliberately **no `go install`, no Homebrew tap, and no plugin
marketplace entry** — the last declined on purpose: this is an agent-agnostic
tool, not something distributed through one agent's storefront. These channels
stay cheap to add later once tagged releases exist; declining them now forecloses
nothing.

## Cold start: what works with nothing installed

A fresh download with **zero agent CLIs installed works everywhere except one
thing: spawning a session.** You can register spaces, browse maps as star-maps,
read tickets, open ad-hoc shells, and drive the review hub — all of it works cold.

The agent CLIs (Claude Code, Codex, and friends) are **not the harness's to
ship.** When you try to spawn a session against a role whose agent is not
installed, the harness **hard-blocks at spawn time with a message that names
exactly what is missing** — and that block message doubles as your installer's
to-do list. There is **no separate doctor command**: the environment diagnosis is
the registry badge and the spawn-time block, surfaced at the moment of need
rather than as ceremony off to the side.

## Building from source

You need Go 1.26+ and Node 22+.

```
make build     # builds web/dist, then the self-contained binary → bin/harness
make check     # go vet + svelte-check
make test      # the Go process-boundary suite
make snapshot  # build the supported release binaries locally (goreleaser, no publish)
```

The frontend is a Svelte SPA ([ADR 0010](docs/adr/0010-svelte-chrome-imperative-islands.md))
built by Vite and `go:embed`ed into the binary, so the shipped artifact is one
offline file with no CDN and no runtime fetch. See
[docs/design-system.md](docs/design-system.md) before touching any UI.

## Documentation

- [CONTEXT.md](CONTEXT.md) — the glossary and the concepts the cockpit is built on
- [docs/adr/](docs/adr/) — the architecture decision records, including
  [ADR 0011](docs/adr/0011-one-supported-artifact-tiered-extras.md) on the
  release tiers this README states
- [docs/design-system.md](docs/design-system.md) — the frontend design system
