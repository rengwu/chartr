// Package web embeds the built Svelte SPA (ADR 0010) so distribution stays one
// self-contained binary. The Vite build writes to web/dist; go:embed folds that
// directory into the binary. A fresh checkout has only dist/.gitkeep — enough
// to compile — and `make web` produces the real index.html and assets. During
// development the SPA is served by Vite's dev server instead (see web/README).
package web

import (
	"embed"
	"io/fs"
)

//go:embed all:dist
var embedded embed.FS

// Dist returns the built SPA rooted at its top level (index.html at "/"). It
// errors only if the dist directory is missing from the binary, which cannot
// happen once this package compiles.
func Dist() (fs.FS, error) {
	return fs.Sub(embedded, "dist")
}
