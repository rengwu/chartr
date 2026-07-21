---
type: task
blocked_by: [14, 15, 17]
---

# Release: one supported artifact

## Question

Shipping per ADR 0011. goreleaser builds and checksums the one supported artifact — the pure-Go browser-serving binary with the embedded frontend — for macOS, Linux, and Windows from a single cgo-free CI job; the per-platform webview shells build best-effort and attach as extra assets where they succeed, never blocking a release. Windows CI smoke-tests the ConPTY-backed PTY layer (ADR 0006 as amended). Distribution is GitHub releases only — no `go install`, no Homebrew, no plugin marketplace. The README states the tiers exactly: what supported means, Windows native as best-effort by decision, WSL2 as the documented sure path, and the honest cold start — with zero agent CLIs installed everything works but spawn, whose block message doubles as the installer's to-do list.

Done when: a tagged release produces checksummed supported binaries for all three OSes from the cgo-free job, with shell assets attached where their toolchains allowed and a shell failure demonstrably not failing the release; the Windows smoke test runs a PTY round-trip in CI; the README's support claims match ADR 0011's tiers word for word in substance.

## Answer

The whole ticket is release *infrastructure and documentation* per ADR 0011 —
no product surface changes. Four new files carry it, plus a small version-stamp
seam in `main.go` and three Makefile targets. The tier boundary is enforced by
*where* each artifact is built, not by any runtime flag: the supported binary
lives inside goreleaser, and the best-effort shells live entirely outside it.

**The supported artifact — `.goreleaser.yaml`.** One `builds` entry,
`./cmd/harness`, `CGO_ENABLED=0`, `goos: [darwin, linux, windows] × goarch:
[amd64, arm64]` — all six targets off one cgo-free runner because nothing in the
binary needs cgo (verified directly: `CGO_ENABLED=0 GOOS=… GOARCH=… go build`
succeeds for every pair). Archives are `.tar.gz` everywhere and `.zip` on
Windows, each bundling the `harness` binary plus the README and ADR 0011; a
`checksums.txt` (SHA-256) covers them. Version/commit/date are stamped through
`-ldflags -X main.version=…`, for which `main.go` grew a `version/commit/date`
trio (defaulting to `dev`/`none`/`unknown` so a from-source build honestly
reports itself) and a `-version` flag. goreleaser's `dist:` is pointed at
`build/goreleaser` so it never collides with `web/dist`, the embed directory.
**The native webview shells are deliberately *not* in this file** — keeping them
out of goreleaser is exactly what guarantees a shell build can never fail the
supported release.

**The release pipeline — `.github/workflows/release.yml`** (on `v*` tags), three
jobs in a deliberate order:

- `windows-smoke` runs first and **gates** the release: the supported artifact
  ships for Windows, so its ConPTY path must round-trip before a tag is cut.
- `release` (`needs: windows-smoke`, cgo-free Ubuntu) runs `make web` then
  goreleaser → the checksummed supported binaries for all three OSes. This job
  succeeding *is* the release.
- `shells` (`needs: release`, `continue-on-error: true`, matrix over
  ubuntu/macos/windows) is the best-effort tier. Because it needs `release`, the
  supported binaries are already published and checksummed before a single shell
  is attempted; because it is `continue-on-error`, any webview toolchain failure
  on any platform attaches nothing for that platform and leaves the release
  untouched; where a shell builds it is `gh release upload`ed as an extra asset.
  **This is the structural guarantee that "a shell failure does not fail the
  release."**

**Windows CI smoke test — `internal/terminal/pty_windows_test.go`**
(`//go:build windows`). The Unix `procstat_test.go` is `!windows` and probes the
foreground-group refinement ConPTY doesn't have; this is its sibling for the one
thing ConPTY *does* have to prove — a real PTY round-trip. It drives the public
`Manager` surface an operator's shell tab uses (`Open` → `Attach` → `Write`),
types `echo <token>` into `cmd.exe`, and reads down-frames until the token
echoes back, so a regression in the go-pty ConPTY binding, the pump loop, or the
broadcast fan-out all surface here. It runs on `windows-latest` in **both** the
release gate and `ci.yml` (every push/PR), satisfying "Windows CI smoke-tests
the ConPTY-backed PTY layer."

**Continuous checks — `.github/workflows/ci.yml`.** A cgo-free Ubuntu job
(`make web` → vitest → `make check` → `make test`, plus a built-CSS amber guard
keyed to ADR 0012's monochrome-chrome rule) and the `windows-smoke` job, so the
ConPTY path is exercised on ordinary changes, not only at release.

**`README.md`** states the tiers to match ADR 0011 in substance: the supported
artifact is the pure-Go, cgo-free, checksummed browser-serving binary for all
three OSes; native webview shells are best-effort, attached only where their
toolchains built; **native Windows is best-effort by decision** with **WSL2 the
documented sure path**; distribution is **GitHub releases only** (no `go
install`, no Homebrew, no plugin marketplace, the last declined as an
agent-agnostic tool); and the **honest cold start** — with zero agent CLIs
installed everything works except spawn, whose hard-block message names what's
missing and **doubles as the installer's to-do list** (there is no doctor
command).

**Makefile** grew `snapshot`/`release` (goreleaser wrappers) and `webview` — the
best-effort shell target the release job calls, a no-op that exits 0 with an
explanatory line until `cmd/webview` exists, so the shell lane stays green and
simply attaches nothing.

**Verified.** `go vet ./...` and `go test ./...` clean; the new Windows test
compiles for its target (`GOOS=windows go test -c ./internal/terminal`) and the
Unix build is unaffected. The supported-binary path is proven end-to-end
locally, not just asserted: `goreleaser check` validates the config, and a full
`goreleaser release --snapshot` produced all six archives (`.tar.gz` ×4, `.zip`
×2) with a `checksums.txt` and the Windows `.zip` containing `harness.exe` (8.7
MB — the SPA is embedded) alongside the README and ADR. All three YAML files
parse.

**Flagged, honestly:**

- **No webview shell source ships in this ticket.** ADR 0011 frames the native
  webview shells as a *tier*, not a deliverable, and shipping a real cgo
  WebKitGTK / `go-webview2` binding I could not build or verify in this
  environment (no webview toolchains here; the darwin box can't validate the
  Windows/Linux cgo paths) would have been unverified code in a repo that
  verifies before it commits. So the shell **lane** is real, non-blocking, and
  ready — the `shells` job, the `make webview` seam, the upload wiring — but it
  attaches nothing today. "Shell assets attached where their toolchains allowed"
  is therefore satisfied as a *pipeline capability* (and its non-blocking
  property structurally guaranteed), not yet demonstrated with a live attached
  asset. The webview shell source is the natural next artifact and slots into
  this lane unchanged.
- **The workflows themselves were not executed.** GitHub Actions can't run in
  this session, so `ci.yml`/`release.yml` are verified by structure, by YAML
  validity, and by proving each step's local equivalent (the full snapshot
  build for the release job; the `GOOS=windows` test-compile for the smoke job;
  `make check`/`make test`/vitest for the check job) — not by a live CI run.
  This is the same environmental deferral tickets 12/14/17 hit for their
  by-eye/browser passes: worth a real tag-and-watch the first time a release is
  cut. The `secrets.GITHUB_TOKEN` the release and upload steps use is the
  workflow-provided default (contents:write), so no repo secret setup is needed.
