package server

import (
	"net"
	"strings"
)

// hostRule is the set of Host values chartr answers to, derived once from the
// address the listener bound. It is the first of the gate's every-route checks
// (gate), and the one that closes DNS rebinding.
//
// Scoping the websocket handshakes to the bound origin does not close it: an
// attacker serves a page from a name whose DNS record they control, re-points it
// at 127.0.0.1 on a short TTL, and the browser now believes
// http://evil.example:8787 *is* chartr. Every same-origin protection falls at
// once — the origin equals the host, so the websocket check admits the
// handshake, responses are readable, and the whole HTTP API is open for read and
// write. Nothing about that is visible from the Origin header, because the
// browser is not lying: it really did connect to us. The one thing that gives it
// away is the name it asks for, which is not one we answer to.
type hostRule struct {
	// unscoped disables the check entirely, for a listener that is not a
	// host:port one at all (a unix socket). There is no address to scope to, and
	// there is also nothing to defend: rebinding is an attack on a *name*
	// resolving to a TCP port, and a socket a browser cannot dial has neither.
	unscoped bool
	// port every Host must carry, as the listener spelled it. A Host with no port
	// is read as the scheme default, 80 — chartr only ever serves http.
	port string
	// names admitted verbatim, lowercased and without a trailing dot.
	names []string
	// ips admitted, compared as addresses rather than as strings so ::1 and
	// ::ffff:127.0.0.1 are not two different answers.
	ips []net.IP
	// ownAddresses additionally admits any address this machine currently holds.
	// It is only set for a wildcard bind, where there is no single right name —
	// see hostRuleFor.
	ownAddresses bool
}

// hostRuleFor derives the rule from the bound address.
//
// A loopback bind admits exactly three spellings — 127.0.0.1, [::1] and
// localhost — and nothing else. That is the closed set the operator's own
// browser and the desktop shell's ephemeral-port URL actually send, so it is an
// allowlist and not a pattern.
//
// A wildcard bind (`:9000`, `0.0.0.0:9000`) has no single correct name. The rule
// is: the three loopback spellings, plus any address this machine currently
// holds, as a literal — and no name beyond `localhost`, ever.
//
// Deliberately, a name is never resolved to decide this. Resolving the Host
// header would defeat the whole check: the rebinding name resolves to 127.0.0.1
// precisely because the attacker pointed it there, so a lookup would agree that
// evil.example is one of our own addresses and admit it. That is why the
// wildcard rule refuses names rather than checking where they point, and the
// cost is real and worth stating: a machine bound to 0.0.0.0 and reached at its
// own hostname (`http://mybox.local:9000/`) is refused, and the operator has to
// use the IP. Non-loopback binds are already the configuration the map warns
// about; narrowing here rather than accepting names is the conservative half of
// that warning, not a surprise on top of it.
func hostRuleFor(addr net.Addr) hostRule {
	host, port, err := net.SplitHostPort(addr.String())
	if err != nil {
		return hostRule{unscoped: true}
	}

	loopback := []net.IP{net.IPv4(127, 0, 0, 1), net.IPv6loopback}
	ip := net.ParseIP(host)
	switch {
	case ip == nil:
		// A listener that named a host rather than an address. Nothing in chartr
		// produces one; if something did, that name alone is the address.
		return hostRule{port: port, names: []string{normalizeHost(host)}}
	case ip.IsUnspecified():
		return hostRule{port: port, names: []string{"localhost"}, ips: loopback, ownAddresses: true}
	case ip.IsLoopback():
		return hostRule{port: port, names: []string{"localhost"}, ips: loopback}
	default:
		return hostRule{port: port, ips: []net.IP{ip}}
	}
}

// allows reports whether a request's Host header names the address we are
// listening on.
func (ru hostRule) allows(hostHeader string) bool {
	if ru.unscoped {
		return true
	}
	if hostHeader == "" {
		return false
	}

	host, port, err := net.SplitHostPort(hostHeader)
	if err != nil {
		// No port, so the scheme's default. A bare IPv6 literal also lands here
		// (`[::1]` unbracketed by SplitHostPort's reckoning); normalizeHost takes
		// the brackets off.
		host, port = hostHeader, "80"
	}
	if port != ru.port {
		return false
	}

	host = normalizeHost(host)
	for _, name := range ru.names {
		if host == name {
			return true
		}
	}

	ip := net.ParseIP(host)
	if ip == nil {
		// A name we do not answer to. It is refused on its face and never
		// resolved — see hostRuleFor.
		return false
	}
	for _, own := range ru.ips {
		if own.Equal(ip) {
			return true
		}
	}
	return ru.ownAddresses && isOwnAddress(ip)
}

// normalizeHost puts a host into the one spelling comparisons are made in:
// lowercase, no surrounding brackets, no trailing root dot. `LOCALHOST.` and
// `localhost` are the same name; so are `[::1]` and `::1`. Nothing here widens
// anything — `evil.example.` normalizes to `evil.example`, which is refused
// either way.
func normalizeHost(host string) string {
	host = strings.ToLower(host)
	host = strings.TrimSuffix(host, ".")
	if len(host) >= 2 && host[0] == '[' && host[len(host)-1] == ']' {
		host = host[1 : len(host)-1]
	}
	return host
}

// isOwnAddress reports whether ip is one this machine currently holds. It is
// asked at request time rather than at startup so a laptop that joins a network
// — or drops a VPN — does not answer to a stale set until it is restarted, and
// it is only ever reached by a request the static set already turned down, so
// the lookup is off the ordinary path.
func isOwnAddress(ip net.IP) bool {
	addrs, err := net.InterfaceAddrs()
	if err != nil {
		return false
	}
	for _, a := range addrs {
		if n, ok := a.(*net.IPNet); ok && n.IP.Equal(ip) {
			return true
		}
	}
	return false
}
