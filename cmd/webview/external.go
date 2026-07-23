package main

import (
	"fmt"
	"net/url"
	"os/exec"
	"runtime"
)

// Opening a link in the operator's real browser.
//
// The shell's window is the whole application, so a URL clicked in terminal
// output must not navigate it — the operator would lose the cockpit with no way
// back. `main_webview.go` binds openExternalURL into the page as
// `window.__chartrOpenExternal`, mirroring the `__chartrTitleBar` contract: the
// shell injects a global, the page treats its presence as the capability, and a
// plain browser tab that never sees it opens a new tab instead (web/src/lib/
// external.ts). This file is untagged so the URL guard compiles and is tested by
// the default cgo-free build, exactly as the lock is (ADR 0013).

// openExternalURL hands a URL to the OS to open in the operator's default
// browser. It never blocks on the child — the browser is the operator's to close.
//
// It is bound to the page, so its argument is whatever the cockpit passed, which
// ultimately traces back to text an agent printed into a terminal. Untrusted
// input reaching `open` is the risk worth naming: `open` and `xdg-open` will
// launch an application for a `file:` path or a registered custom scheme, so the
// guard below is not a formality.
func openExternalURL(raw string) error {
	if err := checkExternalURL(raw); err != nil {
		return err
	}
	opener := osOpener()
	if opener == "" {
		return fmt.Errorf("no way to open a URL on %s", runtime.GOOS)
	}
	cmd := exec.Command(opener, raw)
	if err := cmd.Start(); err != nil {
		return err
	}
	// Reap it in the background so a browser launched from here is not left a
	// zombie child of the shell.
	go func() { _ = cmd.Wait() }()
	return nil
}

// checkExternalURL accepts only an absolute http(s) URL. Everything else — a
// `file:` path, a custom app scheme, a bare string, a scheme-relative reference —
// is refused rather than passed to the OS opener. The client keeps the same
// allowlist; both sides hold it because either alone would be the only thing
// standing between terminal output and a launched application.
func checkExternalURL(raw string) error {
	u, err := url.Parse(raw)
	if err != nil {
		return fmt.Errorf("not a URL: %w", err)
	}
	if u.Scheme != "http" && u.Scheme != "https" {
		return fmt.Errorf("refusing to open %q: only http and https URLs are opened", raw)
	}
	if u.Host == "" {
		return fmt.Errorf("refusing to open %q: no host", raw)
	}
	return nil
}

// osOpener is the platform's "open this in whatever handles it" command — the
// same ladder the config surface uses to open a file in an editor
// (internal/server/configsurface.go). Duplicated rather than shared: it is three
// lines, and the shell must not import the server's unexported internals.
func osOpener() string {
	switch runtime.GOOS {
	case "darwin":
		return "open"
	case "windows":
		return "explorer"
	default:
		return "xdg-open"
	}
}
