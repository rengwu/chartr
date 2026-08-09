package server

import (
	"github.com/rengwu/chartr/internal/config"
)

// The role bindings' server-side half: reading them at every composition. chartr
// no longer seeds any (ADR 0017 — it ships no skills, so there is no default
// source to bind to); a role stays unbound, and refuses its spawn, until the
// operator binds it against a source they registered.

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
