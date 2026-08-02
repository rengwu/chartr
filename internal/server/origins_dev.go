//go:build chartrdev

package server

import "os"

// devOriginPatterns names the Vite dev server as an allowed origin, so the
// development loop — `make dev-web` serving the SPA with HMR and proxying /api
// and /ws to `make dev-backend` — keeps working with the cross-origin check on.
//
// The proxy needs naming because Vite forwards with `changeOrigin: true`
// (web/vite.config.ts): the request arrives carrying the backend's Host and the
// browser's own Origin, so the two genuinely differ and the library's
// Origin-equals-Host rule cannot admit it. Naming this one origin is the whole of
// what the old `InsecureSkipVerify: true` was buying.
//
// It is compiled only under the `chartrdev` tag — see origins_shipped.go for why
// the gate is a tag and not a flag. CHARTR_DEV_ORIGIN overrides the default pair
// for a Vite that landed on another port (5173 taken) or another host, mirroring
// CHARTR_BACKEND in the other direction; it is read only in this build, and
// setting it against a released binary does nothing.
func devOriginPatterns() []string {
	if o := os.Getenv("CHARTR_DEV_ORIGIN"); o != "" {
		return []string{o}
	}
	return []string{"http://localhost:5173", "http://127.0.0.1:5173"}
}
