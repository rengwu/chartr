package server

import (
	"mime"
	"net"
	"net/http"
	"strings"
)

// gate wraps h with the checks that must hold for *every* route, whatever it
// is, applied before anything is routed. There are two today, and a third
// belongs here beside them rather than in another layer of wrapper:
//
//   - the request's Host must name the address chartr is actually listening on
//     (hostRule), which is what closes DNS rebinding;
//   - a write must declare a JSON body (mustDeclareJSON), so a cross-origin page
//     cannot cause one with a request the browser sends without a preflight.
//
// One place rather than per-handler is the point of both: a route added later
// inherits them, and there is no handler either can be forgotten on.
//
// A refusal is a plain http.Error with nothing worth reading in it, and never a
// redirect — a redirect to the right name would hand an attacker's page a
// working URL.
func gate(addr net.Addr, h http.Handler) http.Handler {
	rule := hostRuleFor(addr)
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if !rule.allows(r.Host) {
			http.Error(w, "forbidden", http.StatusForbidden)
			return
		}
		if mustDeclareJSON(r) && !declaresJSON(r) {
			http.Error(w, "expected Content-Type: application/json", http.StatusUnsupportedMediaType)
			return
		}
		h.ServeHTTP(w, r)
	})
}

// mustDeclareJSON reports whether this request has to say its body is JSON.
//
// Two kinds do, for two different reasons. **Every POST**, body or none: it is
// the one state-changing method a browser sends cross-origin with no preflight
// at all when its content type is `text/plain`, `multipart/form-data`,
// `application/x-www-form-urlencoded` — or absent, which is what a bare
// `fetch(url, {method: 'POST'})` sends. The routes that take no body are side
// effects worth causing blind (open a shell, raise the operator's folder
// chooser, write the tracker adapter into their repo), so "carries a body" is
// not the line. **And anything carrying a body**, whatever its method, because a
// body is bytes a handler will decode and an undeclared one has no business
// being decoded.
//
// What is left alone is left alone deliberately. A GET with no body changes
// nothing. A bodyless DELETE or PUT has nothing to decode, and neither is a
// method a browser can send cross-origin without a preflight this server never
// answers — the method split that already keeps `PUT /api/config/agents/{name}`
// out of reach is doing real work, and this check must not disturb it
// (websocket-origin-fix, ticket 03).
//
// This is the cheap second lock, not the remedy: once the origin patterns and
// the host gate are in place a foreign page cannot usefully reach these routes
// at all, and a rebinding attacker is same-origin and can set any content type
// they like. It holds if one of those two is ever misconfigured or regressed.
func mustDeclareJSON(r *http.Request) bool {
	if r.Method == http.MethodPost {
		return true
	}
	// ContentLength is 0 for an absent body and -1 when the length is unknown —
	// a chunked body, which counts.
	return r.ContentLength != 0
}

// declaresJSON reports whether the request declares a JSON body. The header is
// parsed rather than string-compared, so `application/json; charset=utf-8` — what
// a browser and most HTTP clients send — is the same declaration as a bare one,
// and the media type is matched case-insensitively. A missing or unparseable
// header declares nothing.
func declaresJSON(r *http.Request) bool {
	mediaType, _, err := mime.ParseMediaType(r.Header.Get("Content-Type"))
	if err != nil {
		return false
	}
	return strings.EqualFold(mediaType, "application/json")
}
