package server

import (
	"encoding/json"
	"net/http"
	"os"
	"path/filepath"

	"github.com/rengwu/chartr/internal/config"
)

// The agent library's write surface. Registering an agent is the operator saying
// "this is a way I am willing to run a harness on this machine" — a binary,
// whatever flags that harness takes, and how it wants its opening prompt. One
// library serves every space, and a spawn picks from it at the gate.
//
// Both routes write **only the operator's own config** and never a repository's
// committed one, which is what keeps a permission-skipping agent something you
// grant yourself rather than something a `git pull` can hand you. Both are
// followed by the same rebuild every other edit triggers, so the new library
// reflects back over the control socket with no optimistic client state.

// handleSetAgent registers or updates one named agent. It is a PUT because the
// body is the agent's whole spec: what is sent is what the agent becomes, so a
// dropped flag is dropped rather than merged back in.
func (s *Server) handleSetAgent(w http.ResponseWriter, r *http.Request) {
	var body struct {
		Adapter string   `json:"adapter"`
		Args    []string `json:"args"`
		Prompt  string   `json:"prompt"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}

	agent := config.Agent{Adapter: body.Adapter, Args: body.Args, Prompt: body.Prompt}
	path, existing, err := s.readUserConfig()
	if err != nil {
		httpError(w, http.StatusInternalServerError, "reading user config: "+err.Error())
		return
	}
	next, err := config.SetUserAgent(existing, r.PathValue("name"), agent)
	if err != nil {
		// A bad name, no adapter, an unreadable delivery, or a TOML shape the editor
		// will not rewrite — every one the operator's to fix, never guessed at.
		httpError(w, http.StatusBadRequest, err.Error())
		return
	}
	if err := writeFileAtomic(path, next); err != nil {
		httpError(w, http.StatusInternalServerError, "writing user config: "+err.Error())
		return
	}
	s.rebuild()

	writeJSON(w, http.StatusOK, map[string]any{"name": r.PathValue("name"), "path": path})
}

// handleDeleteAgent removes one agent from the library, and touches nothing else.
// A space that last spawned with it simply reads nothing remembered on its next
// snapshot and reopens the picker — there is no assignment to strand.
func (s *Server) handleDeleteAgent(w http.ResponseWriter, r *http.Request) {
	name := r.PathValue("name")
	path, existing, err := s.readUserConfig()
	if err != nil {
		httpError(w, http.StatusInternalServerError, "reading user config: "+err.Error())
		return
	}
	next, err := config.DeleteUserAgent(existing, name)
	if err != nil {
		httpError(w, http.StatusBadRequest, err.Error())
		return
	}
	if err := writeFileAtomic(path, next); err != nil {
		httpError(w, http.StatusInternalServerError, "writing user config: "+err.Error())
		return
	}
	s.rebuild()

	writeJSON(w, http.StatusOK, map[string]any{"name": name, "path": path})
}

// readUserConfig returns the operator's config path and its current bytes, with
// an absent file reading as empty — the library's writers all build on "what is
// there now", and a first registration on a machine with no config yet is the
// ordinary case, not an error.
func (s *Server) readUserConfig() (string, []byte, error) {
	path := filepath.Join(s.opts.DataDir, userConfigName)
	b, err := os.ReadFile(path)
	if err != nil && !os.IsNotExist(err) {
		return path, nil, err
	}
	return path, b, nil
}
