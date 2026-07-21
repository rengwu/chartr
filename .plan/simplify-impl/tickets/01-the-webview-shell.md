---
type: task
blocked_by: []
---

# The webview shell

## Question

Make the cockpit launchable as a real native window — the operator's oldest want,
banked first because it is collision-free (it reuses only `server.New` /
`Serve`, which the cut leaves intact).

Write `cmd/webview` where the dead `make webview` target already points, built
`CGO_ENABLED=1 -tags webview`. It ships two files behind a build tag:
`main_webview.go` (`//go:build webview`, the real cgo shell) and `main_stub.go`
(`//go:build !webview`, a tiny main that prints "built without the webview tag —
use `make webview`" and exits non-zero) — so the default cgo-free build compiles
only the harmless stub and goreleaser (building `./cmd/harness` explicitly) never
sees the cgo. The real shell imports `internal/server`, does what `cmd/harness`'s
`run()` does but binds `127.0.0.1:0`, reads the OS-assigned port off `ln.Addr()`,
and points a `webview/webview` (zserge) window at `http://<that>`; closing the
window cancels the same context `signal.NotifyContext` cancels today. Add a
minimal native menu (Quit ⌘Q, Reload ⌘R, the standard edit items) and
single-instance focus via a data-dir lock file (`.wayfinder-harness/shell.lock`
recording the live instance's loopback URL; a second launch raises the running
window through `webview.Window()`, degrading to a "shell already running at
`<url>`" refuse-with-message where raising is flaky; keyed to the data dir so
distinct `--data-dir` roots are distinct instances). When the native runtime is
absent or the window fails to create, exit non-zero with a message naming exactly
what is missing and pointing at the supported browser binary — no auto browser
fallback, no `--browser`/`--shell` force flag. The release plumbing already exists
(`make webview`, the `shells` matrix, the goreleaser split); confirm it and add
the per-asset `.sha256` sidecar rather than mutating the supported
`checksums.txt`. This writes **ADR 0013** and confirms **ADR 0011** unamended.

Done when: `go build ./...` / `go vet ./...` / `go test ./...` are green at
`CGO_ENABLED=0` (the stub compiles, the embed test is unaffected); `make webview`
builds a tagged binary that opens the cockpit in a real native window with a dock
icon and the native menu; a second launch focuses the existing window or refuses
with the running URL; a missing native runtime exits non-zero naming it and
pointing at `harness`; ADR 0013 is written and the single-instance lockfile logic
is unit-tested without a real window.

## Answer

Shipped as specified. `cmd/webview` is a second binary that imports
`internal/server`, runs `cmd/harness`'s `run()` against `127.0.0.1:0`, reads the
OS-assigned port off `ln.Addr()`, and points a `webview/webview` window at
`http://<that>`. Window-close, `⌘Q` and `SIGTERM` all converge on one teardown:
the signal path dispatches `Terminate` onto the native loop, `Run()` returns, and
the same context cancel shuts the server down. This writes **ADR 0013** and
confirms **ADR 0011** unamended.

**The tag split holds the cgo out of every default build.** `main_webview.go`
(`//go:build webview`) is the real shell; `main_stub.go` (`//go:build !webview`)
prints "built without the webview tag — use `make webview`" and exits 1. `lock.go`
carries no tag at all — it is pure Go, so the one piece of shell behaviour a test
can reach is reachable at `CGO_ENABLED=0`. Verified green there:
`go build ./...`, `go vet ./...`, `go test ./...` (the embed test untouched),
plus `go vet -tags webview` with cgo on.

**Single-instance is keyed to the data dir and identified by pid, not by
`webview.Window()`.** `<data-dir>/.wayfinder-harness/shell.lock` records pid +
loopback URL, claimed with `O_EXCL`. The ticket named the native window handle,
but a second launch is a *different process* and a window handle does not cross
that boundary — macOS raises through `NSRunningApplication`; Linux and Windows
take the refuse-with-message path the spec already provisioned for "where raising
is flaky". A **dead pid means a stale lock, taken over**: `⌘Q` goes through
AppKit's `terminate:` and runs no deferred cleanup, so a lock outliving its
process is the normal case, not the exception. The decision is unchanged; the
mechanism is corrected in ADR 0013 rather than silently.

**The native menu needs no Go callbacks.** Quit ⌘Q, Reload ⌘R and the edit items
are all responder-chain selectors with a nil target — `NSApplication` answers the
app items, `WKWebView` answers `reload:` and the edit items itself. Because a
bare binary is not a `.app` bundle, the shell seeds `CFBundleName` before
`NSApplication` exists, so the menu bar reads `wayfinder-harness` instead of the
executable name. macOS-only by decision; GTK/Win32 keep their own controls.

**Missing runtime is a hard error, and detecting it needed one unsafe corner.**
`webview_create` returns NULL on failure but the Go wrapper hands that NULL back
inside a *non-nil* interface, and calling `Window()` to ask is the crash we are
avoiding — so `nativeHandle` reads the wrapper's handle field by reflection. On
NULL the shell exits non-zero naming the platform's missing piece and pointing at
`harness`. No auto browser fallback, no force flag.

**Release plumbing confirmed and completed.** The `shells` matrix
(`needs: release`, `continue-on-error`, `fail-fast: false`) and the goreleaser
split were already correct. `make webview` now stamps the same
version/commit/date, names the asset
`wayfinder-harness-shell_<version>_<os>_<arch>`, writes a **per-asset `.sha256`
sidecar** (basename-relative, so `shasum -c` works next to the download), and
exits 0 without building when asked to cross-compile — cgo cannot, so the matrix
builds natively per runner.

**Driven for real, not just built.** On darwin: `make webview` → a titled window
with a dock icon serving the cockpit (HTTP 200 on the loopback port), menu bar
`wayfinder-harness / Edit / View` with the expected items under each; a second
launch raised the running window and exited 0; `SIGTERM` tore down the window,
the server, and the lock file. Seven lock tests cover write/read of the loopback
URL, refusal of a live holder (carrying its URL and pid), takeover of a stale and
of a corrupt lock, distinct data dirs as distinct instances, and a release that
declines to clobber a lock that is no longer ours.

**Flagged, honestly:**

- **Only the darwin shell was run.** The Linux and Windows shells are
  build-verified by the CI matrix, not by me — no toolchains here. Worse for
  Linux: `webview_go` pkg-configs `webkit2gtk-4.0`, which recent Ubuntu images no
  longer ship, so the workflow now tries 4.0 and falls back to 4.1 and the lane
  may still fail there. That is precisely the failure the best-effort tier
  absorbs — nothing attaches, the supported release is untouched — but the Linux
  shell should be assumed unbuilt until a real tag proves otherwise.
- **`CFBundleName` seeding mutates the dictionary AppKit returns.** It is the
  long-standing way a non-bundled app names its menu bar, and it is confined to
  one function, but it is a convention rather than a documented contract. If it
  ever stops working the failure is cosmetic: the menu bar reads the executable
  name.
- **The release workflow was not executed.** Same environmental deferral as
  ticket 16 on the previous map — the matrix is verified by structure and by
  running its local equivalent (`make webview`, sidecar, `shasum -c`), not by a
  live tag.
