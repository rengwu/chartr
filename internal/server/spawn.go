package server

import (
	"crypto/rand"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"os"
	"path/filepath"
	"strconv"
	"time"

	"github.com/rengwu/chartr/internal/adapter"
	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/mapscan"
	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/prompt"
	"github.com/rengwu/chartr/internal/registry"
	"github.com/rengwu/chartr/internal/terminal"
)

// sessionRunDir is the gitignored directory, inside each space, that holds live
// sessions' composed payloads (ADR 0005 — one inspectable file per session), and
// the ideate on-ramp's starter prompt (ticket 15), which reuses the same
// writeSessionPayload path though it is deliberately not a session. It sits under
// the chartr's committed `.chartr/` directory but is itself never
// committed: the chartr drops a `.gitignore` of `*` beside it so an agent's
// `git commit -a` can never sweep a payload into the audit trail (ADR 0008).
const sessionRunDir = ".chartr/run"

// handleSpawn is the product's tracer bullet (ticket 09): from a frontier ticket
// it wires the whole chain — resolve the role's binding, hard-block a missing
// agent, compose the payload, write the claim commit, drop the gitignored payload
// and its archived copy, and launch the agent's own TUI with the read-this-file
// opener typed in. The session seats as a tab bound to exactly one ticket.
//
// Every step that can fail does so *before* anything launches, so a refused spawn
// leaves the space exactly as it was — except the claim, which by ADR 0008 stands
// once written (a launch failure after a good claim is a dead session the human
// resolves, never an automatic rollback).
func (s *Server) handleSpawn(w http.ResponseWriter, r *http.Request) {
	e, ok := s.reg.Get(r.PathValue("id"))
	if !ok {
		httpError(w, http.StatusNotFound, "no such space")
		return
	}
	num, err := strconv.Atoi(r.PathValue("num"))
	if err != nil {
		httpError(w, http.StatusBadRequest, "ticket number must be an integer")
		return
	}
	var body struct {
		Role string `json:"role"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	role := body.Role
	if role == "" {
		httpError(w, http.StatusBadRequest, "role is required")
		return
	}

	// Discover fresh (as the preview does) so the spawn acts on the truth on disk,
	// not a cached snapshot — the ticket may have been resolved or claimed since the
	// last push.
	slug := r.PathValue("slug")
	m, found := findMap(mapscan.Discover(e.Path), slug)
	if !found {
		httpError(w, http.StatusNotFound, "no such map")
		return
	}
	tk, found := findTicket(m, num)
	if !found {
		httpError(w, http.StatusNotFound, "no such ticket")
		return
	}

	// Resolve the space's config once — kinds and bindings both come from it. The
	// committed kind declaration overlays the discovered map (Discover only carries
	// the convention guess), so the map reads classified exactly as the pushed model
	// shows it.
	res := s.resolve(e)
	if kind, ok := res.Kinds[slug]; ok {
		m.Kind = kind
	}

	// Which roles are offered follows the map's kind; an unclassified map offers
	// none (ADR 0007). Both refusals are the operator's to resolve — classify the
	// map, or pick a role that belongs to its lifecycle.
	if m.Kind == model.KindUnclassified {
		httpError(w, http.StatusConflict, "this map is unclassified and offers no sessions — classify it first")
		return
	}
	if !config.KindOffersRole(m.Kind, role) {
		httpError(w, http.StatusBadRequest, "role "+role+" is not offered by a "+m.Kind+" map")
		return
	}

	// Every role is a fresh spawn onto the frontier (open, unclaimed, every blocker
	// resolved): anything already claimed, closed, or held behind an unresolved
	// blocker is not its to take.
	if !tk.Frontier {
		httpError(w, http.StatusConflict, "ticket is not on the frontier — it is not a takeable ticket")
		return
	}

	// Resolve the binding and hard-block an absent agent *here*, before any write:
	// the ordinary missing-CLI case is a doorstep diagnosis naming the binding, its
	// source layer, and the local-override fix — and it blocks nothing else (story
	// 40).
	binding, ok := bindingFor(res, role)
	if !ok {
		httpError(w, http.StatusInternalServerError, "no binding for role "+role)
		return
	}
	if !binding.Present {
		httpError(w, http.StatusConflict, binding.Missing)
		return
	}

	// One session per space at a time (spec, State model). Check before writing the
	// claim so we never claim a ticket we cannot then seat; OpenSession re-checks
	// under its own lock to close the race.
	if s.terms.HasLiveSession(e.ID) {
		httpError(w, http.StatusConflict, "this space already has a live session — end it before spawning another")
		return
	}

	// From here the mechanics are shared with respawn (ticket 10): compose the
	// payload, write the claim commit, drop the gitignored and archived copies, and
	// launch the TUI. A fresh spawn mints a new session id.
	result, status, err := s.launchSession(sessionLaunch{
		entry:     e,
		slug:      slug,
		m:         m,
		tk:        tk,
		role:      role,
		binding:   binding,
		sessionID: newSessionID(),
	})
	if err != nil {
		httpError(w, status, err.Error())
		return
	}
	writeJSON(w, http.StatusOK, result)
}

// sessionLaunch carries the resolved inputs the launch mechanics need once every
// gate has passed. It is the seam a fresh spawn and a respawn share (ticket 10):
// both compose the same payload, write the same claim, and open the same kind of
// session tab — they differ only in the gating that precedes this and in whether
// the session id is new.
type sessionLaunch struct {
	entry     registry.Entry
	slug      string
	m         model.Map
	tk        model.Ticket
	role      string
	binding   config.Resolved
	sessionID string
}

// launchSession runs the spawn mechanics: it composes the payload fresh
// off disk (the same assembly the preview shows — ADR 0005), writes the claim
// commit (ADR 0008), drops the payload gitignored inside the space and archived in
// chartr state (story 49), and launches the agent's TUI with the read-this-file
// opener typed in. It returns the action's result and, on failure, the HTTP status
// the caller should surface. Every write that can fail happens here in order, so a
// failure after the claim leaves the claim standing (never rolled back) for the
// death halt to resolve.
func (s *Server) launchSession(in sessionLaunch) (map[string]any, int, error) {
	payload, err := prompt.Compose(prompt.ComposeInput{
		Role:  in.role,
		Roots: s.skillRoots(in.entry.Path),
		Bundle: prompt.Bundle{
			MapName:     in.m.Name,
			MapBody:     in.m.Body,
			TicketNum:   in.tk.Num,
			TicketTitle: in.tk.Title,
			TicketBody:  in.tk.Body,
			Blockers:    blockersOf(in.m, in.tk),
		},
	})
	if err != nil {
		return nil, http.StatusBadRequest, err
	}

	sum := sha256.Sum256([]byte(payload.Markdown))
	payloadSHA := hex.EncodeToString(sum[:])
	claimedAt := time.Now().UTC().Format(time.RFC3339)

	ticketPath, err := ticketFilePath(in.m.Dir, in.tk.Num)
	if err != nil {
		return nil, http.StatusNotFound, err
	}

	// The claim commit — the chartr's one write here (ADR 0008), pathspec-limited
	// to the ticket file and carrying the binding and payload-hash trailers. On a
	// respawn the ticket already carries a stale claim; stampClaim replaces it, so
	// the new session id supersedes the dead one.
	if err := writeClaimCommit(in.entry.Path, ticketPath, in.sessionID, claimedAt, claim{
		SessionID:   in.sessionID,
		Role:        in.role,
		Agent:       in.binding.Adapter,
		Args:        in.binding.Args,
		PayloadSHA:  payloadSHA,
		Skills:      payload.Skills,
		AdapterFrom: in.binding.AdapterFrom,
		ArgsFrom:    in.binding.ArgsFrom,
	}); err != nil {
		return nil, http.StatusInternalServerError, fmt.Errorf("writing the claim commit: %w", err)
	}

	payloadPath, err := s.writeSessionPayload(in.entry.Path, in.sessionID, payload.Markdown)
	if err != nil {
		return nil, http.StatusInternalServerError, fmt.Errorf("writing the session payload: %w", err)
	}
	if err := s.archivePayload(in.sessionID, payload.Markdown); err != nil {
		return nil, http.StatusInternalServerError, fmt.Errorf("archiving the session payload: %w", err)
	}

	// Resolve the binding — and the read-this-file opener — onto this agent's
	// command line (ADR 0002) and launch its TUI in a PTY. Whether the opener rides
	// the argv or gets typed in is the adapter's call, so the spawn path hands over
	// the same line either way.
	launch := adapter.Command(adapter.Spawn{
		Adapter: in.binding.Adapter,
		Args:    in.binding.Args,
		Prompt:  adapter.Opener(payloadPath),
		Deliver: in.binding.Prompt,
	})
	if _, err := s.terms.OpenSession(in.entry.ID, in.entry.Path, in.sessionID, launch.Name, launch.Args,
		launch.TypeIn, terminal.Session{
			MapSlug:   in.slug,
			TicketNum: in.tk.Num,
			Role:      in.role,
			Agent:     in.binding.Adapter,
		}); err != nil {
		// The claim already stands (ADR 0008: it is not rolled back). A live-session
		// race is a conflict; a launch failure after a present-on-PATH check is an
		// environment fault the death halt (ticket 10) picks up.
		if errors.Is(err, terminal.ErrSessionExists) {
			return nil, http.StatusConflict, errors.New("this space already has a live session")
		}
		return nil, http.StatusInternalServerError, fmt.Errorf("launching the session: %w", err)
	}

	return map[string]any{
		"sessionId":  in.sessionID,
		"ticketNum":  in.tk.Num,
		"role":       in.role,
		"agent":      in.binding.Adapter,
		"args":       in.binding.Args,
		"payloadSha": payloadSHA,
	}, http.StatusOK, nil
}

// resolve resolves a space's whole config — bindings and committed map kinds —
// across the three layers, exactly as the pushed model renders it, so the spawn
// path reads kinds and a binding's PATH presence from what the operator sees.
func (s *Server) resolve(e registry.Entry) config.Resolution {
	userTOML, _ := os.ReadFile(filepath.Join(s.opts.DataDir, userConfigName))
	workspaceTOML, _ := os.ReadFile(filepath.Join(e.Path, config.WorkspaceConfigName))
	return config.Resolve(config.Input{
		WorkspaceTOML: workspaceTOML,
		UserTOML:      userTOML,
		SpacePath:     e.Path,
	})
}

// bindingFor picks one role's resolved binding out of a resolution.
func bindingFor(res config.Resolution, role string) (config.Resolved, bool) {
	for _, b := range res.Bindings {
		if string(b.Role) == role {
			return b, true
		}
	}
	return config.Resolved{}, false
}

// writeSessionPayload writes the composed payload to the gitignored run directory
// inside the space and returns its absolute path (what the opener points the agent
// at). It (re)writes a `.gitignore` of `*` beside the payloads on every spawn, so
// the ignore holds even on a fresh clone that never had the run directory.
func (s *Server) writeSessionPayload(repo, sessionID, markdown string) (string, error) {
	runDir := filepath.Join(repo, sessionRunDir)
	if err := os.MkdirAll(filepath.Join(runDir, sessionID), 0o755); err != nil {
		return "", err
	}
	if err := os.WriteFile(filepath.Join(runDir, ".gitignore"), []byte("*\n"), 0o644); err != nil {
		return "", err
	}
	path := filepath.Join(runDir, sessionID, "payload.md")
	if err := os.WriteFile(path, []byte(markdown), 0o644); err != nil {
		return "", err
	}
	return path, nil
}

// archivePayload keeps the exact payload a session received in chartr-owned state
// outside the repository (under the data root), so the record survives the space's
// gitignored copy being cleaned and answers "what was this session told" word for
// word (story 49).
func (s *Server) archivePayload(sessionID, markdown string) error {
	dir := filepath.Join(s.opts.DataDir, "sessions", sessionID)
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return err
	}
	return os.WriteFile(filepath.Join(dir, "payload.md"), []byte(markdown), 0o644)
}

// newSessionID mints a session's stable identity — the id its tab, its claim
// trailer (claimed_by), its gitignored payload path, and its archive are all keyed
// by, so the whole spawn refers to one session everywhere. Random rather than
// sequential so two spaces' ids never collide and an id leaks no ordering.
func newSessionID() string {
	var b [6]byte
	_, _ = rand.Read(b[:])
	return "s" + hex.EncodeToString(b[:])
}
