package server

import (
	"encoding/json"
	"net/http"
	"os"
	"path/filepath"

	"github.com/rengwu/chartr/internal/adapter"
	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/mapscan"
	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/prompt"
	"github.com/rengwu/chartr/internal/registry"
)

// userConfigName is the operator's local, uncommitted config under the chartr
// state root, keyed by space path. It carries per-machine binding overrides.
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
	// The global skill library resolves with no repo in play, so it is the same
	// answer whether or not a space is registered — which is exactly what the
	// settings route's global scope needs.
	return model.Model{
		Spaces: spaces,
		Config: s.globalLayers(),
		Skills: s.resolvedSkills(""),
		Agents: agentLibrary(userTOML),
	}
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
			Role:         string(b.Role),
			Adapter:      b.Adapter,
			Args:         b.Args,
			Prompt:       b.Prompt,
			AdapterFrom:  string(b.AdapterFrom),
			ArgsFrom:     string(b.ArgsFrom),
			PromptFrom:   string(b.PromptFrom),
			Agent:        b.Agent,
			AgentMissing: b.AgentMissing,
			Present:      b.Present,
			Missing:      b.Missing,
		})
	}

	maps := mapscan.Discover(e.Path)

	// Ad-hoc shells and sessions are runtime state the manager owns, not derived
	// from disk; fold them in so a mapless space still shows its terminal tabs and a
	// reconnecting browser rediscovers the open shells (story 29).
	terminals := make([]model.Terminal, 0)
	for _, info := range s.terms.ForSpace(e.ID) {
		term := model.Terminal{
			ID:     info.ID,
			Title:  info.Title,
			Proc:   info.Proc,
			Status: info.Status,
			Alive:  info.Alive,
		}
		if info.Session != nil {
			term.Session = &model.Session{
				MapSlug:   info.Session.MapSlug,
				TicketNum: info.Session.TicketNum,
				Role:      info.Session.Role,
				Agent:     info.Session.Agent,
			}
			// The quiet hint is the server's call, not the terminal's: the sampler
			// reports raw silence, and here — where the role is in hand — that silence
			// becomes "quiet" only for an AFK role. An idle grilling session shows
			// nothing (spec, Sessions and adapters; stories 34–35).
			if info.Silent && config.RoleIsAFK(info.Session.Role) {
				term.Status = model.TerminalQuiet
			}
		}
		terminals = append(terminals, term)
	}

	// Fold in the skill library's own notices — a fork behind the shipped default
	// (story 23) — so a stale fork is surfaced on the space without the operator
	// opening every role in the preview.
	warnings := append([]string{}, res.Warnings...)
	warnings = append(warnings, prompt.LibraryWarnings(s.skillRoots(e.Path))...)

	return model.Space{
		ID:        e.ID,
		Name:      filepath.Base(e.Path),
		Path:      e.Path,
		Branch:    gitBranch(e.Path),
		Pinned:    e.Pinned,
		Dirty:     gitDirty(e.Path),
		Bindings:  bindings,
		Skills:    s.resolvedSkills(e.Path),
		Layers:    s.spaceLayers(e.Path),
		Maps:      maps,
		Terminals: terminals,
		Warnings:  warnings,
	}
}

// writeFileAtomic writes data to path via a temp file and rename, so a crash
// mid-write cannot leave the operator's committed config truncated. The committed
// config sits under `.chartr/`, which a space may not have yet, so the
// parent directory is created first.
func writeFileAtomic(path string, data []byte) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
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

// openerPlaceholder stands in for the read-this-file opener in the library's
// command preview. It is deliberately unmistakable for a real path: the preview
// answers "where does the opener go", not "what will it say".
const openerPlaceholder = "‹opener›"

// agentLibrary derives the operator's registered agent library for the snapshot.
// It is resolved from the user config alone — the library is global, the same
// answer for every space and for none at all — so the settings surface can list
// and edit agents before a single space is registered. Never nil on the wire.
func agentLibrary(userTOML []byte) []model.Agent {
	resolved, _ := config.ResolveAgents(userTOML, nil)
	out := make([]model.Agent, 0, len(resolved))
	for _, a := range resolved {
		// The command preview comes from the seam that builds the real argv, with a
		// placeholder for the opener, so the library never shows a command the spawn
		// path would not actually run.
		launch := adapter.Command(adapter.Spawn{
			Adapter: a.Adapter,
			Args:    a.Args,
			Prompt:  openerPlaceholder,
			Deliver: a.Prompt,
		})
		out = append(out, model.Agent{
			Name:     a.Name,
			Adapter:  a.Adapter,
			Args:     a.Args,
			Prompt:   a.Prompt,
			Delivery: adapter.DeliveryFor(a.Adapter, a.Prompt).String(),
			Command:  append([]string{launch.Name}, launch.Args...),
			Present:  a.Present,
			Missing:  a.Missing,
		})
	}
	return out
}
