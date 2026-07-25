//go:build webview && !darwin

package main

// setAppIcon is a no-op off macOS. GTK and Win32 take a window icon rather than
// an application one, and webview_go exposes no handle to set it — the same
// reason installNativeMenu stops at the mac (ADR 0013). The taskbar falls back
// to the platform default.
func setAppIcon([]byte) {}
