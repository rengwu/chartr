package server

import (
	"encoding/json"
	"errors"
	"net/http"
	"os"
	"path/filepath"

	"github.com/rengwu/chartr/internal/adapter"
	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/mapscan"
	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/registry"
)

// userConfigName is the operator's local, uncommitted config under the chartr
// config root. It carries the global agent library — the only execution config
// there is, and never committed.
const userConfigName = "user.toml"

// terminalConfigName is the operator's per-machine terminal customization, beside
// the agent library under the same config root. Never committed, never per space —
// the single source of truth for every terminal island's look and feel.
const terminalConfigName = "terminal.toml"

// notifyConfigName is the operator's machine-wide notification rule. It is a
// separate file because it controls server behaviour, not terminal customization
// or execution.
const notifyConfigName = "notify.toml"

// repoSpace resolves the space an action names in its path, and is the one
// doorstep for every action that needs a repository behind it. It answers
// not-found for an id the registry does not hold, and refuses the Scratch
// identity with a bad request: Scratch is the operator's home directory, and
// spawning, charting, releasing, configuring or deregistering it would act on
// `$HOME` (spec, "Repo-scoped actions refuse it at the server").
//
// The refusal is a server rule and not merely an absent button — the chrome does
// not offer these controls on Scratch, but a stale or hand-written client must
// not be able to reach past that. Deregistering is refused rather than silently
// ignored for the same reason: a no-op on an explicit destructive request tells
// the caller nothing.
//
// The three terminal handlers deliberately do *not* use it. Opening, closing and
// acknowledging a shell is the whole of what Scratch is for, so they keep the
// plain registry lookup.
func (s *Server) repoSpace(w http.ResponseWriter, r *http.Request) (registry.Entry, bool) {
	e, ok := s.reg.Get(r.PathValue("id"))
	if !ok {
		httpError(w, http.StatusNotFound, "no such space")
		return registry.Entry{}, false
	}
	if e.Scratch {
		httpError(w, http.StatusBadRequest,
			"the Scratch space holds ad-hoc shells only — it has no repository to act on")
		return registry.Entry{}, false
	}
	return e, true
}

