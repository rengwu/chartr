package server

import (
	"fmt"
	"net"
)

// exposedBind reports whether the address chartr actually bound is reachable by
// anything other than this machine itself.
//
// It is asked of the *resolved* listener address rather than the `-addr` string,
// because that string does not say what happened: `:9000` and `0.0.0.0:9000` are
// a wildcard bind spelled two ways, and only the listener knows which interface
// it ended up on. The desktop shell's `127.0.0.1:0` is the case that must stay
// quiet — it is loopback, and a warning every desktop launch prints is a warning
// nobody reads.
//
// This is deliberately only a question, with the answer acted on at one call site
// in Serve. If the trust-boundary map later decides a non-loopback bind should be
// refused outright rather than warned about (its ticket 04), the refusal replaces
// that one call; nothing here pre-empts that decision.
func exposedBind(addr net.Addr) bool {
	host, _, err := net.SplitHostPort(addr.String())
	if err != nil {
		// Not a host:port listener at all (a unix socket). Nothing on the network
		// can reach it, so there is nothing to warn about.
		return false
	}
	if normalizeHost(host) == "localhost" {
		return false
	}
	// Anything that is not a loopback IP is treated as exposed, including a host
	// that does not parse as an address. net.Listen resolves names, so an addr
	// that is not an IP is not a case chartr produces; if one ever appears, the
	// warning is the safe way to be wrong about it.
	ip := net.ParseIP(host)
	return ip == nil || !ip.IsLoopback()
}

// exposureWarning is the line an operator sees when they bind chartr somewhere
// other than loopback. It says what is exposed rather than that something is:
// chartr has no authentication of any kind, so reaching the port is the whole of
// the access check, and the API behind it opens shells and spawns agents in the
// operator's own account. The origin and Host checks do not help here — neither
// one is in the way of a client that is not a browser.
func exposureWarning(addr net.Addr) string {
	return fmt.Sprintf("chartr: WARNING: listening on %s, which is not loopback. "+
		"chartr has no authentication: anyone who can reach this port can open shells, "+
		"run commands and spawn agents on this machine as you, and read any file you can. "+
		"Bind 127.0.0.1 (the default) unless you meant to expose it.", addr)
}
