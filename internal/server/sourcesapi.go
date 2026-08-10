package server

import (
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"time"

	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/sources"
)

// The sources section's write half (ticket 10): the five actions the planning
// tickets designed — register a source, remove one, toggle enabled, reorder, and
// refresh a git source. Everything else about the source list is an
// open-the-file action on `sources.toml` through the existing named-layer route;
// there is never a second config store.
//
// Every action ends in a rebuild, so the row the operator just changed comes
// back over the control socket rather than being patched optimistically in the
// browser. The registry is the authority on what is legal — this file maps its
// errors onto status codes and does no validation of its own.
//
// The five source mutations also reconcile the standing `CHARTR.md` in every
// registered space (chartr-md, ticket 01), because the sources block is half of
// what that document says. It is called explicitly rather than folded into
// rebuild(): rebuild also fires on terminal churn and space registration, and
// the trigger set for the standing document is deliberately narrower than that.

// sourceStates converts the registry's walked rows into their wire mirror. The
// walk is uncached, so this is a fresh read of every source root on every
// rebuild — the same freshness a spawn gets, which is what lets a skill folder
// created a second ago show up in the list without a restart.
func (s *Server) sourceStates() []model.Source {
	out := []model.Source{}
	if s.srcs == nil {
		return out
	}
	for _, st := range s.srcs.States() {
		row := model.Source{
			Name:     st.Name,
			Kind:     st.Kind,
			Path:     st.Path,
			URL:      st.URL,
			Ref:      st.Ref,
			Commit:   st.Commit,
			Enabled:  st.Enabled,
			Status:   st.Status,
			Skills:   []string{},
			Warnings: st.Warnings,
		}
		if !st.Fetched.IsZero() {
			row.Fetched = st.Fetched.UTC().Format(time.RFC3339)
		}
		for _, sk := range st.Skills {
			row.Skills = append(row.Skills, sk.Name)
			if sk.Shadowed {
				row.Shadowed = append(row.Shadowed, sk.Name)
			}
		}
		out = append(out, row)
	}
	return out
}

// roleBindingRows reports the four roles as they stand in `user.toml`, each
// beside whether its ref resolves today. An unbound role rides with an empty ref:
// chartr seeds none (ADR 0017 — it ships no skills), so a role is unbound until
// the operator binds it, and rebinding is an edit of `user.toml` from the same
// section.
func (s *Server) roleBindingRows() []model.RoleBinding {
	bound := s.roleBindings()
	out := make([]model.RoleBinding, 0, len(config.Roles))
	for _, r := range config.Roles {
		ref := bound[r]
		row := model.RoleBinding{
			Role:       string(r),
			Ref:        ref,
			Candidates: []string{},
		}
		if s.srcs != nil {
			row.Candidates = roleCandidates(s.srcs, r)
			row.Resolved = roleResolvedRef(s.srcs, r, ref)
			row.Resolves = row.Resolved != ""
		}
		out = append(out, row)
	}
	return out
}

// roleCandidates lists every `Source/skill` an enabled source offers that the
// role accepts (its SkillAliases), in resolution order. Shadowed losers are kept
// — reaching one is the whole point of pinning against precedence — and a skill
// carried under the same qualified name by two rows is listed once.
func roleCandidates(reg *sources.Registry, role config.Role) []string {
	aliases := config.SkillAliases(role)
	if len(aliases) == 0 {
		return []string{}
	}
	out := []string{}
	seen := map[string]bool{}
	for _, src := range reg.List() {
		if !src.Enabled {
			continue
		}
		st, ok := reg.Walk(src.Name)
		if !ok {
			continue
		}
		for _, sk := range st.Skills {
			for _, n := range aliases {
				if !strings.EqualFold(sk.Name, n) {
					continue
				}
				ref := src.Name + "/" + sk.Name
				if !seen[ref] {
					seen[ref] = true
					out = append(out, ref)
				}
				break
			}
		}
	}
	return out
}

// roleResolvedRef reports the `Source/skill` a spawn would actually run for this
// role today, or "" when nothing resolves. "auto" takes the precedence winner —
// the first candidate — while a `Source/skill` ref is a pin that resolves exactly
// or not at all: no alias fallback, so a pin into a disabled or gone source reads
// as unresolved rather than silently borrowing another source's skill.
func roleResolvedRef(reg *sources.Registry, role config.Role, ref string) string {
	if ref == "" {
		return ""
	}
	if ref == config.RoleBindingAuto {
		if c := roleCandidates(reg, role); len(c) > 0 {
			return c[0]
		}
		return ""
	}
	if _, err := reg.Resolve(ref); err == nil {
		return ref
	}
	return ""
}