// handleSyncSkills is the manual mirror refresh behind the cockpit's "Sync
// skills" control: it reconciles this space's `.chartr/skills` mirror against
// the enabled sources on demand — old skills pruned, current ones (re)written in
// place — the same reconcile the spawn barrier runs, brought forward so an
// ad-hoc session chartr did not spawn reads a fresh tree. It also reconciles
// this space's copy of the write contract at `.chartr/TRACKER-CONVENTION.md`
// alongside it, since both are repo-local generated files an agent sandboxed to
// this space depends on. repoSpace refuses Scratch (no repository to mirror
// into), and a server with no source registry reconciles the skills half to
// nothing rather than erroring.
func (s *Server) handleSyncSkills(w http.ResponseWriter, r *http.Request) {
	e, ok := s.repoSpace(w, r)
	if !ok {
		return
	}
	if err := s.ensureSkillsCurrent(e); err != nil {
		httpError(w, http.StatusInternalServerError, "syncing skills: "+err.Error())
		return
	}
	if err := s.ensureConventionsCurrent(e); err != nil {
		httpError(w, http.StatusInternalServerError, "syncing the write contract: "+err.Error())
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

// handleRegister registers a folder as a space. Registration is a plain HTTP
// action so its outcome surfaces in the response (ADR 0010), and the new space
// also lands in the pushed snapshot via rebuild.
//
// It is also the third trigger for the standing `CHARTR.md` (chartr-md, ticket
// 01, amended). A newly registered space would otherwise carry no document until
// the next restart or sources change, which is exactly the window an operator
// registers a folder and opens an agent in it. The reconcile runs *after*
// `Register`, so the newly registered folder is already known for the document to
// be written into.
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

	entry, err := s.reg.Register(body.Path)
	if err != nil {
		// A bad path is the operator's to fix; surface it as the response.
		httpError(w, http.StatusBadRequest, err.Error())
		return
	}
	// Seed the new space's skill mirror straight away so an agent opened in it
	// before any spawn still reads the enabled sources' skills. Best-effort: the
	// space is registered regardless, and the spawn barrier is the correctness
	// path — a mirror hiccup here must not fail the registration the operator
	// asked for. The mirror is gitignored, so this writes nothing a teammate sees.
	_ = s.ensureSkillsCurrent(entry)
	// Same seeding, same best-effort footing, for the space's copy of the write
	// contract — the file CHARTR.md's conventions sentence is about to point at.
	_ = s.ensureConventionsCurrent(entry)
	s.rebuild()
	s.reconcileChartrDoc()

	writeJSON(w, http.StatusOK, map[string]any{
		"id":   entry.ID,
		"path": entry.Path,
	})
}

// handleDeregister forgets a space — the registry entry and its local order and
// recency — and touches nothing in the repository (story 4). Scratch cannot be
// forgotten: it is refused here rather than no-op'd, so a client that asks gets
// an answer instead of a silence it would read as success.
func (s *Server) handleDeregister(w http.ResponseWriter, r *http.Request) {
	e, ok := s.repoSpace(w, r)
	if !ok {
		return
	}
	if err := s.reg.Deregister(e.ID); err != nil {
		httpError(w, http.StatusInternalServerError, err.Error())
		return
	}
	s.rebuild()
	w.WriteHeader(http.StatusNoContent)
}

// handleOpenSpace reveals a registered repository in the platform's folder
// browser. It resolves the path from the registry id and deliberately bypasses
// the editor preference used for config files: this action means "show folder".
func (s *Server) handleOpenSpace(w http.ResponseWriter, r *http.Request) {
	e, ok := s.repoSpace(w, r)
	if !ok {
		return
	}
	opener := osOpener()
	if opener == "" {
		httpError(w, http.StatusInternalServerError, "no folder opener is available on this platform")
		return
	}
	if err := start(opener, e.Path); err != nil {
		httpError(w, http.StatusInternalServerError, "opening space folder: "+err.Error())
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

// handleReorder applies the operator's sidebar arrangement: one endpoint taking
// the complete ordered list of space ids, never a per-row move. The whole list
// is what the drag and the keyboard both send, so there is one write path, the
// write is idempotent, and two drags in quick succession cannot interleave into a
// half-moved sidebar. A list that omits a registered space, names an unknown id,
// or repeats one is a client bug: it is refused with a `400` and changes nothing.
func (s *Server) handleReorder(w http.ResponseWriter, r *http.Request) {
	var body struct {
		IDs []string `json:"ids"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	if err := s.reg.Reorder(body.IDs); err != nil {
		if errors.Is(err, registry.ErrBadReorder) {
			httpError(w, http.StatusBadRequest, err.Error())
			return
		}
		httpError(w, http.StatusInternalServerError, err.Error())
		return
	}
	// A reorder is the one mutation that derives nothing new: the same spaces, in
	// a different sequence. Republishing the snapshot permuted is the whole of it,
	// and it skips a rebuild's per-space `git status`, `.plan/` walks and config
	// resolution — the work the operator was watching the sidebar wait for.
	// The registry's own list is what sets the order, not the posted ids: the
	// sidebar may legitimately omit Scratch, and the registry places it.
	if !s.hub.reorderSpaces(spaceIDs(s.reg.List())) {
		// The snapshot is not the set we just ordered — a rebuild raced us, or a
		// space appeared between the write and here. Derive it properly.
		s.rebuild()
	}
	w.WriteHeader(http.StatusNoContent)
}

// spaceIDs is the registry's sidebar order as a plain id sequence.
func spaceIDs(entries []registry.Entry) []string {
	ids := make([]string, 0, len(entries))
	for _, e := range entries {
		ids = append(ids, e.ID)
	}
	return ids
}

// handleSetSpaceAgent records the agent a space should spawn with, decoupled from
// actually spawning. The selector calls it the moment the operator picks, so the
// choice survives a page reload and the next snapshot reflects it as the space's
// remembered agent. The name is trusted the same way a spawn's is — the selector
// only offers registered agents, and an unresolvable name simply reads back as
// nothing remembered (chooseAgent), so no membership check is enforced here.
func (s *Server) handleSetSpaceAgent(w http.ResponseWriter, r *http.Request) {
	e, ok := s.repoSpace(w, r)
	if !ok {
		return
	}
	var body struct {
		Agent string `json:"agent"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	if body.Agent == "" {
		httpError(w, http.StatusBadRequest, "agent is required")
		return
	}
	if err := s.reg.SetLastAgent(e.ID, body.Agent); err != nil {
		httpError(w, http.StatusInternalServerError, err.Error())
		return
	}
	s.rebuild()
	w.WriteHeader(http.StatusNoContent)
}

// handleRenameSpace sets a space's display name, or clears the override when the
// body carries an empty name so the folder basename reads again. It is
// presentation only — the rename changes nothing but the label the sidebar shows,
// which is why it takes no confirmation and needs no place in the settings
// surface. Repo-scoped like the agent setter above: Scratch carries no persisted
// row to name, so the guard refuses it here rather than let a stale client try.
func (s *Server) handleRenameSpace(w http.ResponseWriter, r *http.Request) {
	e, ok := s.repoSpace(w, r)
	if !ok {
		return
	}
	var body struct {
		Name string `json:"name"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	if err := s.reg.Rename(e.ID, body.Name); err != nil {
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
		// Scratch follows the operator's home but must never make it a discovery
		// root. Its model is rebuilt from terminal state only.
		if e.Scratch {
			continue
		}
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
	userTOML, _ := os.ReadFile(filepath.Join(s.opts.ConfigDir, userConfigName))

	// Terminal customization is its own per-machine file beside the agent library,
	// resolved once (server Seam 1). A missing file falls back to the built-in
	// default baked into the binary (config.DefaultTerminalTOML), so a fresh
	// machine gets the default look rather than a bare terminal; an operator's own
	// file replaces it wholesale. The resolved prefs ride the snapshot globally;
	// the parse warnings surface on each space beside the agent-library warnings,
	// through the same config-warnings surface (spec, Warnings surface).
	termTOML, _ := os.ReadFile(filepath.Join(s.opts.ConfigDir, terminalConfigName))
	if len(termTOML) == 0 {
		termTOML = config.DefaultTerminalTOML
	}
	termPrefs, termWarnings := config.ResolveTerminalPrefs(termTOML)

	// Session notification timing is another per-machine file, read and resolved
	// on every rebuild. Missing and malformed files both leave a usable machine;
	// only malformed values add warnings to the same surface as terminal.toml.
	notifyTOML, _ := os.ReadFile(filepath.Join(s.opts.ConfigDir, notifyConfigName))
	notifyPrefs, notifyWarnings := config.ResolveNotifyPrefs(notifyTOML)
	s.terms.ConfigureNotifications(
		notifyPrefs.Enabled, notifyPrefs.After, notifyPrefs.Settle)

	configWarnings := append([]string{}, termWarnings...)
	configWarnings = append(configWarnings, notifyWarnings...)
	spaces := make([]model.Space, 0, len(entries))
	for _, e := range entries {
		spaces = append(spaces, s.deriveSpace(e, userTOML, configWarnings))
	}
	// The global skill library resolves with no repo in play, so it is the same
	// answer whether or not a space is registered — which is exactly what the
	// settings route's global scope needs.
	// Whether a native folder chooser exists is a property of the machine, not of
	// the registry, so it is resolved here on the same lookup the picker itself
	// does rather than cached — it is a $PATH check, and the answer must not go
	// stale if the operator installs zenity mid-run.
	// The prompt only matters once the chooser is actually raised; the
	// availability check builds the same command either way, so any string does.
	_, nativePicker := nativePicker(pickStartDir(), defaultPickPrompt)

	return model.Model{
		Spaces: spaces,
		Config: s.globalLayers(),
		Agents: agentLibrary(userTOML),
		// The known-CLI probe is a $PATH check, like the folder chooser above: it is
		// a property of the machine, resolved fresh on every rebuild so a newly
		// installed harness shows up without a restart. Never nil for the wire.
		Detected: detectedAgents(),
		// The source list is walked fresh here for exactly the reason the agent
		// probe is: it is a read of the filesystem, and a snapshot that cached it
		// would show a stale skill count the moment a source changed underneath.
		Sources: s.sourceStates(),
		Roles:   s.roleBindingRows(),
		// Whether a git source can be registered at all. Same shape as the two
		// probes above — a PATH check, resolved per rebuild, never cached.
		GitAvailable: config.LookPath("git"),
		Terminal:     modelTerminalPrefs(termPrefs),
		Notify: model.NotifyPrefs{
			After:   notifyPrefs.After.String(),
			Settle:  notifyPrefs.Settle.String(),
			Enabled: notifyPrefs.Enabled,
		},
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

func (s *Server) deriveSpace(e registry.Entry, userTOML []byte, configWarnings []string) model.Space {
	terminals := s.modelTerminals(e.ID)
	if e.Scratch {
		return model.Space{
			ID:        e.ID,
			Name:      "Scratch",
			Scratch:   true,
			Path:      e.Path,
			Maps:      []model.Map{},
			Terminals: terminals,
		}
	}

	// The agent library is global, but its live warnings — an agent with no
	// adapter, an unreadable prompt delivery — surface where the operator is, so
	// they ride each space's warnings the way the binding resolver's used to.
	_, agentWarnings := config.ResolveAgents(userTOML, nil)

	maps := mapscan.Discover(e.Path)

	warnings := append([]string{}, agentWarnings...)
	// The per-machine config parse warnings are global, but they surface where the
	// operator is, the way the agent-library warnings do.
	warnings = append(warnings, configWarnings...)

	// The folder basename is the default label; the operator's own rename, when
	// they have set one, stands in for it. Purely presentation — the path and id
	// below are untouched, so every action still resolves by them.
	name := filepath.Base(e.Path)
	if e.Name != "" {
		name = e.Name
	}

	return model.Space{
		ID:        e.ID,
		Name:      name,
		Path:      e.Path,
		LastAgent: e.LastAgent,
		Maps:      maps,
		Terminals: terminals,
		Warnings:  warnings,
	}
}

// modelTerminals is the runtime-only half shared by registered and Scratch
// spaces. It performs no filesystem reads; the terminal manager already groups
// every tab by the registry identity it was opened against.
func (s *Server) modelTerminals(spaceID string) []model.Terminal {
	terminals := make([]model.Terminal, 0)
	for _, info := range s.terms.ForSpace(spaceID) {
		term := model.Terminal{
			ID:     info.ID,
			Title:  info.Title,
			Proc:   info.Proc,
			Status: info.Status,
			Alive:  info.Alive,
			// The dot rides the snapshot beside the tab's status, which is what makes it
			// survive a browser reload: the run it records may have ended with no browser
			// attached at all.
			FinishedUnseen: info.FinishedUnseen,
		}
		if info.Session != nil {
			term.Session = &model.Session{
				MapSlug:   info.Session.MapSlug,
				TicketNum: info.Session.TicketNum,
				Role:      info.Session.Role,
				Agent:     info.Session.Agent,
			}
		}
		terminals = append(terminals, term)
	}
	return terminals
}

// writeFileAtomic writes data to path via a temp file and rename, so a crash
// mid-write cannot leave the operator's config truncated. Every caller writes
// under the config root — the agent library and the terminal and notification
// files — which a machine may not have yet, so the parent directory is created
// first. That config is chartr's own state and can carry an agent's environment,
// so it is written owner-only, and the rename carries the temp file's mode onto
// the destination rather than leaving an older 0644 in place (ticket 05).
func writeFileAtomic(path string, data []byte) error {
	if err := os.MkdirAll(filepath.Dir(path), ownerDirMode); err != nil {
		return err
	}
	tmp := path + ".tmp"
	if err := writeOwnerFile(tmp, data); err != nil {
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

// codeLiveSession tags the one refusal the operator is allowed to overrule: a
// space that already has a live session (ADR 0003 as amended). The surface needs
// to tell it apart from every other 409 — a held ticket, a missing agent — which
// are refusals of fact and no confirmation can turn into a spawn. Only that
// distinction is machine-readable; the message stays the human's copy.
const codeLiveSession = "live_session_exists"

// httpErrorCode is httpError plus a stable code the surface can branch on.
func httpErrorCode(w http.ResponseWriter, status int, code, msg string) {
	writeJSON(w, status, map[string]string{"error": msg, "code": code})
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
		// The environment leads the preview the way it leads a shell line, and reads
		// resolved: an operator who typed `~/.claude2` sees the absolute path their
		// agent will actually be given, which is the only place that expansion is
		// visible before a launch.
		command := append(append([]string{}, a.LaunchEnv...), launch.Name)
		out = append(out, model.Agent{
			Name:     a.Name,
			Adapter:  a.Adapter,
			Args:     a.Args,
			Env:      a.Env,
			Prompt:   a.Prompt,
			Delivery: adapter.DeliveryFor(a.Adapter, a.Prompt).String(),
			Command:  append(command, launch.Args...),
			Present:  a.Present,
			Missing:  a.Missing,
		})
	}
	return out
}
