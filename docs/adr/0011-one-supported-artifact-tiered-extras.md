# One supported artifact; everything else is best-effort

The harness ships as **one supported artifact**: the pure-Go binary that serves the browser frontend from its embedded Vite `dist/` (ADR 0010) — cross-compiled for macOS, Linux and Windows from a single CI job, because nothing in it requires cgo. Everything beyond it is a **best-effort tier** that may fail to build without blocking a release: the native webview shell per platform (cgo + WebKitGTK on Linux, cgo on macOS, cgo-free `go-webview2` on Windows), and native Windows itself (built and smoke-tested in CI, not driven daily; WSL2 is the documented sure path).

Distribution is **GitHub releases only**, goreleaser-built and checksummed, with best-effort shells attached as extra assets where they built. Declined: `go install`, Homebrew, and a Claude Code plugin marketplace entry — the last deliberately, an agent-agnostic tool not distributed through one agent's storefront.

There is **no doctor command**. The environment diagnosis is the registry badge and the spawn-time hard-block message (ticket 05), surfaced at the moment of need. A cold start with zero agent CLIs installed works everywhere except spawn — the agent CLIs are not the harness's to ship.

## Considered Options

- **Replace Go with Rust + Tauri** — the best shipping story available (installers, updater, signing, tiny artifacts) and the model-layer reuse turned out cheap to forfeit (~800 lines). Rejected because Tauri inverts browser-first into a local window, and it moves the PTY fan-out core — the codebase's gnarliest concurrency — from goroutines to async Rust while adopting system-webview variance as our bug surface.
- **Webview shell as a supported equal** (the wayfinder-maps posture) — doubles the release matrix and makes the cgo toolchains release-blocking; the asymmetry is real and should be a tier boundary, not a support promise.
- **Browser app-mode launch instead of any shell** — zero-cost native feel (~85%), but the operator wants a real shell available; app-mode remains possible without being an artifact.
- **A doctor command** — the same facts as the badges with more ceremony, away from the moment of need.

## Consequences

- The release pipeline must treat shell build failures as warnings, not errors.
- "Supported" claims in the README follow the tiers exactly; native Windows is labelled best-effort explicitly rather than implied.
- Channels beyond GitHub releases (Homebrew especially) stay cheap to add later once tagged releases exist; declining them now forecloses nothing.
