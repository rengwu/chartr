//go:build webview && !darwin

package main

import "runtime"

// missingRuntime names what a failed window creation means on this platform.
// The whole point of naming it is that a missing dependency is never papered
// over with a silent browser launch (spec story 58).
var missingRuntime = map[string]string{
	"linux":   "no WebKitGTK or no display — check libwebkit2gtk and $DISPLAY/$WAYLAND_DISPLAY",
	"windows": "the Microsoft Edge WebView2 runtime is missing",
}[runtime.GOOS]

// setAppName is a no-op off macOS: GTK and Win32 windows take their name from
// the window title, which the shell sets directly.
func setAppName(string) {}

// installNativeMenu is a no-op off macOS. The native menu is the mac answer to a
// bare window losing the browser's menu bar; GTK and Win32 windows keep their
// own window controls, and inventing a menu bar for them is not this ticket's
// work (ADR 0013).
func installNativeMenu(string) {}

// raiseInstance always reports false here: raising another process's window is
// exactly the "flaky" case the spec names, so these platforms take the
// refuse-with-message path (spec story 54) rather than pretend.
func raiseInstance(int) bool { return false }
