package server

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/prompt"
	"github.com/rengwu/chartr/internal/registry"
)

// The effective config surface (ticket 05, ADR 0014): the read half derives every
// participating layer with its path so the settings route can show where each
// value lives, and the write half edits exactly one thing — a role binding, into
// the user layer — and opens everything else in the operator's editor.

// The layer names the client may ask about. A name is a token the server
// resolves to a path; the client never sends a path, so a local server can never
// be talked into opening an arbitrary file (story 45). `skill:<name>` reaches the
// winning directory of one resolved skill.
const (
	layerUserConfig      = "user-config"
	layerBuiltinSkills   = "builtin-skills"
	layerUserSkills      = "user-skills"
	layerWorkspaceConfig = "workspace-config"
	layerWorkspaceSkills = "workspace-skills"
	layerSkillPrefix     = "skill:"
)

// globalLayers are the config layers every space resolves through: the
// operator's local binding overrides and the two skill libraries that are not a
// space's own. They are derived once per rebuild rather than repeated under each
// space.
//
// Note the split the surface has to tell honestly: bindings live in the chartr
// state root (`<dataDir>/user.toml`) while the *user skill layer* lives under the
// operator's config root (`<configDir>/skills/`). One "user layer" in ADR 0009's
// sense, two files, because the two halves were adopted a ticket apart — the
// surface shows both paths rather than implying a single file.
func (s *Server) globalLayers() []model.ConfigLayer {
	roots := prompt.RootsFor(s.opts.DataDir, s.opts.ConfigDir, "")
	return []model.ConfigLayer{
		layerAt(layerBuiltinSkills, string(config.LayerBuiltin), "skills", roots.Builtin),
		layerAt(layerUserConfig, string(config.LayerUser), "bindings", filepath.Join(s.opts.DataDir, userConfigName)),
		layerAt(layerUserSkills, string(config.LayerUser), "skills", roots.User),
	}
}

// spaceLayers are the layers a space carries in its own repository — committed,
// shared, and versioned (ADR 0009).
func (s *Server) spaceLayers(repoDir string) []model.ConfigLayer {
	return []model.ConfigLayer{
		layerAt(layerWorkspaceConfig, string(config.LayerWorkspace), "bindings",
			filepath.Join(repoDir, config.WorkspaceConfigName)),
		layerAt(layerWorkspaceSkills, string(config.LayerWorkspace), "skills",
			s.skillRoots(repoDir).Workspace),
	}
}

func layerAt(name, layer, holds, path string) model.ConfigLayer {
	_, err := os.Stat(path)
	return model.ConfigLayer{Name: name, Layer: layer, Holds: holds, Path: path, Exists: err == nil}
}

// resolveLayerPath turns a layer name into an absolute path, server-side. An
// unknown name is refused rather than treated as a path — that refusal is the
// whole security property of the open action.
func (s *Server) resolveLayerPath(name string, e registry.Entry) (string, error) {
	if skill, ok := strings.CutPrefix(name, layerSkillPrefix); ok {
		return s.resolveSkillDir(skill, e.Path)
	}
	for _, l := range append(s.globalLayers(), s.spaceLayers(e.Path)...) {
		if l.Name == name {
			return l.Path, nil
		}
	}
	return "", fmt.Errorf("unknown config layer %q", name)
}

// resolveSkillDir names the directory that actually won a skill, so "open the
// grill skill" opens the copy a session would read rather than a layer that lost.
// A skill resolving from the binary's embedded floor opens the materialized
// built-in directory, which is where an edit belongs.
func (s *Server) resolveSkillDir(name, repoDir string) (string, error) {
	for _, known := range prompt.Names() {
		if known != name {
			continue
		}
		if sk, ok := prompt.Resolve(name, s.skillRoots(repoDir)); ok && sk.Dir != "" {
			return sk.Dir, nil
		}
		return filepath.Join(s.skillRoots(repoDir).Builtin, name), nil
	}
	return "", fmt.Errorf("unknown skill %q", name)
}

// resolvedSkills is the space's whole skill library with the layer that won each
// directory — the positive "your grill resolves from: user" the surface shows
// beside the stale-fork warning (story 34).
func (s *Server) resolvedSkills(repoDir string) []model.ResolvedSkill {
	lib := prompt.Library(s.skillRoots(repoDir))
	out := make([]model.ResolvedSkill, 0, len(lib))
	for _, sk := range lib {
		out = append(out, model.ResolvedSkill{
			Name:        sk.Name,
			Layer:       sk.Layer,
			Dir:         sk.Dir,
			Description: sk.Description,
			ForkedFrom:  sk.ForkedFrom,
			Stale:       sk.Stale,
		})
	}
	return out
}

