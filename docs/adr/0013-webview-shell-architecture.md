# A separate cgo `cmd/webview` shell around the in-process server

The cockpit is launchable as a real OS window by a **second binary**, `cmd/webview`, built `CGO_ENABLED=1 -tags webview`. The supported `harness` binary (ADR 0011) stays pure Go and cgo-free; the shell is the best-effort tier and never touches the supported release lane.

The shell is **not a second application**. It imports `internal/server`, does exactly what `cmd/harness`'s `run()` does — `server.New`, listen, `Serve` — with two differences: it binds `127.0.0.1:0` and reads the OS-assigned port off `ln.Addr()`, so there is no fixed port to collide on; and it points a `webview/webview` (zserge) window at `http://<that>`. One process, one server, one window. Closing the window cancels the same context `signal.NotifyContext` cancels today, so `⌘Q`, window-close and `SIGTERM` all converge on one teardown. Rejected: a `harness --webview` flag (cgo in the supported binary), spawning the supported binary as a child (two processes to reason about), and Wails (a second IPC layer for a UI that already talks HTTP and websockets).

The package is **split by build tag** so the cgo is invisible to every default build. `main_webview.go` (`//go:build webview`) is the real shell; `main_stub.go` (`//go:build !webview`) is a `main` that prints "built without the webview tag — use `make webview`" and exits non-zero. `go build ./...`, `go vet ./...` and `go test ./...` at `CGO_ENABLED=0` still compile this package — they compile the harmless half — and goreleaser, which builds `./cmd/harness` explicitly, never sees the cgo at all.

**Single-instance is a lock file, not a window handle.** `<data-dir>/.wayfinder-harness/shell.lock` records the live instance's pid and loopback URL; the claim is an `O_EXCL` create. Keying it to the data dir is what makes distinct `--data-dir` roots distinct instances *by construction* — there is no global lock to contend for. A second launch raises the running window and exits 0; where raising is not possible it refuses with the running URL rather than pretend (the operator can still open that URL in a browser). **The mechanism is a pid, not `webview.Window()`**: the planning map named the native handle, but a second launch is a *different process*, and a window handle is meaningless across the process boundary. macOS raises via `NSRunningApplication`; Linux and Windows take the refuse-with-message path. A lock whose pid is dead is **stale and taken over** — `⌘Q` routes through AppKit's `terminate:` and runs no deferred cleanup, so a lock outliving its process is the normal case, not the exceptional one, and it must never lock the operator out of their own cockpit.

**Native integration is a window, a dock icon, and a minimal menu** — Quit (⌘Q), Reload (⌘R), and the standard edit items, every one of them a responder-chain selector so the menu needs no callback into Go: `NSApplication` answers the app items and `WKWebView` answers `reload:` and the edit items itself. The menu is macOS-only; GTK and Win32 windows keep their own controls. Because a bare binary is not a `.app` bundle, the shell seeds `CFBundleName` before `NSApplication` exists, so the menu bar reads `wayfinder-harness` rather than the executable's name. **Declined**: a dock badge for the "Needs you" queue, and a `harness://` URL scheme — each returns only on a concrete trigger.

**A missing native runtime is a hard error** that names what is missing and points at `harness`. `webview_create` returns NULL on failure and the Go wrapper hands that NULL back inside a non-nil interface, so the shell reads the wrapper's handle field directly to detect it — the one unsafe corner, confined to one function. There is **no auto browser fallback and no `--browser`/`--shell` force flag**: the two binaries *are* the choice, and neither impersonates the other.

**Release** keeps the tiers apart structurally. The shell rides the same tag with the same version/commit stamp, is built natively per runner (cgo does not cross-compile) by the `continue-on-error`, `needs: release`, `fail-fast: false` `shells` matrix, and ships a **per-asset `.sha256` sidecar**. The supported `checksums.txt` is never mutated by a best-effort artifact.

## Consequences

- The shell is a second `main` that must track `cmd/harness`'s startup. It is ~40 lines of overlap; a third consumer would earn a shared `run` helper, two do not.
- `webview/webview` enters `go.mod` as a direct dependency even though the default build never links it. `go mod tidy` keeps it because the tagged build needs it.
- The shell's window behaviour and menu are **not unit-tested** — they need a real display. The lock is, in full; the tagged build is the CI matrix's job.
- Linux's shell depends on the distro shipping headers `webview_go` pkg-configs for (`webkit2gtk-4.0`). Where it does not, that platform simply attaches no asset — which is what the best-effort tier is for.

## Considered options

- **`harness --webview`** — one binary, one download; rejected because it puts cgo and a system webview dependency inside the supported artifact, which ADR 0011 exists to prevent.
- **Shell spawns the supported binary as a child** — keeps the server code untouched, but buys two processes, a port handshake, and orphan cleanup for no gain over an in-process import.
- **Wails / Tauri-style framework** — a full app framework with its own IPC, bindings and build pipeline, for a UI that is already a website talking HTTP and websockets over loopback.
- **Auto-fallback to the browser when the window fails** — friendly, and exactly the papering-over that makes a missing dependency invisible. The error names it instead.
- **A single-instance lock keyed globally (or by port)** — would make two data dirs fight over one window; keying to the data dir makes the invariant fall out of the path.
