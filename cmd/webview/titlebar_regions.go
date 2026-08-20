//go:build webview

package main

// titleBarButtonRect is one live clickable rectangle in the page's top strip,
// measured in AppKit points from the viewport's top-left corner. The page scales
// its CSS-pixel DOM rectangle by the live WKWebView page zoom before reporting
// it; the macOS drag overlay uses the result as its exact passthrough region.
type titleBarButtonRect struct {
	X      float64 `json:"x"`
	Y      float64 `json:"y"`
	Width  float64 `json:"width"`
	Height float64 `json:"height"`
}
