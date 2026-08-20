//go:build webview

package main

import (
	"io/fs"

	webview "github.com/webview/webview_go"

	"github.com/rengwu/chartr/web"
)

// appIconPath names the icon inside the embedded SPA that setAppIcon should
// apply — the file differs per platform (see icon_darwin.go, icon_linux.go,
// icon_other.go), all of them files Vite copies from web/public to the dist
// root, so this is one the browser can fetch too.
//
// applyAppIcon dresses the running app in the chartr mark, where the platform
// has somewhere to put it (the macOS Dock; the Linux window manager's taskbar
// and alt-tab switcher; nowhere else, today).
//
// Every failure here is silent and survivable, because none of them is worth
// refusing to start over: a fresh checkout embeds only dist/.gitkeep, so before
// `make web` there is no icon in the binary at all. The shell simply keeps the
// platform's default, exactly as it did before it had a mark.
//
// It takes the webview itself, not just the bytes, because GTK has no
// process-wide icon call that reaches a window already created: the Linux
// implementation needs the specific GtkWindow handle w.Window() hands back.
// macOS and Windows ignore it — the Dock tile is set through NSApplication, and
// Windows has no runtime call at all (icon_darwin.go, icon_other.go).
func applyAppIcon(w webview.WebView) {
	dist, err := web.Dist()
	if err != nil {
		return
	}
	png, err := fs.ReadFile(dist, appIconPath)
	if err != nil || len(png) == 0 {
		return
	}
	setAppIcon(w, png)
}
