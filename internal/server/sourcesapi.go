package server

import (
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"time"

	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/model"
	"github.com/rengwu/chartr/internal/sources"
)

// The sources section's write half (ticket 10): the six actions the planning
// tickets designed — register a source, remove one, toggle enabled, reorder,
// refresh a git source, and restore a role binding to its default. Everything
// else about the source list is an open-the-file action on `sources.toml`
// through the existing named-layer route; there is never a second config store.
//
// Every action ends in a rebuild, so the row the operator just changed comes
// back over the control socket rather than being patched optimistically in the
// browser. The registry is the authority on what is legal — this file maps its
// errors onto status codes and does no validation of its own.

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
			Default:  st.Default,
			Status:   st.Status,
			Skills:   []string{},
			Warnings: st.Warnings,
		}
		if !st.Fetched.IsZero() {
			row.Fetched = st.Fetched.UTC().Format(time.RFC3339)
		}
		// The default row is "shipped with this build" until its first refresh
		// converts it into a real checkout; `Pinned` is the same test the refresh
		// itself branches on, so the row cannot claim one state and take the other.
		row.Seeded = st.Default && !sources.Pinned(st.Path)
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
// beside the default a restore would write and whether the ref resolves today.
// A deleted binding rides with an empty ref rather than being filled in: nothing
// refills one automatically (ticket 03), and the restore control below is its
// only recovery.
func (s *Server) roleBindingRows() []model.RoleBinding {
	bound := s.roleBindings()
	out := make([]model.RoleBinding, 0, len(config.Roles))
	for _, r := range config.Roles {
		ref := bound[r]
		row := model.RoleBinding{
			Role:    string(r),
			Ref:     ref,
			Default: sources.DefaultBinding(string(r)),
		}
		if ref != "" && s.srcs != nil {
			_, err := s.srcs.Resolve(ref)
			row.Resolves = err == nil
		}
		out = append(out, row)
	}
	return out
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
		errors.Is(err, sources.ErrProtected),
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
	writeJSON(w, http.StatusOK, map[string]any{
		"name": src.Name, "commit": src.Commit, "fetched": src.Fetched.UTC().Format(time.RFC3339),
	})
}

// handleRestoreRoleBinding writes a role's seeded binding back into `user.toml`.
// It is the only recovery for a deleted binding, and it is deliberately an
// operator action rather than a startup repair: a missing row is a legitimate
// way to make a role refuse until it is rebound (ticket 03), so chartr restores
// one only when asked. Rebinding a role to anything *other* than its default is
// an edit of `user.toml`, openable from the same section.
func (s *Server) handleRestoreRoleBinding(w http.ResponseWriter, r *http.Request) {
	name := r.PathValue("role")
	if !config.IsRole(name) {
		httpError(w, http.StatusBadRequest, fmt.Sprintf("unknown role %q", name))
		return
	}
	role := config.Role(name)
	ref := sources.DefaultBinding(name)

	path, existing, err := s.readUserConfig()
	if err != nil {
		httpError(w, http.StatusInternalServerError, "reading your config: "+err.Error())
		return
	}
	next, err := config.SetUserRole(existing, role, ref)
	if err != nil {
		httpError(w, http.StatusInternalServerError, "writing the role binding: "+err.Error())
		return
	}
	if err := writeFileAtomic(path, next); err != nil {
		httpError(w, http.StatusInternalServerError, "writing your config: "+err.Error())
		return
	}
	s.rebuild()
	writeJSON(w, http.StatusOK, map[string]any{"role": name, "ref": ref})
}
