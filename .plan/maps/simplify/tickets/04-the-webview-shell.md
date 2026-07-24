---
type: grilling
---

# The webview shell

## Question

The settled direction: a native window around the existing Svelte cockpit, mac-first, Linux second. ADR 0011 pre-decided the *tiering* (the browser-serving binary stays the one supported artifact; shells are best-effort per platform) and a dead `make webview` target already points at a `cmd/webview` that was never written. This ticket settles what the shell actually is.

The pressure points: the shell's value is a dock icon, a real window, native menus, and not living in a browser tab — but every gram of framework beyond that is cost. Wails gives a full desktop-app framework (bindings, events, menus, its own build system) for a product whose entire UI already talks to its Go backend over HTTP and websockets; `webview/webview` (zserge) gives a window and nothing else, with cgo on macOS (WKWebView) and Linux (WebKitGTK). The cgo arrival also breaks the current single cgo-free CI job — ADR 0011 anticipated this with the split-lane release, but the concrete pipeline shape is undecided.

Settle:

- **Process model.** One binary with a `--webview` flag, a subcommand (`chartr shell`?), or a separate `cmd/webview` binary that embeds or launches the server? Who owns the port and the server lifecycle — does the shell spawn the server in-process on a random loopback port, and what happens on a second launch (focus the existing window, open another)?
- **The library.** `webview/webview` versus Wails versus anything newer, judged on: WKWebView quality on macOS, WebKitGTK pain on Linux, menu/dock affordances the cockpit actually needs, and build complexity. The honest default is the thinnest thing that puts the cockpit in a window — argue against it if it can't hold up.
- **How native it acts.** Which platform integrations are in the shell's job description at all: app menu, dock badge (the "Needs you" queue's natural home), keyboard shortcuts, single-instance behaviour, opening `chartr://` links? Each one named is scope; each one declined is a sentence in the answer.
- **The release pipeline.** What the goreleaser/CI split actually looks like with a cgo lane (macOS and Linux runners building shells, the cgo-free job untouched), what "shell build failure never fails a release" means mechanically, and whether the shell and the binary share a version and a checksums file.
- **The fallback contract.** When WebKitGTK is missing or the shell fails to start, what does the operator get — an automatic browser fall-back, a hard error naming the missing piece (the spawn-block pattern), or a flag to force one or the other?

## Answer

**The shell is a separate `cmd/webview` binary that starts the existing server in-process on a random loopback port and points a `webview/webview` (zserge) window at it — a real window with a minimal native menu and single-instance focus, and nothing else.** It shares the supported binary's version and commit stamp but never its cgo-free promise: it is the best-effort tier ADR 0011 already carved out, and every piece of its release plumbing already exists in the tree (`make webview`, the `shells` CI matrix, the goreleaser split). This ticket does not amend ADR 0011 — every tiering premise holds — it records the shell's own architecture as a **new ADR 0013**.

### The process model

**A separate binary, not a flag.** The supported artifact stays pure-Go and cgo-free (ADR 0011); the moment cgo enters `cmd/chartr` that promise is gone. So the shell cannot be `chartr --webview` — it is `cmd/webview`, exactly where the dead `make webview` target already points, built with `CGO_ENABLED=1 -tags webview`.

- **The server runs in-process on a random loopback port.** `cmd/webview` imports `internal/server` and does what `cmd/chartr`'s `run()` already does — `server.New` then `net.Listen` — but binds `127.0.0.1:0`, reads the OS-assigned port off `ln.Addr()`, and hands `http://<that>` to the webview. One process, no fixed port to collide on, and the server dies with the window: closing the webview cancels the same context `signal.NotifyContext` cancels today. Rejected: spawning the supported binary as a child (two processes, port-collision handling, and process supervision the in-process path never needs — the model layer is import-cheap, which is the reuse ADR 0001 banked).
- **Second launch focuses the existing window** rather than opening a duplicate — the one-window invariant is the whole point of a shell. Mechanism: a lock file in the data-dir (`.chartr/shell.lock`) recording the live instance's loopback URL. A second launch that finds a held lock sends a focus request to that URL's control surface; the running instance raises its window through the native handle `webview.Window()` exposes (cgo, already paid). Where raising proves per-platform flaky, it degrades to **refuse-with-message** — "chartr shell already running at `<url>`", exit non-zero — which still honours one-window without pretending the raise worked. The lock is keyed to the data-dir, so distinct `--data-dir` roots are distinct instances by construction.

### The library

**`webview/webview` (zserge), the thinnest thing that holds up.** ADR 0011 already named its exact backends — WKWebView on macOS, WebKitGTK on Linux, cgo-free `go-webview2` on Windows — which *is* zserge's backend set; the ticket's "honest default" and the ADR's Tauri rejection both point here. Wails is rejected on its own terms: a full desktop-app framework (bindings, event bus, its own build system, menu DSL) for a UI that already talks to its Go backend over HTTP and websockets is paying for a second IPC layer we will never call. The one thing zserge does *not* give us is menus (see below) — a few dozen lines of per-platform native code, cheaper than adopting a framework to avoid writing them.

### How native it acts

Two integrations are in the shell's job description; two named in the question are declined.

**In scope:**

