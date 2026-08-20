//go:build webview && !darwin && !linux

package main

import webview "github.com/webview/webview_go"

// appIconPath is read (and discarded) even here: applyAppIcon loads the bytes
// before deciding what to do with them, so every platform needs a value.
const appIconPath = "icon-512.png"

// setAppIcon is a no-op on Windows: Win32 takes a window icon from the running
// executable's own resources rather than a runtime call, and webview_go exposes
// no handle to set one dynamically — the same reason installNativeMenu stops at
// the mac (ADR 0013). The taskbar falls back to the platform default.
func setAppIcon(webview.WebView, []byte) {}
