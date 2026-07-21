---
type: grilling
---

# The webview shell

## Question

The settled direction: a native window around the existing Svelte cockpit, mac-first, Linux second. ADR 0011 pre-decided the *tiering* (the browser-serving binary stays the one supported artifact; shells are best-effort per platform) and a dead `make webview` target already points at a `cmd/webview` that was never written. This ticket settles what the shell actually is.

The pressure points: the shell's value is a dock icon, a real window, native menus, and not living in a browser tab — but every gram of framework beyond that is cost. Wails gives a full desktop-app framework (bindings, events, menus, its own build system) for a product whose entire UI already talks to its Go backend over HTTP and websockets; `webview/webview` (zserge) gives a window and nothing else, with cgo on macOS (WKWebView) and Linux (WebKitGTK). The cgo arrival also breaks the current single cgo-free CI job — ADR 0011 anticipated this with the split-lane release, but the concrete pipeline shape is undecided.

Settle:

- **Process model.** One binary with a `--webview` flag, a subcommand (`harness shell`?), or a separate `cmd/webview` binary that embeds or launches the server? Who owns the port and the server lifecycle — does the shell spawn the server in-process on a random loopback port, and what happens on a second launch (focus the existing window, open another)?
- **The library.** `webview/webview` versus Wails versus anything newer, judged on: WKWebView quality on macOS, WebKitGTK pain on Linux, menu/dock affordances the cockpit actually needs, and build complexity. The honest default is the thinnest thing that puts the cockpit in a window — argue against it if it can't hold up.
- **How native it acts.** Which platform integrations are in the shell's job description at all: app menu, dock badge (the "Needs you" queue's natural home), keyboard shortcuts, single-instance behaviour, opening `harness://` links? Each one named is scope; each one declined is a sentence in the answer.
- **The release pipeline.** What the goreleaser/CI split actually looks like with a cgo lane (macOS and Linux runners building shells, the cgo-free job untouched), what "shell build failure never fails a release" means mechanically, and whether the shell and the binary share a version and a checksums file.
- **The fallback contract.** When WebKitGTK is missing or the shell fails to start, what does the operator get — an automatic browser fall-back, a hard error naming the missing piece (the spawn-block pattern), or a flag to force one or the other?
