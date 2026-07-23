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
// state root. It carries the global agent library — the only execution config
// there is, and never committed.
const userConfigName = "user.toml"

// terminalConfigName is the operator's per-machine terminal customization, beside
// the agent library under the same state root. Never committed, never per space —
// the single source of truth for every terminal island's look and feel.
const terminalConfigName = "terminal.toml"

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
// sidebar order): each space with its maps discovered live from `.plan/`, and the
// operator's global agent library resolved once. Config and maps are read from
// disk on every rebuild, so the snapshot reflects the files as of this moment.
func (s *Server) buildModelFor(entries []registry.Entry) model.Model {
	// The operator's config is one file, read once. It carries the global agent
	// library; a missing or unreadable file resolves as "no agents registered".
	userTOML, _ := os.ReadFile(filepath.Join(s.opts.DataDir, userConfigName))

	// Terminal customization is its own per-machine file beside the agent library,
	// resolved once (server Seam 1). A missing file is all defaults and no
	// warnings — today's look, unchanged. The resolved prefs ride the snapshot
	// globally; the parse warnings surface on each space beside the agent-library
	// warnings, through the same config-warnings surface (spec, Warnings surface).
	termTOML, _ := os.ReadFile(filepath.Join(s.opts.DataDir, terminalConfigName))
	termPrefs, termWarnings := config.ResolveTerminalPrefs(termTOML)

	spaces := make([]model.Space, 0, len(entries))
	for _, e := range entries {
		spaces = append(spaces, s.deriveSpace(e, userTOML, termWarnings))
	}
	// The global skill library resolves with no repo in play, so it is the same
	// answer whether or not a space is registered — which is exactly what the
	// settings route's global scope needs.
	// Whether a native folder chooser exists is a property of the machine, not of
	// the registry, so it is resolved here on the same lookup the picker itself
	// does rather than cached — it is a $PATH check, and the answer must not go
	// stale if the operator installs zenity mid-run.
	_, nativePicker := nativePicker(pickStartDir())

	return model.Model{
		Spaces: spaces,
		Config: s.globalLayers(),
		Skills: s.resolvedSkills(""),
		Agents: agentLibrary(userTOML),
		// The known-CLI probe is a $PATH check, like the folder chooser above: it is
		// a property of the machine, resolved fresh on every rebuild so a newly
		// installed harness shows up without a restart. Never nil for the wire.
		Detected:     detectedAgents(),
		Terminal:     modelTerminalPrefs(termPrefs),
		NativePicker: nativePicker,
	}
}

// modelTerminalPrefs converts the resolved config prefs into their wire mirror.
// The two structs are field-for-field identical by design (the mirror keeps the
// model package off the config package); this is the one place the fields are
// copied, so a widened prefs set touches only here.
func modelTerminalPrefs(p config.TerminalPrefs) model.TerminalPrefs {
	return model.TerminalPrefs{
		FontFamily:     p.FontFamily,
		FontSize:       p.FontSize,
		FontWeight:     p.FontWeight,
		FontWeightBold: p.FontWeightBold,
		LineHeight:     p.LineHeight,
		LetterSpacing:  p.LetterSpacing,
		Ligatures:      p.Ligatures,

		CursorStyle:         p.CursorStyle,
		CursorBlink:         p.CursorBlink,
		CursorInactiveStyle: p.CursorInactiveStyle,
		CursorWidth:         p.CursorWidth,

		Scrollback:            p.Scrollback,
		ScrollSensitivity:     p.ScrollSensitivity,
		FastScrollModifier:    p.FastScrollModifier,
		FastScrollSensitivity: p.FastScrollSensitivity,
		SmoothScrollDuration:  p.SmoothScrollDuration,

		MinimumContrastRatio: p.MinimumContrastRatio,

		Unicode11: p.Unicode11,

		ScrollbarWidth:    p.ScrollbarWidth,
		ScrollbarThumb:    p.ScrollbarThumb,
		ScrollbarTrack:    p.ScrollbarTrack,
		ScrollbarAutoHide: p.ScrollbarAutoHide,

		PaddingTop:    p.PaddingTop,
		PaddingRight:  p.PaddingRight,
		PaddingBottom: p.PaddingBottom,
		PaddingLeft:   p.PaddingLeft,

		ShiftEnterNewline:     p.ShiftEnterNewline,
		CopyOnSelect:          p.CopyOnSelect,
		RightClickSelectsWord: p.RightClickSelectsWord,
		MacOptionIsMeta:       p.MacOptionIsMeta,

		Preset: p.Preset,

		Background:   p.Background,
		Foreground:   p.Foreground,
		Cursor:       p.Cursor,
		CursorAccent: p.CursorAccent,
		Selection:    p.Selection,

		Black:   p.Black,
		Red:     p.Red,
		Green:   p.Green,
		Yellow:  p.Yellow,
		Blue:    p.Blue,
		Magenta: p.Magenta,
		Cyan:    p.Cyan,
		White:   p.White,

		BrightBlack:   p.BrightBlack,
		BrightRed:     p.BrightRed,
		BrightGreen:   p.BrightGreen,
		BrightYellow:  p.BrightYellow,
		BrightBlue:    p.BrightBlue,
		BrightMagenta: p.BrightMagenta,
		BrightCyan:    p.BrightCyan,
		BrightWhite:   p.BrightWhite,
	}
}

func (s *Server) deriveSpace(e registry.Entry, userTOML []byte, termWarnings []string) model.Space {
	// The agent library is global, but its live warnings — an agent with no
	// adapter, an unreadable prompt delivery — surface where the operator is, so
	// they ride each space's warnings the way the binding resolver's used to.
	_, agentWarnings := config.ResolveAgents(userTOML, nil)

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
	// (story 23) — beside the agent-library warnings, so both live problems are
	// surfaced on the space without the operator opening settings.
	warnings := append([]string{}, agentWarnings...)
	warnings = append(warnings, prompt.LibraryWarnings(s.skillRoots(e.Path))...)
	// The terminal.toml parse warnings are global (the file is per-machine), but
	// they surface where the operator is, the way the agent-library warnings do.
	warnings = append(warnings, termWarnings...)

	return model.Space{
		ID:        e.ID,
		Name:      filepath.Base(e.Path),
		Path:      e.Path,
		Branch:    gitBranch(e.Path),
		Pinned:    e.Pinned,
		Dirty:     gitDirty(e.Path),
		LastAgent: e.LastAgent,
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

// detectedAgents probes the curated list of known agent CLIs against the current
// PATH, returning those present in curated order for the registration surface's
// helper text. It never nils: an empty slice is the honest "none of the ones I
// know are installed", which the surface renders as a generic example instead.
func detectedAgents() []string {
	found := config.DetectAgents(nil)
	if found == nil {
		return []string{}
	}
	return found
}

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
