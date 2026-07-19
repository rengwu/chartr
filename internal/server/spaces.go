package server

import (
	"encoding/json"
	"net/http"
	"os"
	"path/filepath"

	"github.com/rengwu/wayfinder-harness/internal/config"
	"github.com/rengwu/wayfinder-harness/internal/mapscan"
	"github.com/rengwu/wayfinder-harness/internal/model"
	"github.com/rengwu/wayfinder-harness/internal/registry"
)

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

// handleClassify declares a map's kind (ADR 0007). Classification is one HTTP
// action: the operator confirms the convention guess, and the harness writes the
// kind into the space's committed workspace config, keyed by map slug, so the
// classification rides the repo to every teammate (story 15). The write appends
// to the config and is left in the working tree — the harness owns no commit here
// (its commits are the lifecycle writes of later tickets), the operator commits
// their config as they commit their bindings.
func (s *Server) handleClassify(w http.ResponseWriter, r *http.Request) {
	e, ok := s.reg.Get(r.PathValue("id"))
	if !ok {
		httpError(w, http.StatusNotFound, "no such space")
		return
	}
	slug := r.PathValue("slug")

	var body struct {
		Kind string `json:"kind"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	if !model.ValidKind(body.Kind) {
		httpError(w, http.StatusBadRequest, "kind must be planning or implementation")
		return
	}

	path := filepath.Join(e.Path, config.WorkspaceConfigName)
	existing, err := os.ReadFile(path)
	if err != nil && !os.IsNotExist(err) {
		httpError(w, http.StatusInternalServerError, "reading committed config: "+err.Error())
		return
	}
	next, err := config.DeclareMapKind(existing, slug, body.Kind)
	if err != nil {
		// An already-declared slug or an unknown kind is the operator's to fix.
		httpError(w, http.StatusBadRequest, err.Error())
		return
	}
	if err := writeFileAtomic(path, next); err != nil {
		httpError(w, http.StatusInternalServerError, "writing committed config: "+err.Error())
		return
	}
	s.rebuild()

	writeJSON(w, http.StatusOK, map[string]any{"slug": slug, "kind": body.Kind})
}

// rebuild recomputes the whole derived model from the registry and current
// config on disk, and pushes it to every browser. It is the one place the
// registry slice of the model is published, called after every mutating action,
// on every filesystem notice, and once at startup. It also reconciles the
// discovery watch set to the registry, so a newly registered space starts being
// watched and a forgotten one stops.
func (s *Server) rebuild() {
	entries := s.reg.List()
	roots := make([]string, 0, len(entries))
	for _, e := range entries {
		roots = append(roots, e.Path)
	}
	s.watch.setRoots(roots)
	s.hub.setModel(s.buildModelFor(entries))
}

// buildModelFor derives the model from the given registry entries (already in
// sidebar order): each space with its role bindings resolved fresh across the
// three config layers and its maps discovered live from `.plan/`. Config and
// maps are read from disk on every rebuild, so the snapshot reflects the files
// as of this moment.
func (s *Server) buildModelFor(entries []registry.Entry) model.Model {
	// The user layer is one file for all spaces, keyed by space path; read it
	// once. A missing or unreadable file resolves as "no local overrides".
	userTOML, _ := os.ReadFile(filepath.Join(s.opts.DataDir, userConfigName))

	spaces := make([]model.Space, 0, len(entries))
	for _, e := range entries {
		spaces = append(spaces, s.deriveSpace(e, userTOML))
	}
	return model.Model{Spaces: spaces}
}

func (s *Server) deriveSpace(e registry.Entry, userTOML []byte) model.Space {
	workspaceTOML, _ := os.ReadFile(filepath.Join(e.Path, config.WorkspaceConfigName))

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

	// Discovery derives every map with its convention guess; the committed
	// declaration overlays it, so a classified map goes inert-no-more and its
	// lingering guess is cleared. A declared kind whose slug matches no discovered
	// map simply dangles (a renamed directory), which is inert-and-fine, never an
	// error (ADR 0007).
	maps := mapscan.Discover(e.Path)
	for i := range maps {
		if kind, ok := res.Kinds[maps[i].Slug]; ok {
			maps[i].Kind = kind
			maps[i].KindGuess = ""
		}
	}

	return model.Space{
		ID:       e.ID,
		Name:     filepath.Base(e.Path),
		Path:     e.Path,
		Pinned:   e.Pinned,
		Bindings: bindings,
		Maps:     maps,
		Warnings: res.Warnings,
	}
}

// writeFileAtomic writes data to path via a temp file and rename, so a crash
// mid-write cannot leave the operator's committed config truncated. The parent
// directory is the space repo, which already exists.
func writeFileAtomic(path string, data []byte) error {
	tmp := path + ".tmp"
	if err := os.WriteFile(tmp, data, 0o644); err != nil {
		return err
	}
	if err := os.Rename(tmp, path); err != nil {
		_ = os.Remove(tmp)
		return err
	}
	return nil
}

func writeJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(v)
}

func httpError(w http.ResponseWriter, status int, msg string) {
	writeJSON(w, status, map[string]string{"error": msg})
}
