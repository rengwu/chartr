package server

import (
	"encoding/json"
	"net/http"

	"github.com/rengwu/chartr/internal/config"
)

// The role bindings' server-side half: reading them at every composition, and the
// one write the settings picker offers. chartr no longer seeds any (ADR 0017 — it
// ships no skills, so there is no default source to bind to); a role stays
// unbound, and refuses its spawn, until the operator binds it against a source
// they registered — or picks "no preference", which writes the `auto` sentinel
// that resolves by source precedence.

// roleBindings reads the `[roles]` table fresh off disk, so an operator's hand
// edit reaches the very next spawn without a restart — the same freshness every
// other read of `user.toml` has. An unreadable file yields no bindings, which
// refuses the spawn with the role named rather than composing with a guess.
func (s *Server) roleBindings() config.RoleBindings {
	_, existing, err := s.readUserConfig()
	if err != nil {
		return config.RoleBindings{}
	}
	b, _ := config.ReadRoleBindings(existing)
	return b
}

// handleSetRoleBinding binds one role to a `Source/skill` ref or to the `auto`
// sentinel, writing into the operator's own `user.toml` — never a repository's
// committed config, the same boundary the agent library holds. It is the picker's
// only write: "no preference" sends `auto`, and choosing a listed skill sends its
// qualified ref. Like every other config edit it ends in a rebuild, so the row
// reflects back over the control socket with no optimistic client state.
func (s *Server) handleSetRoleBinding(w http.ResponseWriter, r *http.Request) {
	role := r.PathValue("role")
	if !config.IsRole(role) {
		httpError(w, http.StatusBadRequest, "no role named "+role)
		return
	}
	var body struct {
		Ref string `json:"ref"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		httpError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	path, existing, err := s.readUserConfig()
	if err != nil {
		httpError(w, http.StatusInternalServerError, "reading user config: "+err.Error())
		return
	}
	next, err := config.SetUserRole(existing, config.Role(role), body.Ref)
	if err != nil {
		httpError(w, http.StatusBadRequest, err.Error())
		return
	}
	if err := writeFileAtomic(path, next); err != nil {
		httpError(w, http.StatusInternalServerError, "writing user config: "+err.Error())
		return
	}
	s.rebuild()

	writeJSON(w, http.StatusOK, map[string]any{"role": role, "ref": body.Ref})
}
