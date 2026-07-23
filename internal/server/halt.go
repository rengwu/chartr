package server

import (
	"errors"
	"fmt"
	"net/http"
	"os"
	"path/filepath"

	"github.com/rengwu/chartr/internal/adapter"
	"github.com/rengwu/chartr/internal/mapscan"
	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/registry"
	"github.com/rengwu/chartr/internal/terminal"
)

// The death halt (ticket 10). When a session's process exits, its tab stays pinned
// to its ticket — dead, scrollback intact — and chartr does nothing on its
// own: no auto-kill, no timeout, no auto-requeue. The operator resolves it exactly
// three ways, each a plain HTTP action so nothing changes without a call:
//
//   - resume  — relaunch the same session on the same ticket (same-ticket crash
//     recovery only, ADR 0005 as amended): its claim stands, its payload is
//     re-materialized, and the agent walks back into its own working tree.
//   - respawn — a fresh session on the same ticket: a new claim supersedes the
//     stale one and a fresh payload is composed, so nothing carries across.
//   - release — clear the claim back to the frontier: the ticket derives open and
//     takeable again, recorded as its own commit.
//
// Each requires the session to be dead first: a death halts to the human, and
// these are the human's answers to it. A live session is refused — the operator
// ends it (by typing into its TUI) before choosing.

// haltTarget resolves the {space, session-tab} a halt action names, writing the
// error response and returning ok=false when it cannot: an unknown space or
// session, a tab that is an ad-hoc shell rather than a session, or a session still
// live. On ok it guarantees info.Session is non-nil and the session is dead.
func (s *Server) haltTarget(w http.ResponseWriter, r *http.Request) (registry.Entry, terminal.Info, bool) {
	e, ok := s.reg.Get(r.PathValue("id"))
	if !ok {
		httpError(w, http.StatusNotFound, "no such space")
		return registry.Entry{}, terminal.Info{}, false
	}
	info, ok := s.terms.Lookup(r.PathValue("sid"))
	if !ok || info.SpaceID != e.ID {
		httpError(w, http.StatusNotFound, "no such session")
		return registry.Entry{}, terminal.Info{}, false
	}
	if info.Session == nil {
		httpError(w, http.StatusBadRequest, "that tab is an ad-hoc shell, not a session")
		return registry.Entry{}, terminal.Info{}, false
	}
	if info.Alive {
		httpError(w, http.StatusConflict, "the session is still live — end it before resuming, respawning, or releasing it")
		return registry.Entry{}, terminal.Info{}, false
	}
	return e, info, true
}

// handleResume relaunches a dead session on its own ticket — same-ticket crash
// recovery (ADR 0005 as amended). The claim stands (no new commit), the archived
// payload is re-materialized to the gitignored path the opener points at, and the
// agent is launched afresh under the same session id, so it walks back into the
// working tree it left. The session's own agent is re-resolved from the library,
// so a resume after the operator fixed a missing CLI picks up the fix and
// re-checks presence — and one whose agent has since gone is refused rather than
// silently relaunched on something else.
func (s *Server) handleResume(w http.ResponseWriter, r *http.Request) {
	e, info, ok := s.haltTarget(w, r)
	if !ok {
		return
	}
	sess := *info.Session

	if s.terms.HasLiveSession(e.ID) {
		httpError(w, http.StatusConflict, "this space already has a live session — end it before resuming")
		return
	}
	spec, status, err := agentSpec(s.resolve(e), sess.AgentName)
	if err != nil {
		httpError(w, status, err.Error())
		return
	}

	payloadPath, err := s.ensureSessionPayload(e.Path, info.ID)
	if err != nil {
		httpError(w, http.StatusConflict, err.Error())
		return
	}

	// Drop the pinned dead tab so the same session id can seat a live one again.
	s.terms.Discard(info.ID)

	launch := adapter.Command(adapter.Spawn{
		Adapter: spec.Adapter,
		Args:    spec.Args,
		Prompt:  adapter.Opener(payloadPath),
		Deliver: spec.Prompt,
	})
	if _, err := s.terms.OpenSession(e.ID, e.Path, info.ID, launch.Name, launch.Args,
		launch.TypeIn, terminal.Session{
			MapSlug:   sess.MapSlug,
			TicketNum: sess.TicketNum,
			Role:      sess.Role,
			Agent:     spec.Adapter,
			AgentName: spec.Name,
		}); err != nil {
		if errors.Is(err, terminal.ErrSessionExists) {
			httpError(w, http.StatusConflict, "this space already has a live session")
			return
		}
		httpError(w, http.StatusInternalServerError, "relaunching the session: "+err.Error())
		return
	}

	writeJSON(w, http.StatusOK, map[string]any{
		"sessionId": info.ID,
		"ticketNum": sess.TicketNum,
		"role":      sess.Role,
		"resumed":   true,
	})
}