// sourceStatus maps a registry error onto the status code that says what the
// operator can do about it: a conflict for a name already taken, a bad request
// for anything they typed, and a plain 500 for a clone or a write that failed.
func sourceStatus(err error) int {
	switch {
	case errors.Is(err, sources.ErrNotFound):
		return http.StatusNotFound
	case errors.Is(err, sources.ErrDuplicateName):
		return http.StatusConflict
	case errors.Is(err, sources.ErrBadName),
		errors.Is(err, sources.ErrGitMissing),
		errors.Is(err, sources.ErrBadReorder):
		return http.StatusBadRequest
	default:
		return http.StatusInternalServerError
	}
}

// handleRegisterSource registers a folder or a git repository of skills. The
// kind is declared by the caller, never inferred from the shape of what they
// typed. A git registration with `git` absent from PATH is refused at the gate,
// naming why, before any row is written — the registry does that check itself,
// so this handler only surfaces it.
func (s *Server) handleRegisterSource(w http.ResponseWriter, r *http.Request) {
	var body struct {
		Name string `json:"name"`
		Kind string `json:"kind"`
		Path string `json:"path"`
		URL  string `json:"url"`
		Ref  string `json:"ref"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}

	var (
		src sources.Source
		err error
	)
	switch body.Kind {
	case sources.KindDir:
		src, err = s.srcs.RegisterDir(body.Name, body.Path)
	case sources.KindGit:
		// A clone is not instant; it is also not cancellable, so the request simply
		// waits on it and the row appears only once the checkout is in place.
		src, err = s.srcs.RegisterGit(body.Name, body.URL, body.Ref)
	default:
		httpError(w, http.StatusBadRequest, fmt.Sprintf("a source is %q or %q, not %q",
			sources.KindDir, sources.KindGit, body.Kind))
		return
	}
	if err != nil {
		httpError(w, sourceStatus(err), err.Error())
		return
	}
	s.rebuild()
	s.reconcileChartrDoc()
	writeJSON(w, http.StatusOK, map[string]any{"name": src.Name, "path": src.Path})
}

// handleRemoveSource drops a row. It never touches what the row pointed at: a
// `dir` source is the operator's own folder, and a git checkout under `sources/`
// is left where it is — chartr does not collect them, which is why the section
// says so beside the file name.
func (s *Server) handleRemoveSource(w http.ResponseWriter, r *http.Request) {
	name := r.PathValue("name")
	if err := s.srcs.Remove(name); err != nil {
		httpError(w, sourceStatus(err), err.Error())
		return
	}
	s.rebuild()
	s.reconcileChartrDoc()
	writeJSON(w, http.StatusOK, map[string]any{"name": name, "removed": true})
}

// handleSetSourceEnabled toggles a source off without losing it — the reversible
// half of remove, and the one that keeps a row's position while it contributes
// nothing. The default row toggles like any other; only its removal and its
// position are protected.
func (s *Server) handleSetSourceEnabled(w http.ResponseWriter, r *http.Request) {
	var body struct {
		Enabled bool `json:"enabled"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	name := r.PathValue("name")
	if err := s.srcs.SetEnabled(name, body.Enabled); err != nil {
		httpError(w, sourceStatus(err), err.Error())
		return
	}
	s.rebuild()
	s.reconcileChartrDoc()
	writeJSON(w, http.StatusOK, map[string]any{"name": name, "enabled": body.Enabled})
}

// handleReorderSources rewrites the list order, which *is* resolution order.
// The whole list is sent every time — the registry refuses anything that is not
// a permutation of what it holds, so a stale browser cannot half-apply a move.
func (s *Server) handleReorderSources(w http.ResponseWriter, r *http.Request) {
	var body struct {
		Names []string `json:"names"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	if err := s.srcs.Reorder(body.Names); err != nil {
		httpError(w, sourceStatus(err), err.Error())
		return
	}
	s.rebuild()
	s.reconcileChartrDoc()
	writeJSON(w, http.StatusOK, map[string]any{"reordered": true})
}

// handleRefreshSource moves a git source to the tip of its ref and records the
// new pin. It is explicit and destructive inside chartr's own checkout — the
// section says so on the row, because the alternative is an operator losing an
// edit they made in a directory they did not know was chartr's.
func (s *Server) handleRefreshSource(w http.ResponseWriter, r *http.Request) {
	src, err := s.srcs.Refresh(r.PathValue("name"))
	if err != nil {
		httpError(w, sourceStatus(err), err.Error())
		return
	}
	s.rebuild()
	s.reconcileChartrDoc()
	writeJSON(w, http.StatusOK, map[string]any{
		"name": src.Name, "commit": src.Commit, "fetched": src.Fetched.UTC().Format(time.RFC3339),
	})
}

