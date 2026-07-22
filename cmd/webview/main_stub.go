//go:build !webview

package main

import (
	"fmt"
	"os"
)

// main is what the default, cgo-free build of this package compiles: a refusal.
//
// The real shell needs cgo and a system webview library, which the supported
// release lane deliberately does not have (ADR 0011). Keeping the cgo behind
// //go:build webview and leaving this stub in its place is what lets
// `go build ./...`, `go vet ./...` and `go test ./...` stay green at
// CGO_ENABLED=0 — the wildcard still compiles this package, it just compiles
// the harmless half.
func main() {
	fmt.Fprintln(os.Stderr,
		"chartr shell: built without the webview tag — use `make webview`.\n"+
			"The native shell needs cgo and a system webview library; the supported,\n"+
			"cgo-free cockpit binary is `chartr`.")
	os.Exit(1)
}
