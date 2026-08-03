package server_test

import (
	"net/http"
	"os"
	"path/filepath"
	"strconv"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
)

// websocket-origin-fix ticket 03 at the process boundary: a write must declare a
// JSON body, so a cross-origin page cannot cause one with a request the browser
// sends without ever asking us first.
//
// The hole these close is the CORS preflight, or rather its absence. A POST whose
// content type is `text/plain`, `application/x-www-form-urlencoded` or
// `multipart/form-data` — and one carrying no content type at all — is a *simple
// request*: the browser delivers it cross-origin and only then refuses to show
// the response to the page that sent it. The attacker cannot read the answer, but
// every route below is a side effect worth causing blind: registering a space
// runs `git init` in the folder it names, and the bodyless POSTs open shells and
// write into the operator's repository.
//
// This is defence in depth and not the remedy. Once tickets 01 and 02 are in
// place a foreign page cannot usefully reach these routes at all, and a rebinding
// attacker is same-origin and can set any content type they like. It is the cheap
// second lock that holds if one of the other two is ever misconfigured or
// regressed — so every assertion below is on the *effect*, not the status code.

// The content types a browser will POST cross-origin with no preflight. JSON
// travels through `text/plain` unchanged, which is what makes that one more than
// a curiosity: the body a handler decodes is byte-for-byte the body it wanted.
var simpleContentTypes = []string{
	"text/plain;charset=UTF-8",
	"application/x-www-form-urlencoded",
	"multipart/form-data; boundary=----probe",
}

// Registering is the worst of the body-bearing routes: it takes a path from the
// request and runs `git init` in it when it is not already a repository. A
// refusal that still ran that would be no refusal at all, so the folder is
// checked, not just the status.
func TestASimpleContentTypePostRegistersNothing(t *testing.T) {
	h := chartrtest.Start(t)

	for _, contentType := range simpleContentTypes {
		plain := t.TempDir() // a folder, not a repo
		body := `{"path":` + strconv.Quote(plain) + `}`

		code, _ := h.PostWithContentType("/api/spaces", contentType, body)
		if code != http.StatusUnsupportedMediaType {
			t.Errorf("POST /api/spaces as %q = %d, want %d",
				contentType, code, http.StatusUnsupportedMediaType)
		}
		if _, err := os.Stat(filepath.Join(plain, ".git")); !os.IsNotExist(err) {
			t.Errorf("a %q POST ran git init in %s", contentType, plain)
		}
		if registeredPath(t, h, plain) {
			t.Errorf("a %q POST registered %s as a space", contentType, plain)
		}
	}
}

// The routes that take no body are why "carries a body" is not the line. A bare
// `fetch(url, {method: 'POST'})` sends neither a body nor a content type and is
// simple on both counts, and what it reaches here opens a real shell in the
// operator's working tree.
func TestABarePostOpensNoShell(t *testing.T) {
	h := chartrtest.Start(t)
	space := register(t, h, chartrtest.NewSpaceRepo(t)).ID

	code, _ := h.PostWithContentType("/api/spaces/"+space+"/terminals", "", "")
	if code != http.StatusUnsupportedMediaType {
		t.Errorf("bare POST /api/spaces/{id}/terminals = %d, want %d",
			code, http.StatusUnsupportedMediaType)
	}
	if n := len(openTerminals(t, h, space)); n != 0 {
		t.Errorf("a bare POST opened %d shell(s) in the space, want none", n)
	}

	// The same route, declared, still opens one — so the refusal above is the
	// undeclared content type's doing and not the route being broken.
	h.OpenTerminal(space)
	if n := len(openTerminals(t, h, space)); n != 1 {
		t.Errorf("a declared POST left %d shell(s) open, want 1", n)
	}
}

// What the operator's own cockpit sends has to keep working, and it does not send
// a bare `application/json`: `fetch` appends a charset, and other clients spell
// the media type in whatever case they like. The header is parsed, not compared.
func TestJSONWithAParameterOrOddCasingIsAccepted(t *testing.T) {
	h := chartrtest.Start(t)

	for _, contentType := range []string{"application/json; charset=utf-8", "Application/JSON"} {
		plain := t.TempDir()
		body := `{"path":` + strconv.Quote(plain) + `}`

		code, out := h.PostWithContentType("/api/spaces", contentType, body)
		if code != http.StatusOK {
			t.Fatalf("POST /api/spaces as %q = %d, body %s", contentType, code, out)
		}
		if !registeredPath(t, h, plain) {
			t.Errorf("a %q POST did not register %s", contentType, plain)
		}
	}
}

// A read declares nothing and is asked for nothing. The websocket handshake is
// the case worth naming: it is a GET with no body, and a gate that asked it for a
// content type would take the cockpit down with it.
func TestAGetWithNoBodyIsUnaffected(t *testing.T) {
	h := chartrtest.Start(t)

	if code, body := h.Get("/api/health"); code != http.StatusOK {
		t.Errorf("GET /api/health = %d, body %s", code, body)
	}

	cc := h.DialControl(ctx(t))
	defer cc.Close()
	if snap := cc.ReadSnapshot(ctx(t)); snap.Spaces == nil {
		t.Error("the control socket sent no snapshot")
	}
}

// The method split protects what the content type does not: a cross-origin
// DELETE forces a preflight this server never answers, which is what keeps
// `DELETE /api/config/agents/{name}` and the rest out of reach. It carries no
// body here and is asked for no content type, and that has to stay true — a
// bodyless DELETE has nothing to decode, and requiring a header on it would only
// break the operator's own client.
func TestABodylessDeleteNeedsNoContentType(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	space := register(t, h, repo).ID

	if code, body := h.Delete("/api/spaces/" + space); code != http.StatusNoContent {
		t.Fatalf("DELETE /api/spaces/{id} = %d, body %s", code, body)
	}
	if registeredPath(t, h, repo) {
		t.Errorf("%s is still registered after a DELETE that answered 204", repo)
	}
}

// registeredPath reports whether a folder is a space in the current snapshot —
// the public answer to "did that request register anything".
func registeredPath(t *testing.T, h *chartrtest.Chartr, path string) bool {
	t.Helper()
	for _, s := range h.Snapshot(ctx(t)).Spaces {
		if s.Path == path {
			return true
		}
	}
	return false
}

// openTerminals is the space's ad-hoc shells as the snapshot reports them.
func openTerminals(t *testing.T, h *chartrtest.Chartr, space string) []string {
	t.Helper()
	var ids []string
	for _, term := range findSpace(t, h.Snapshot(ctx(t)), space).Terminals {
		ids = append(ids, term.ID)
	}
	return ids
}