- **A real window and a dock/taskbar icon** — the floor; the reason the shell exists over a browser tab.
- **A minimal native menu.** zserge gives a window and an `NSApplication`/GTK app, not a menu bar, so the menu is our thin per-platform code: **Quit** (⌘Q), **Reload** (⌘R), and the standard **edit** items (cut/copy/paste/select-all) that a browser tab supplied for free and a bare webview window otherwise lacks. Nothing app-specific — every chartr command already lives in the chrome and stays there. The menu restores OS-level affordances, it does not become a second command surface.
- **Single-instance focus** — mechanism above.

**Declined, each a sentence:**

- **No dock badge for the "Needs you" queue.** The queue is already surfaced in the chrome; mirroring its count onto a dock badge is bespoke cgo per platform (zserge has no badge API) for a signal the operator sees the moment the window is focused. *Revisit* if the shell ships and the queue is genuinely missed while the window is unfocused — that is the concrete trigger, not an anticipated one.
- **No `chartr://` URL scheme.** Nothing in the product emits such a link, and registering a scheme is real OS-integration surface (`Info.plist` on macOS, a desktop entry + MIME association on Linux) provisioned for a consumer that does not exist — the speculative bloat this map cuts. It returns when a real producer of `chartr://` links does.

This is deliberately less than "feels native," exactly as the map's settled decision accepts: the shell is the app in a real window, no more, and the revisit trigger if daily-driving still feels wrong is a TUI companion, not more shell.

### The release pipeline

**Already built, and this ticket confirms rather than designs it.** The cgo-free single job never breaks, because the split the ticket worried about is already in the tree:

- **The supported lane is untouched.** `.goreleaser.yaml` builds only `./cmd/chartr`, `CGO_ENABLED=0`, six GOOS/GOARCH off one runner, and owns `checksums.txt`. The webview shell is deliberately *not* a goreleaser build — keeping it out is the structural guarantee a shell failure cannot fail the supported release.
- **The shell lane is `continue-on-error` and `needs: release`.** `release.yml`'s `shells` matrix (linux/darwin/windows) runs *after* the supported release is published and checksummed, installs each platform's webview toolchain, runs `make webview GOOS=…`, and uploads whatever built. `fail-fast: false` plus `continue-on-error: true` is what "a shell build failure never fails a release" means mechanically: the release already exists before the first shell is attempted, and every shell job may fail independently.
- **The cgo-free wildcard stays green** because the shell's cgo lives behind `//go:build webview`. `cmd/webview` ships two files: `main_webview.go` (`//go:build webview`, the real cgo shell) and `main_stub.go` (`//go:build !webview`, a tiny main that prints "built without the webview tag — use `make webview`" and exits non-zero). So `go vet ./...` / `go test ./...` / `go build ./...` at `CGO_ENABLED=0` compile only the harmless stub, the embed test is unaffected, and the real shell requires both the tag and cgo. goreleaser never sees it because it builds `./cmd/chartr` explicitly, not `./...`.
- **Shared version, separate checksum.** The shell links the same `-ldflags -X main.version/commit/date` as the supported binary, off the same tag — one version across both tiers. It is *not* in the supported `checksums.txt` (it is attached after goreleaser has already written and signed that file, and a best-effort asset must never mutate the supported manifest). Instead the `shells` job uploads a per-asset `<shell>.sha256` sidecar alongside each shell it attaches — cheap, honest, and it leaves the supported checksums exactly as goreleaser produced them.

### The fallback contract

**Hard error naming the missing piece — the spawn-block pattern, no silent magic.** When the webview library or its runtime is absent (WebKitGTK dev libs missing on Linux, WebView2 runtime absent on Windows) or the window fails to create, the shell exits non-zero with a message that names exactly what is missing and points at the supported browser binary: e.g. *"native shell unavailable: WebKitGTK (libwebkit2gtk-4.1) not found. The supported `chartr` binary serves the same cockpit in your browser."* This matches ADR 0011's stance — no doctor command, diagnosis surfaced at the moment of need — and it refuses to paper a missing native dependency over with an automatic browser launch the operator did not ask for and might not notice. Rejected: **auto browser fallback** (hides the degradation; an operator who ran the shell wanted the shell, and silently getting a browser tab instead is a worse surprise than a named error) and a **force flag** (`--browser`/`--shell` is surface for a choice the two binaries already express — run `chartr` for the browser, `webview` for the shell; a flag that makes each impersonate the other is the ambiguity the two-binary split exists to avoid).

### What this writes

- **New ADR 0013 — the webview shell architecture:** separate `cmd/webview` binary, in-process server on `127.0.0.1:0`, `webview/webview` (zserge), the build-tag seam, single-instance-via-lockfile, the two declined integrations, and the hard-error fallback.
- **ADR 0011 — confirmed, unamended.** Every tiering premise holds; this ticket is the concrete shell the ADR anticipated, not a change to it.
- The dead `make webview` target and the `shells`/goreleaser split stop being scaffolding and gain their source; no pipeline file needs reshaping, only `cmd/webview` needs writing.

### Revisit trigger

- **The shell still feels wrong to daily-drive** → the map's own trigger fires: a TUI companion, not another web layer or more native chrome.
- **Single-instance raise is unreliable on a platform** → that platform degrades to refuse-with-message (already the fallback), and raising is filed as a per-platform polish ticket, not a blocker.
- **A dock badge is genuinely missed**, or **a real `chartr://` producer appears** → each declined integration returns as its own ticket, earned by the concrete need rather than provisioned ahead of it.
