package main

import "testing"

// Launching a browser needs a desktop, so what is testable here is the guard in
// front of it — the one thing standing between a URL that came out of a terminal
// and an application the OS opener would happily launch.

func TestCheckExternalURLAcceptsWebURLs(t *testing.T) {
	for _, raw := range []string{
		"http://example.com",
		"https://example.com/docs?q=1#frag",
		"http://127.0.0.1:8787/",
		"HTTPS://EXAMPLE.COM/shouty", // url.Parse lower-cases the scheme
	} {
		if err := checkExternalURL(raw); err != nil {
			t.Errorf("checkExternalURL(%q) = %v, want it opened", raw, err)
		}
	}
}

func TestCheckExternalURLRefusesEverythingElse(t *testing.T) {
	for _, raw := range []string{
		"file:///etc/passwd",   // the opener would launch an app on a local file
		"javascript:alert(1)",  // meaningless to the OS, meaningful to a webview
		"vscode://file/tmp/x",  // a registered custom scheme is still an app launch
		"mailto:a@example.com", // not a page; not this hook's job
		"/etc/passwd",          // no scheme at all
		"example.com",          // a bare host is not an absolute URL
		"http://",              // scheme but no host
		"",
	} {
		if err := checkExternalURL(raw); err == nil {
			t.Errorf("checkExternalURL(%q) = nil, want it refused", raw)
		}
	}
}
