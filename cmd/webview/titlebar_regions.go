//go:build webview

package main

// titleBarButtonRect is one live clickable rectangle in the page's top strip,
// measured in CSS pixels from the viewport's top-left corner. The page reports
// these whenever its header changes; the macOS drag overlay uses them as its
// exact passthrough regions.
type titleBarButtonRect struct {
	X      float64 `json:"x"`
	Y      float64 `json:"y"`
	Width  float64 `json:"width"`
	Height float64 `json:"height"`
}
