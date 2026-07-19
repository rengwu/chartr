package server

import (
	"encoding/json"
	"net/http"
	"os"
	"path/filepath"

	"github.com/rengwu/wayfinder-harness/internal/config"
	"github.com/rengwu/wayfinder-harness/internal/model"
	"github.com/rengwu/wayfinder-harness/internal/registry"
)

// workspaceConfigName is the committed workspace config file in a space's repo
// (ADR 0009): the shared, versioned, portable layer holding role bindings (and,
// from ticket 04, map kinds). Absent is the common case and never an error.
const workspaceConfigName = ".wayfinder-harness.toml"

// userConfigName is the operator's local, uncommitted config under the harness
// state root, keyed by space path. It carries per-machine binding overrides and
// the local-only autopilot flag.
const userConfigName = "user.toml"

// handleRegister registers a folder as a space. Registration is a plain HTTP
// action so its outcome — including an announced `git init` — surfaces in the
// response (ADR 0010, story 2), and the new space also lands in the pushed
// snapshot via rebuild.
func (s *Server) handleRegister(w http.ResponseWriter, r *http.Request) {
	var body struct {
		Path string `json:"path"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	if body.Path == "" {
		httpError(w, http.StatusBadRequest, "path is required")
		return
	}

	entry, gitInited, err := s.reg.Register(body.Path)
	if err != nil {
		// A bad path is the operator's to fix; surface it as the response.
		httpError(w, http.StatusBadRequest, err.Error())
		return
	}
	s.rebuild()

	writeJSON(w, http.StatusOK, map[string]any{
		"id":        entry.ID,
		"path":      entry.Path,
		"gitInited": gitInited,
	})
}

// handleDeregister forgets a space — the registry entry and its local pin and
// recency — and touches nothing in the repository (story 4).
func (s *Server) handleDeregister(w http.ResponseWriter, r *http.Request) {
	if err := s.reg.Deregister(r.PathValue("id")); err != nil {
		httpError(w, http.StatusInternalServerError, err.Error())
		return
	}
	s.rebuild()
	w.WriteHeader(http.StatusNoContent)
}

// handlePin sets whether a space is pinned, which reorders the sidebar
// (pinned-first) on the next snapshot.
func (s *Server) handlePin(w http.ResponseWriter, r *http.Request) {
	var body struct {
		Pinned bool `json:"pinned"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	if err := s.reg.SetPin(r.PathValue("id"), body.Pinned); err != nil {
		httpError(w, http.StatusInternalServerError, err.Error())
		return
	}
	s.rebuild()
	w.WriteHeader(http.StatusNoContent)
}

// rebuild recomputes the whole derived model from the registry and current
// config on disk, and pushes it to every browser. It is the one place the
// registry slice of the model is published, called after every mutating action
// and once at startup.
func (s *Server) rebuild() {
	s.hub.setModel(s.buildModel())
}

// buildModel derives the model from the registry: each registered space in
// sidebar order, with its role bindings resolved fresh across the three config
// layers. Config is read from disk on every rebuild — ticket 02 has no config
// watch, so the effective bindings reflect the files as of the last action.
func (s *Server) buildModel() model.Model {
	// The user layer is one file for all spaces, keyed by space path; read it
	// once. A missing or unreadable file resolves as "no local overrides".
	userTOML, _ := os.ReadFile(filepath.Join(s.opts.DataDir, userConfigName))

	entries := s.reg.List()
	spaces := make([]model.Space, 0, len(entries))
	for _, e := range entries {
		spaces = append(spaces, s.deriveSpace(e, userTOML))
	}
	return model.Model{Spaces: spaces}
}

func (s *Server) deriveSpace(e registry.Entry, userTOML []byte) model.Space {
	workspaceTOML, _ := os.ReadFile(filepath.Join(e.Path, workspaceConfigName))

	res := config.Resolve(config.Input{
		WorkspaceTOML: workspaceTOML,
		UserTOML:      userTOML,
		SpacePath:     e.Path,
	})

	bindings := make([]model.RoleBinding, 0, len(res.Bindings))
	for _, b := range res.Bindings {
		bindings = append(bindings, model.RoleBinding{
			Role:        string(b.Role),
			Adapter:     b.Adapter,
			Model:       b.Model,
			Args:        b.Args,
			AdapterFrom: string(b.AdapterFrom),
			ModelFrom:   string(b.ModelFrom),
			ArgsFrom:    string(b.ArgsFrom),
			Present:     b.Present,
			Missing:     b.Missing,
		})
	}

	return model.Space{
		ID:       e.ID,
		Name:     filepath.Base(e.Path),
		Path:     e.Path,
		Pinned:   e.Pinned,
		Bindings: bindings,
		Warnings: res.Warnings,
	}
}

func writeJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(v)
}

func httpError(w http.ResponseWriter, status int, msg string) {
	writeJSON(w, status, map[string]string{"error": msg})
}
