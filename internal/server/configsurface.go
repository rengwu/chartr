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

	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/prompt"
	"github.com/rengwu/chartr/internal/registry"
)

// The settings surface: the agent library (agents.go) and the paths of the files
// behind it, each openable in the operator's own editor. There is no committed
// execution layer and no per-field provenance to render — ADR 0014's effective
// config surface is superseded (ticket 05). What is left is a read of where each
// file lives and a named-layer open action.

// The layer names the client may ask about. A name is a token the server
// resolves to a path; the client never sends a path, so a local server can never
// be talked into opening an arbitrary file (story 45). `skill:<name>` reaches the
// winning directory of one resolved skill.
const (
	layerUserConfig      = "user-config"
	layerTerminalConfig  = "terminal-config"
	layerBuiltinSkills   = "builtin-skills"
	layerUserSkills      = "user-skills"
	layerWorkspaceSkills = "workspace-skills"
	layerSkillPrefix     = "skill:"
)

// globalLayers are the files the operator's config lives in, shared by every
// space: the agent library and the two skill libraries that are not a space's own.
// They are derived once per rebuild rather than repeated under each space.
//
// Every user-scoped layer lives under one roof — the operator's config root
// (`~/.config/chartr`): the agent library (`user.toml`), terminal customization
// (`terminal.toml`), the operator's own skills (`skills/`), and the materialized
// built-in library (`builtin-skills/`). The surface lists each as its own path so
// the operator can open exactly the file a layer resolves from.
//
// `terminal.toml` is per-machine terminal customization, read on every rebuild
// into the snapshot's resolved prefs. It is listed here so the Settings surface
// can open it in the operator's editor through exactly the same named-layer
// action — read-value-plus-open-file, never a second config store.
func (s *Server) globalLayers() []model.ConfigLayer {
	roots := prompt.RootsFor(s.opts.ConfigDir, "")
	return []model.ConfigLayer{
		layerAt(layerBuiltinSkills, "built-in", "skills", roots.Builtin),
		layerAt(layerUserConfig, "user", "agents", filepath.Join(s.opts.ConfigDir, userConfigName)),
		layerAt(layerTerminalConfig, "user", "terminal",
			filepath.Join(s.opts.ConfigDir, terminalConfigName)),
		layerAt(layerUserSkills, "user", "skills", roots.User),
	}
}

// spaceLayers are the layers a space carries in its own repository. Execution is
// no longer among them — there is no committed config file (ticket 05) — so what
// remains is the space's committed skill library (ADR 0009's content half).
func (s *Server) spaceLayers(repoDir string) []model.ConfigLayer {
	return []model.ConfigLayer{
		layerAt(layerWorkspaceSkills, "workspace", "skills",
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

// resolveGlobalLayerPath is the same resolution with no space in play: the
// layers every space shares, and skills resolved through the built-in and user
// roots alone. The global scope of the settings route is reachable before any
// space is registered, so its open action cannot be routed through a space id.
func (s *Server) resolveGlobalLayerPath(name string) (string, error) {
	if skill, ok := strings.CutPrefix(name, layerSkillPrefix); ok {
		return s.resolveSkillDir(skill, "")
	}
	for _, l := range s.globalLayers() {
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
	openResolved(w, path)
}

// handleOpenGlobalLayer is handleOpenLayer for the layers that are nobody's
// space: the operator's own config file and the two global skill libraries. Same
// named-layer resolution, same editor ladder — it just never needs a space.
func (s *Server) handleOpenGlobalLayer(w http.ResponseWriter, r *http.Request) {
	var body struct {
		Layer string `json:"layer"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}

	path, err := s.resolveGlobalLayerPath(body.Layer)
	if err != nil {
		httpError(w, http.StatusBadRequest, err.Error())
		return
	}
	openResolved(w, path)
}

// openResolved runs the editor ladder on an already-resolved path and reports how
// far it got. A layer with nothing on disk yet is reported as such with its path:
// the surface says where the value *would* go, and a read-shaped action creates
// nothing.
func openResolved(w http.ResponseWriter, path string) {
	if _, err := os.Stat(path); err != nil {
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
// chartr has no business waiting on it.
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
