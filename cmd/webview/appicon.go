//go:build webview

package main

import (
	"io/fs"

	"github.com/rengwu/chartr/web"
)

// appIconPath is the icon's name inside the embedded SPA. Vite copies web/public
// to the dist root, so the file the browser fetches as the PWA icon and the file
// the Dock shows are the same bytes — the shell carries no second copy of the
// artwork, and the two can never drift apart.
const appIconPath = "icon-512.png"

// applyAppIcon dresses the running app in the chartr mark, where the platform
// has somewhere to put it (the macOS Dock; nowhere else, today).
//
// Every failure here is silent and survivable, because none of them is worth
// refusing to start over: a fresh checkout embeds only dist/.gitkeep, so before
// `make web` there is no icon in the binary at all. The shell simply keeps the
// platform's default, exactly as it did before it had a mark.
func applyAppIcon() {
	dist, err := web.Dist()
	if err != nil {
		return
	}
	png, err := fs.ReadFile(dist, appIconPath)
	if err != nil || len(png) == 0 {
		return
	}
	setAppIcon(png)
}
