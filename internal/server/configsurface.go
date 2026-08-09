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
	"github.com/rengwu/chartr/internal/sources"
)

// The settings surface: the agent library (agents.go) and the paths of the files
// behind it, each openable in the operator's own editor. There is no committed
// execution layer and no per-field provenance to render — ADR 0014's effective
// config surface is superseded (ticket 05). What is left is a read of where each
// file lives and a named-layer open action.

// The layer names the client may ask about. A name is a token the server
// resolves to a path; the client never sends a path, so a local server can never
// be talked into opening an arbitrary file (story 45).
const (
	layerUserConfig     = "user-config"
	layerTerminalConfig = "terminal-config"
	layerNotifyConfig   = "notify-config"
	layerSources        = "sources-config"
	layerPreferences    = "preferences"
)

// globalLayers are the files the operator's config lives in, shared by every
// space: the agent library and the terminal and notification prefs. They are
// derived once per rebuild rather than repeated under each space.
//
// Every one of them lives under one roof — the operator's config root
// (`~/.config/chartr`). The skill libraries are gone from this list with the
// layer model itself (ADR 0017): skills come from registered sources, whose
// paths the sources section owns.
//
// `terminal.toml` is per-machine terminal customization, read on every rebuild
// into the snapshot's resolved prefs. It is listed here so the Settings surface
// can open it in the operator's editor through exactly the same named-layer
// action — read-value-plus-open-file, never a second config store.
func (s *Server) globalLayers() []model.ConfigLayer {
	return []model.ConfigLayer{
		layerAt(layerUserConfig, "user", "agents", filepath.Join(s.opts.ConfigDir, userConfigName)),
		layerAt(layerTerminalConfig, "user", "terminal",
			filepath.Join(s.opts.ConfigDir, terminalConfigName)),
		layerAt(layerNotifyConfig, "user", "notifications",
			filepath.Join(s.opts.ConfigDir, notifyConfigName)),
		// The two files the sources section owns. `sources.toml` is edited through
		// that section's controls and openable here for everything it deliberately
		// does not edit; `preferences.md` is the operator's own and is the one of
		// the two they are expected to write in. The generated write contract used
		// to be a third file here (`conventions.md`, under this same config root);
		// it now lives per-space at `.chartr/TRACKER-CONVENTION.md` so a session
		// sandboxed to its own space can read it, which took it out of the
		// operator's config root and off this list — it has no single global path
		// left to name.
		layerAt(layerSources, "user", "sources", sources.FilePath(s.opts.ConfigDir)),
		layerAt(layerPreferences, "user", "preferences", prompt.PreferencesPath(s.opts.ConfigDir)),
	}
}

func layerAt(name, layer, holds, path string) model.ConfigLayer {
	_, err := os.Stat(path)
	return model.ConfigLayer{Name: name, Layer: layer, Holds: holds, Path: path, Exists: err == nil}
}

// resolveLayerPath turns a layer name into an absolute path, server-side. An
// unknown name is refused rather than treated as a path — that refusal is the
// whole security property of the open action.
func (s *Server) resolveLayerPath(name string, _ registry.Entry) (string, error) {
	return s.resolveGlobalLayerPath(name)
}

// resolveGlobalLayerPath is the same resolution with no space in play. Every
// layer left is the operator's own config file, so the two resolutions have
// collapsed into one; the space-scoped route survives because the client still
// asks through a space id. The global scope of the settings route is reachable
// before any space is registered, so its open action cannot be routed through
// one.
func (s *Server) resolveGlobalLayerPath(name string) (string, error) {
	for _, l := range s.globalLayers() {
		if l.Name == name {
			return l.Path, nil
		}
	}
	return "", fmt.Errorf("unknown config layer %q", name)
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
	e, ok := s.repoSpace(w, r)
	if !ok {
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

// layerTemplates maps a creatable layer name to the starter bytes written when
// the operator asks to scaffold it from defaults. Only layers with an entry here
// can be created through handleCreateGlobalLayer; a name with no template is
// refused, the same shape as an unknown layer. `terminal.toml` and `notify.toml`
// carry starters; the agent library and skill roots are grown by their own edits,
// not stamped from a defaults file.
var layerTemplates = map[string][]byte{
	layerTerminalConfig: config.ScaffoldTerminalTOML,
	layerNotifyConfig:   config.ScaffoldNotifyTOML,
}

// handleCreateGlobalLayer stamps a config file from its defaults template — the
// companion to the open action for a layer that does not exist yet. The open
// action deliberately creates nothing (a read-shaped action), so a file the
// operator has never written can only be opened once it exists; this is how it
// comes to exist. It writes **only the operator's own config**, never a
// repository's committed one, and only a layer that carries a bundled template.
//
// It never clobbers: a layer already on disk is refused with a conflict rather
// than overwritten, so the button can only ever create the file the surface says
// is missing and can never wipe an operator's real customization. On success it
// runs the same rebuild every other config edit triggers, so the freshly-created
// file — and the values it now puts in force — reflect back over the control
// socket with no optimistic client state.
func (s *Server) handleCreateGlobalLayer(w http.ResponseWriter, r *http.Request) {
	var body struct {
		Layer string `json:"layer"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}

	template, ok := layerTemplates[body.Layer]
	if !ok {
		// Either an unknown layer or one with nothing to stamp from — both are the
		// operator asking for something the server does not offer, refused the same way.
		httpError(w, http.StatusBadRequest, fmt.Sprintf("config layer %q cannot be created from a template", body.Layer))
		return
	}

	path, err := s.resolveGlobalLayerPath(body.Layer)
	if err != nil {
		httpError(w, http.StatusBadRequest, err.Error())
		return
	}

	if _, err := os.Stat(path); err == nil {
		httpError(w, http.StatusConflict, fmt.Sprintf("%s already exists; open it to edit", path))
		return
	} else if !os.IsNotExist(err) {
		httpError(w, http.StatusInternalServerError, "checking config path: "+err.Error())
		return
	}

	if err := writeFileAtomic(path, template); err != nil {
		httpError(w, http.StatusInternalServerError, "writing config: "+err.Error())
		return
	}
	s.rebuild()

	writeJSON(w, http.StatusOK, map[string]any{"path": path, "created": true})
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