// handleSetBinding edits one field of one role binding — the one high-churn
// setting the surface edits inline (story 39). It writes **only** the user layer
// (ADR 0009 as amended): bindings resolve user-over-workspace, so that is their
// home, and a local UI never writes a space's committed config on a teammate's
// behalf. Clearing a field reveals the layer beneath it, so the edit is
// reversible rather than a one-way ratchet.
//
// The write is surgical and comment-preserving (config.SetUserBinding), and it is
// followed by the same rebuild the classify action triggers, so the new value and
// its new provenance reflect straight back over the control socket with no
// optimistic client state (story 43).
func (s *Server) handleSetBinding(w http.ResponseWriter, r *http.Request) {
	e, ok := s.reg.Get(r.PathValue("id"))
	if !ok {
		httpError(w, http.StatusNotFound, "no such space")
		return
	}

	var body struct {
		Role  string `json:"role"`
		Field string `json:"field"`
		// Value is the new value: a string for adapter and model, an array of
		// strings for args, and null to clear the override.
		Value json.RawMessage `json:"value"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}

	edit := config.BindingEdit{SpacePath: e.Path, Role: body.Role, Field: body.Field}
	switch {
	case len(body.Value) == 0 || string(body.Value) == "null":
		edit.Clear = true
	case body.Field == config.FieldArgs:
		if err := json.Unmarshal(body.Value, &edit.Args); err != nil {
			httpError(w, http.StatusBadRequest, "args must be a list of strings")
			return
		}
	default:
		if err := json.Unmarshal(body.Value, &edit.Value); err != nil {
			httpError(w, http.StatusBadRequest, "value must be a string")
			return
		}
	}

	path := filepath.Join(s.opts.DataDir, userConfigName)
	existing, err := os.ReadFile(path)
	if err != nil && !os.IsNotExist(err) {
		httpError(w, http.StatusInternalServerError, "reading user config: "+err.Error())
		return
	}
	next, err := config.SetUserBinding(existing, edit)
	if err != nil {
		// An unknown role or field, or a shape the editor will not rewrite — the
		// operator's to fix, never something to guess at.
		httpError(w, http.StatusBadRequest, err.Error())
		return
	}
	if err := writeFileAtomic(path, next); err != nil {
		httpError(w, http.StatusInternalServerError, "writing user config: "+err.Error())
		return
	}
	s.rebuild()

	writeJSON(w, http.StatusOK, map[string]any{
		"role": edit.Role, "field": edit.Field, "cleared": edit.Clear, "path": path,
	})
}

// handleOpenLayer opens a config layer in the operator's editor — the escape
// hatch for everything the surface deliberately does not edit inline (story 44).
// It resolves a *named* layer server-side and never a path from the client.
//
// The ladder is $VISUAL, then $EDITOR, then the OS opener, and finally the
// absolute path itself: a headless or misconfigured environment still tells the
// operator where the file is instead of failing silently. A layer that does not
// exist yet is reported as such with its path — the surface says where the value
// would go, and nothing is created on a read-shaped action.
func (s *Server) handleOpenLayer(w http.ResponseWriter, r *http.Request) {
	e, ok := s.reg.Get(r.PathValue("id"))
	if !ok {
		httpError(w, http.StatusNotFound, "no such space")
		return
	}

	var body struct {
		Layer string `json:"layer"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}

	path, err := s.resolveLayerPath(body.Layer, e)
	if err != nil {
		httpError(w, http.StatusBadRequest, err.Error())
		return
	}
	if _, statErr := os.Stat(path); statErr != nil {
		writeJSON(w, http.StatusOK, map[string]any{
			"path": path, "exists": false, "opened": "none",
		})
		return
	}

	opened, with := openInEditor(path)
	writeJSON(w, http.StatusOK, map[string]any{
		"path": path, "exists": true, "opened": opened, "with": with,
	})
}

// openInEditor launches the operator's editor on path, reporting how it got
// there. It never blocks on the child: an editor is the operator's to close, and
// the chartr has no business waiting on it.
func openInEditor(path string) (how, with string) {
	for _, env := range []string{"VISUAL", "EDITOR"} {
		cmd := strings.TrimSpace(os.Getenv(env))
		if cmd == "" {
			continue
		}
		// An editor variable may carry flags ("code -w"); split on whitespace, the
		// same shape every other tool reads it as.
		fields := strings.Fields(cmd)
		if err := start(fields[0], append(fields[1:], path)...); err == nil {
			return "editor", cmd
		}
	}
	if opener := osOpener(); opener != "" {
		if err := start(opener, path); err == nil {
			return "os", opener
		}
	}
	return "none", ""
}

func osOpener() string {
	switch runtime.GOOS {
	case "darwin":
		return "open"
	case "windows":
		return "explorer"
	default:
		return "xdg-open"
	}
}

func start(name string, args ...string) error {
	cmd := exec.Command(name, args...)
	if err := cmd.Start(); err != nil {
		return err
	}
	// Reap it in the background so a long-lived editor is not left a zombie.
	go func() { _ = cmd.Wait() }()
	return nil
}
