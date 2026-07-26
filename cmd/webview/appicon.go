//go:build webview

package main

import (
	"io/fs"

	"github.com/rengwu/chartr/web"
)

// appIconPath is the icon's name inside the embedded SPA. Vite copies web/public
// to the dist root, so this is one of the files the browser can fetch — and it is
// the same bytes `make bundle` downscales into the bundle's .icns (ADR 0016), so
// the loose shell's Dock tile and the bundle's Finder icon can never drift apart.
//
// This is the mac-specific master rather than the square icon-512.png the PWA
// manifest points at, because setApplicationIconImage: does not mask either: what
// the PNG says, the Dock draws. Apple's shape has to be in the pixels, and it is
// only in this one.
const appIconPath = "icon-mac-1024.png"

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
