---
type: task
blocked_by: [14, 15, 17]
---

# Release: one supported artifact

## Question

Shipping per ADR 0011. goreleaser builds and checksums the one supported artifact — the pure-Go browser-serving binary with the embedded frontend — for macOS, Linux, and Windows from a single cgo-free CI job; the per-platform webview shells build best-effort and attach as extra assets where they succeed, never blocking a release. Windows CI smoke-tests the ConPTY-backed PTY layer (ADR 0006 as amended). Distribution is GitHub releases only — no `go install`, no Homebrew, no plugin marketplace. The README states the tiers exactly: what supported means, Windows native as best-effort by decision, WSL2 as the documented sure path, and the honest cold start — with zero agent CLIs installed everything works but spawn, whose block message doubles as the installer's to-do list.

Done when: a tagged release produces checksummed supported binaries for all three OSes from the cgo-free job, with shell assets attached where their toolchains allowed and a shell failure demonstrably not failing the release; the Windows smoke test runs a PTY round-trip in CI; the README's support claims match ADR 0011's tiers word for word in substance.
