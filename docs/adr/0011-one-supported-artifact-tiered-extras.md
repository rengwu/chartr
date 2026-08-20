# One supported artifact; everything else is best-effort

**Amended again by [0019](0019-native-deb-and-rpm-packages.md): native deb and
rpm Linux desktop packages now join the AppImage on the supported tier.** Both
architectures and both formats build, install and render before a tag may
publish. **Amended again for v0.2.4: the Apple Silicon disk image is also
required and verified before publication.** The best-effort tier now means the
Windows loose shell; the pure-Go browser binary remains the portable baseline.

chartr ships as **one supported artifact**: the pure-Go binary that serves the browser frontend from its embedded Vite `dist/` (ADR 0010) — cross-compiled for macOS, Linux and Windows from a single CI job, because nothing in it requires cgo. Everything beyond it is a **best-effort tier** that may fail to build without blocking a release: the native webview shell per platform (cgo on macOS, cgo-free `go-webview2` on Windows), and native Windows itself (built and smoke-tested in CI, not driven daily; WSL2 is the documented sure path).

**Amended: the Linux AppImage is a second supported artifact, and it gates the release.** The best-effort posture was the right default and it failed on exactly the case it was designed to absorb — quietly. `webview_go` pins `pkg-config: webkit2gtk-4.0`, that package left every current distro, and because the Linux shell was only ever built inside a `continue-on-error` job, `v0.1.0` shipped with no Linux app at all and no signal that anything was missing. A tier that may fail silently is indistinguishable from a tier that does not exist. So Linux's desktop app is now built, **smoke-tested against a container with no WebKit and no GTK installed**, and a failure fails the tag. What makes that gate honest is that it looks at the screen: a bundled WebKit that cannot spawn its helper processes still starts, still binds its port, still serves the SPA over loopback and still exits 0, rendering an error page where the cockpit should be — so the assertion is a screenshot's brightness, not an exit code. macOS and Windows shells keep the best-effort tier unchanged.

Distribution is **GitHub releases only**, goreleaser-built and checksummed, with best-effort shells attached as extra assets where they built. Declined: `go install`, Homebrew, and a Claude Code plugin marketplace entry — the last deliberately, an agent-agnostic tool not distributed through one agent's storefront.

There is **no doctor command**. The environment diagnosis is the registry badge and the spawn-time hard-block message (ticket 05), surfaced at the moment of need. A cold start with zero agent CLIs installed works everywhere except spawn — the agent CLIs are not chartr's to ship.

## Considered Options

- **Replace Go with Rust + Tauri** — the best shipping story available (installers, updater, signing, tiny artifacts) and the model-layer reuse turned out cheap to forfeit (~800 lines). Rejected because Tauri inverts browser-first into a local window, and it moves the PTY fan-out core — the codebase's gnarliest concurrency — from goroutines to async Rust while adopting system-webview variance as our bug surface.
- **Webview shell as a supported equal** (the wayfinder-maps posture) — doubles the release matrix and makes the cgo toolchains release-blocking; the asymmetry is real and should be a tier boundary, not a support promise.
- **Browser app-mode launch instead of any shell** — zero-cost native feel (~85%), but the operator wants a real shell available; app-mode remains possible without being an artifact.
- **A doctor command** — the same facts as the badges with more ceremony, away from the moment of need.

## Consequences

- The release pipeline gates on the Linux desktop packages and Apple Silicon disk image. The Windows shell remains best-effort.
- Bundling WebKitGTK costs a ~78 MB artifact and one binary edit: WebKitGTK 2.52 removed the `WEBKIT_EXEC_PATH` override, so the helper-process directory compiled into the library is rewritten at package time (`scripts/relocate-webkit.py`) and pointed at the AppDir by `packaging/linux/AppRun`. That is a moving part which a WebKit upgrade can break, which is why the rewrite fails loudly when the expected path is absent rather than shipping a broken renderer.
- The AppImage borrows libEGL, libGLESv2, libfontconfig and libwayland-client from the host — they are bound to the GPU driver, the operator's fonts and their compositor, so bundling them would substitute our build machine's idea of their hardware.
- "Supported" claims in the README follow the tiers exactly; native Windows is labelled best-effort explicitly rather than implied.
- Channels beyond GitHub releases (Homebrew especially) stay cheap to add later once tagged releases exist; declining them now forecloses nothing.
