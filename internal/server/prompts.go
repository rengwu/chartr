package server

import (
	"encoding/json"
	"errors"
	"fmt"
	"net/http"

	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/prompts"
	"github.com/rengwu/chartr/internal/registry"
)

// The prompt catalog's write half: create, edit, delete a preset, and set which
// presets a space applies at launch. Everything the pane reads rides the model
// push, so these four actions are the whole HTTP surface — there is never a
// second client-side store, and each one ends in a rebuild so the row the
// operator just changed comes back over the control socket.
//
// The catalog is the authority on what is legal; this file maps its errors onto
// status codes and validates only what it alone knows: that a space's selection
// names presets that exist.

// promptStatus maps a catalog error onto the status code that says what the
// operator can do about it. A malformed file is a conflict rather than a server
// error: nothing is broken here, the file on disk is simply not one chartr may
// overwrite until they fix it.
func promptStatus(err error) int {
	switch {
	case errors.Is(err, prompts.ErrNotFound):
		return http.StatusNotFound
	case errors.Is(err, prompts.ErrInvalid):
		return http.StatusBadRequest
	case errors.Is(err, prompts.ErrMalformed):
		return http.StatusConflict
	default:
		return http.StatusInternalServerError
	}
}

// promptBody is what a create and an edit both send: the two fields a preset has.
type promptBody struct {
	Name string `json:"name"`
	Body string `json:"body"`
}

func decodePrompt(w http.ResponseWriter, r *http.Request) (promptBody, bool) {
	var body promptBody
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return promptBody{}, false
	}
	return body, true
}

// handleCreatePrompt appends a preset to the catalog. The id comes back in the
// response because it is chartr's, derived from the name and stable from here
// on: an edit keeps it, and it is what a space's selection names.
func (s *Server) handleCreatePrompt(w http.ResponseWriter, r *http.Request) {
	body, ok := decodePrompt(w, r)
	if !ok {
		return
	}
	p, err := s.prompts.Create(body.Name, body.Body)
	if err != nil {
		httpError(w, promptStatus(err), err.Error())
		return
	}
	s.rebuild()
	writeJSON(w, http.StatusOK, map[string]any{"id": p.ID, "name": p.Name})
}

// handleUpdatePrompt rewrites a preset's name and text. The id is untouched, so
// every space that selected it keeps its selection and later launches simply
// receive the new text — which is the whole of what editing a preset means.
func (s *Server) handleUpdatePrompt(w http.ResponseWriter, r *http.Request) {
	body, ok := decodePrompt(w, r)
	if !ok {
		return
	}
	p, err := s.prompts.Update(r.PathValue("id"), body.Name, body.Body)
	if err != nil {
		httpError(w, promptStatus(err), err.Error())
		return
	}
	s.rebuild()
	writeJSON(w, http.StatusOK, map[string]any{"id": p.ID, "name": p.Name})
}

// handleDeletePrompt drops a preset and, in the same action, removes it from
// every space that had it selected — ordinary deletion cleans up its own
// references rather than leaving ids behind for a launch to puzzle over.
//
// The catalog is written first because it is the authority: if the registry
// write then fails the operator is told, and the orphaned id reads as a missing
// selection (surfaced on the space, never substituted) rather than as a preset
// that came back from the dead.
func (s *Server) handleDeletePrompt(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	if err := s.prompts.Delete(id); err != nil {
		httpError(w, promptStatus(err), err.Error())
		return
	}
	if err := s.reg.RemovePrompt(id); err != nil {
		s.rebuild()
		httpError(w, http.StatusInternalServerError, "clearing the deleted preset from spaces: "+err.Error())
		return
	}
	s.rebuild()
	writeJSON(w, http.StatusOK, map[string]any{"id": id, "deleted": true})
}

// handleSetSpacePrompts records which presets a space applies at launch: the
// whole selected list in one write. Every id must name a preset that exists —
// a list naming an unknown one is a stale client, and persisting it would store
// a selection the operator never made. Repo-scoped: Scratch has no launch
// selection in this first version, and the guard refuses it here.
func (s *Server) handleSetSpacePrompts(w http.ResponseWriter, r *http.Request) {
	e, ok := s.repoSpace(w, r)
	if !ok {
		return
	}
	var body struct {
		IDs []string `json:"ids"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	for _, id := range body.IDs {
		if _, held := s.prompts.Get(id); !held {
			httpError(w, http.StatusBadRequest, fmt.Sprintf("no preset called %q is in the catalog", id))
			return
		}
	}
	if err := s.reg.SetPrompts(e.ID, body.IDs); err != nil {
		httpError(w, http.StatusInternalServerError, err.Error())
		return
	}
	s.rebuild()
	w.WriteHeader(http.StatusNoContent)
}

// promptCatalog is the catalog on the wire, in creation order. Never nil.
func (s *Server) promptCatalog() []model.Prompt {
	out := []model.Prompt{}
	if s.prompts == nil {
		return out
	}
	for _, p := range s.prompts.List() {
		out = append(out, model.Prompt{ID: p.ID, Name: p.Name, Body: p.Body})
	}
	return out
}

// spacePrompts resolves one space's launch selection for the snapshot: the ids
// it selected, in catalog order, with a warning for each id the catalog no
// longer holds. The missing one is named and dropped rather than replaced —
// receiving a preset the operator did not choose is worse than receiving one
// fewer — and ordinary deletion has already cleaned its own references, so this
// only fires on a hand-edited registry or a half-finished delete.
func (s *Server) spacePrompts(selected []string) (ids []string, warnings []string) {
	ids = []string{}
	if s.prompts == nil {
		return ids, nil
	}
	chosen, missing := s.prompts.Selected(selected)
	for _, p := range chosen {
		ids = append(ids, p.ID)
	}
	for _, id := range missing {
		warnings = append(warnings, fmt.Sprintf(
			"the preset %q is selected here at launch but is no longer in the catalog; it is skipped", id))
	}
	return ids, warnings
}

// launchPrompts is what a launch in this space composes: the space's selected
// presets, in catalog order. It is the one resolution both launch paths use —
// the ticket payload (preview and spawn alike) and a free session's small run
// payload — so a preset applies the same way whichever way an agent starts.
//
// A selected id the catalog no longer holds is skipped here without a word: the
// snapshot already surfaces it on the space (spacePrompts), and a launch is the
// wrong moment to discover it.
func (s *Server) launchPrompts(e registry.Entry) []prompts.Prompt {
	if s.prompts == nil {
		return nil
	}
	chosen, _ := s.prompts.Selected(e.Prompts)
	return chosen
}
