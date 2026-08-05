package server

import (
	"net/http"
	"strconv"

	"github.com/rengwu/chartr/internal/gitroot"
	"github.com/rengwu/chartr/internal/mapscan"
	"github.com/rengwu/chartr/internal/terminal"
)

// Release, keyed on the ticket rather than on a session.
//
// The death halt's release (halt.go) reaches a claim through the dead session tab
// that holds it — which works only for as long as that tab exists, and tabs live
// in memory. A chartr restart drops every tab while the claim on disk survives, so
// the ticket derives `claimed`, falls off the frontier, and the three halt actions
// that could have cleared it are unreachable: the ticket is stuck with no way back.
// The same dead end is reached by a claim committed on another machine, or one
// whose dead tab the operator dismissed.
//
// So release is offered a second way in — same write, same commit, addressed by
// {space, map, ticket} instead of {space, session}. It is the *only* ticket-keyed
// lifecycle action: resume and respawn stay session-keyed because both relaunch a
// session, and a session is exactly what an orphaned claim no longer has. The way
// back onto an orphaned ticket is release, then spawn — two acts, so the ticket's
// history reads claim → release → claim rather than a claim silently replacing a
// claim.
//
// chartr still takes none of this on its own (ADR 0005): there is no startup sweep
// of stale claims and no timeout. It cannot tell a claim its own restart orphaned
// from a teammate's live one on a shared map, and the answer to that is the
// operator, not a heuristic.
func (s *Server) handleReleaseTicket(w http.ResponseWriter, r *http.Request) {
	e, ok := s.repoSpace(w, r)
	if !ok {
		return
	}
	num, err := strconv.Atoi(r.PathValue("num"))
	if err != nil {
		httpError(w, http.StatusBadRequest, "ticket number must be an integer")
		return
	}

	// Discover fresh, as spawn does, so the release acts on the truth on disk
	// rather than a snapshot the operator's click may have outlived.
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
	// Only a claim can be released. A ticket that is open, resolved, or ruled out
	// has nothing to clear, and saying so is what keeps this from being a general
	// "edit the frontmatter" endpoint — the ticket file is the agent's record, and
	// release touches exactly the two keys chartr itself stamped.
	if tk.Status != "claimed" {
		httpError(w, http.StatusConflict, "ticket is not claimed — there is nothing to release")
		return
	}

	// A claim this chartr is actively running is not stale, whatever the frontmatter
	// says: releasing it would strip the claim out from under a live agent that is
	// still editing the tree. The operator ends the session first — the same order
	// the halt actions insist on, refused here for the same reason.
	if _, held := liveSessionOn(s.terms.ForSpace(e.ID), slug, num); held {
		httpErrorCode(w, http.StatusConflict, codeLiveSession,
			"a live session in this space still holds this ticket — end it before releasing the claim")
		return
	}

	ticketPath, err := ticketFilePath(m.Dir, num)
	if err != nil {
		httpError(w, http.StatusNotFound, err.Error())
		return
	}
	gitRoot, err := gitroot.Resolve(e.Path)
	if err != nil {
		httpError(w, http.StatusInternalServerError, "locating Git root for the release: "+err.Error())
		return
	}
	// The commit names the session being released, read off the ticket's own
	// frontmatter — the claim is the only thing that knows, since by construction
	// there may be no tab left to ask.
	if err := writeReleaseCommit(gitRoot, ticketPath, tk.ClaimedBy); err != nil {
		httpError(w, http.StatusInternalServerError, "releasing the claim: "+err.Error())
		return
	}

	// A dead tab pinned to this ticket outlived its claim: its resume and respawn
	// now point at a ticket nobody holds. Drop it, exactly as the halt's own release
	// does, so the released ticket leaves no session behind offering to re-enter it.
	for _, info := range s.terms.ForSpace(e.ID) {
		if info.Session != nil && !info.Alive &&
			info.Session.MapSlug == slug && info.Session.TicketNum == num {
			s.terms.Discard(info.ID)
		}
	}
	// Discard pushes when it drops a tab, but a release with no tab to drop must
	// push on its own — the ticket is open again and the star has to say so before
	// the response lands.
	s.rebuild()

	writeJSON(w, http.StatusOK, map[string]any{
		"ticketNum": num,
		"claimedBy": tk.ClaimedBy,
		"released":  true,
	})
}

// liveSessionOn finds a space's live session bound to one ticket, if it has one.
// It answers a narrower question than HasLiveSession's "is anything running here":
// a release is refused only by the session holding *this* ticket, never by one
// working elsewhere on the map.
func liveSessionOn(tabs []terminal.Info, slug string, num int) (terminal.Info, bool) {
	for _, info := range tabs {
		if info.Alive && info.Session != nil &&
			info.Session.MapSlug == slug && info.Session.TicketNum == num {
			return info, true
		}
	}
	return terminal.Info{}, false
}
