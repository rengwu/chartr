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