// handleRespawn starts a fresh session on the same ticket a dead session held. A
// new claim supersedes the stale one (re-stamped in place, its own pathspec-limited
// commit) and a fresh payload is composed, so nothing carries across from the dead
// attempt (ADR 0005) — this is the "start over, cleanly" answer, distinct from
// resume's crash recovery. The new session lands only after it launches; the dead
// tab is dropped last, so a failed respawn leaves the halt in place to retry.
func (s *Server) handleRespawn(w http.ResponseWriter, r *http.Request) {
	e, info, ok := s.haltTarget(w, r)
	if !ok {
		return
	}
	sess := *info.Session

	if s.terms.HasLiveSession(e.ID) {
		httpError(w, http.StatusConflict, "this space already has a live session — end it before respawning")
		return
	}

	m, tk, ok := s.frozenTicket(w, e, sess.MapSlug, sess.TicketNum)
	if !ok {
		return
	}
	// Respawn re-claims a ticket a dead session still holds; anything else
	// (already released, resolved) is not this halt's to re-take.
	if tk.Status != "claimed" {
		httpError(w, http.StatusConflict, "ticket is no longer claimed by this session — nothing to respawn")
		return
	}
	// "Start over cleanly" composes a fresh payload and writes a fresh claim; it
	// does not change what executes. So the agent comes from the dead session
	// itself, not from re-deciding — and one that has since been deregistered or
	// fallen off PATH is refused with the message any other absent agent gets,
	// surfaced rather than silently substituted.
	spec, status, err := agentSpec(s.resolve(e), sess.AgentName)
	if err != nil {
		httpError(w, status, err.Error())
		return
	}

	result, status, err := s.launchSession(sessionLaunch{
		entry:     e,
		slug:      sess.MapSlug,
		m:         m,
		tk:        tk,
		role:      sess.Role,
		spec:      spec,
		sessionID: newSessionID(),
	})
	if err != nil {
		httpError(w, status, err.Error())
		return
	}
	// The fresh session is live; retire the dead tab it replaces.
	s.terms.Discard(info.ID)

	writeJSON(w, http.StatusOK, result)
}

// handleRelease clears a dead session's claim back to the frontier: the ticket
// derives open and takeable again, recorded as its own pathspec-limited commit
// (never an amend, never a push — ADR 0008). The pinned dead tab is then dropped.
// This is the "abandon the attempt" answer — it retries nothing and touches no
// prose the session wrote; the ticket file is the agent's record alone.
func (s *Server) handleRelease(w http.ResponseWriter, r *http.Request) {
	e, info, ok := s.haltTarget(w, r)
	if !ok {
		return
	}
	sess := *info.Session

	m, ok := findMap(mapscan.Discover(e.Path), sess.MapSlug)
	if !ok {
		httpError(w, http.StatusNotFound, "no such map")
		return
	}
	ticketPath, err := ticketFilePath(m.Dir, sess.TicketNum)
	if err != nil {
		httpError(w, http.StatusNotFound, err.Error())
		return
	}
	if err := writeReleaseCommit(e.Path, ticketPath, info.ID); err != nil {
		httpError(w, http.StatusInternalServerError, "releasing the claim: "+err.Error())
		return
	}
	// Drop the pinned dead tab; its notify re-derives the now-open ticket too.
	s.terms.Discard(info.ID)

	writeJSON(w, http.StatusOK, map[string]any{
		"sessionId": info.ID,
		"ticketNum": sess.TicketNum,
		"released":  true,
	})
}

// frozenTicket discovers the map and ticket a session names, fresh off disk (as
// spawn does). It writes the error response and returns ok=false when either is
// gone (a map or ticket renamed out from under a pinned dead session).
func (s *Server) frozenTicket(w http.ResponseWriter, e registry.Entry, slug string, num int) (model.Map, model.Ticket, bool) {
	m, found := findMap(mapscan.Discover(e.Path), slug)
	if !found {
		httpError(w, http.StatusNotFound, "no such map")
		return model.Map{}, model.Ticket{}, false
	}
	tk, found := findTicket(m, num)
	if !found {
		httpError(w, http.StatusNotFound, "no such ticket")
		return model.Map{}, model.Ticket{}, false
	}
	return m, tk, true
}

// ensureSessionPayload guarantees the gitignored payload a session's opener points
// at is on disk, restoring it from the per-session archive when the operator has
// cleaned the run directory. It re-asserts the `*` .gitignore either way, so the
// restored payload can never be swept into a commit (ADR 0008). It errors only when
// the archive is gone too, in which case the session cannot be resumed.
func (s *Server) ensureSessionPayload(repo, sid string) (string, error) {
	path := filepath.Join(repo, sessionRunDir, sid, "payload.md")
	if _, err := os.Stat(path); err == nil {
		_ = os.WriteFile(filepath.Join(repo, sessionRunDir, ".gitignore"), []byte("*\n"), 0o644)
		return path, nil
	}
	archived, err := os.ReadFile(filepath.Join(s.opts.DataDir, "sessions", sid, "payload.md"))
	if err != nil {
		return "", fmt.Errorf("the session's payload is gone and cannot be restored — respawn a fresh session instead")
	}
	return s.writeSessionPayload(repo, sid, string(archived))
}
