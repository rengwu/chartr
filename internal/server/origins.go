package server

import "net"

// originPatterns names the browser origins the two websocket handshakes admit,
// derived from the address chartr is actually listening on.
//
// coder/websocket refuses a cross-origin handshake by default, and that default
// is the whole guard here. Websockets are not subject to CORS, so no preflight
// stands between a page on an arbitrary origin and these sockets, and binding to
// loopback is not a boundary: 127.0.0.1 is reachable from every origin the
// operator happens to have open. The control socket hands the whole model
// snapshot — absolute paths, branch state, the live terminal ids — to whoever
// completes the handshake, and the terminal socket writes inbound frames straight
// to the operator's PTY, so the check is the only thing standing between another
// tab and their shells.
//
// The library already authorizes a handshake whose Origin equals the request's
// Host, which is every ordinary browse. These patterns are the *other* spellings
// of the same address — a browser at http://localhost:PORT reaching a server bound
// to 127.0.0.1:PORT — plus, in a development build only, the Vite dev server
// (devOriginPatterns, which is empty in every shipped build). Each pattern names
// scheme, host and port in full, so there is no wildcard anywhere and nothing
// matches by prefix.
//
// A loopback or wildcard bind yields the three loopback spellings of the port.
// That is not a widening: a browser sets Origin itself and a page cannot forge it,
// so an Origin of localhost:PORT means the page was served by whatever owns that
// port — which, on the same loopback stack, is chartr. A bind to one specific
// non-loopback address names that address alone; a wildcard bind is additionally
// reachable under names chartr cannot enumerate, and those ride the library's own
// Origin-equals-Host rule.
func originPatterns(addr net.Addr) []string {
	host, port, err := net.SplitHostPort(addr.String())
	if err != nil {
		// Not a host:port listener (a unix socket, say). There is no address to
		// scope to, so nothing is added: the library's Origin-equals-Host default
		// stays the whole check rather than anything being silently admitted.
		return devOriginPatterns()
	}

	hosts := []string{host}
	if ip := net.ParseIP(host); ip != nil && (ip.IsLoopback() || ip.IsUnspecified()) {
		hosts = []string{"127.0.0.1", "::1", "localhost"}
	}

	pats := make([]string, 0, len(hosts)+2)
	for _, h := range hosts {
		pats = append(pats, "http://"+net.JoinHostPort(h, port))
	}
	return append(pats, devOriginPatterns()...)
}
