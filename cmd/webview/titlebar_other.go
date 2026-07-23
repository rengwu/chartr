//go:build webview && !darwin

package main

import webview "github.com/webview/webview_go"

// installTitleBar is macOS-only. Everywhere else the window keeps the title bar
// its window manager draws — Windows and the Linux desktops each have their own
// conventions for the buttons' side, order and behaviour, and a bar that guessed
// wrong would be worse than the native one. Reporting zero is how the cockpit
// learns to render nothing.
func installTitleBar(webview.WebView) int { return 0 }
