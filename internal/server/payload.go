package server

import (
	"net/http"
	"strconv"

	"github.com/rengwu/wayfinder-harness/internal/mapscan"
	"github.com/rengwu/wayfinder-harness/internal/model"
	"github.com/rengwu/wayfinder-harness/internal/prompt"
)

// handlePayloadPreview composes and returns the payload a session for one ticket
// and role would be told (ticket 08). It is read-only inspection — the preview
// surface story 45–49 want — so it is a GET, and it reads the prompt library and
// the map fresh off disk so an edit to a materialized prompt shows up on the next
// preview with no restart. The `role` query selects which role prompt resolves;
// composition is otherwise identical.
func (s *Server) handlePayloadPreview(w http.ResponseWriter, r *http.Request) {
	e, ok := s.reg.Get(r.PathValue("id"))
	if !ok {
		httpError(w, http.StatusNotFound, "no such space")
		return
	}

	role := r.URL.Query().Get("role")
	if role == "" {
		httpError(w, http.StatusBadRequest, "role query parameter is required")
		return
	}

	num, err := strconv.Atoi(r.PathValue("num"))
	if err != nil {
		httpError(w, http.StatusBadRequest, "ticket number must be an integer")
		return
	}

	// Discover fresh so a just-edited map (or prompt) is reflected — the preview's
	// whole point is to show the current truth on disk, not a cached snapshot.
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

	bundle := prompt.Bundle{
		MapName:     m.Name,
		MapBody:     m.Body,
		TicketNum:   tk.Num,
		TicketTitle: tk.Title,
		TicketBody:  tk.Body,
		Blockers:    blockersOf(m, tk),
	}

	payload, err := prompt.Compose(prompt.ComposeInput{
		Role:    role,
		DataDir: s.opts.DataDir,
		RepoDir: e.Path,
		Bundle:  bundle,
	})
	if err != nil {
		// An unknown role is the caller's to fix; every other input was validated.
		httpError(w, http.StatusBadRequest, err.Error())
		return
	}

	writeJSON(w, http.StatusOK, payload)
}

// blockersOf gathers a ticket's blockers with their answers pulled inline from
// the same map (ADR 0005), mirroring the detail pane's inline-blocker reading: a
// blocker resolved in this map contributes its answer prose; one that names a
// ticket the map does not hold is surfaced as missing rather than dropped.
func blockersOf(m model.Map, tk model.Ticket) []prompt.Blocker {
	if len(tk.BlockedBy) == 0 {
		return nil
	}
	out := make([]prompt.Blocker, 0, len(tk.BlockedBy))
	for _, n := range tk.BlockedBy {
		bt, found := findTicket(m, n)
		if !found {
			out = append(out, prompt.Blocker{Num: n, Title: "(missing ticket)"})
			continue
		}
		out = append(out, prompt.Blocker{
			Num:    bt.Num,
			Title:  bt.Title,
			Answer: prompt.AnswerSection(bt.Body),
		})
	}
	return out
}

func findMap(maps []model.Map, slug string) (model.Map, bool) {
	for _, m := range maps {
		if m.Slug == slug {
			return m, true
		}
	}
	return model.Map{}, false
}

func findTicket(m model.Map, num int) (model.Ticket, bool) {
	for _, t := range m.Tickets {
		if t.Num == num {
			return t, true
		}
	}
	return model.Ticket{}, false
}
