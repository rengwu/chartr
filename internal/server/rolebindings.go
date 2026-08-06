package server

import (
	"fmt"

	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/sources"
)

// The role bindings' two server-side halves: seeding them once at startup, and
// reading them at every composition.

// seedRoleBindings writes the four default bindings into the operator's config on
// the first startup that finds no `[roles]` table at all. It runs after the
// default source has been reconciled, so what it points at is guaranteed to be on
// disk; a startup that finds a table — even a table one row short — writes
// nothing, because a deleted row is a legitimate way to make a role refuse until
// it is rebound, and refilling it would be exactly the auto-restore that stance
// rules out.
//
// The write is quiet, like every other first-run write in this effort: nothing
// reports it, and the operator finds it in their own file or in the settings
// surface whenever they next look.
func seedRoleBindings(configDir string) error {
	if configDir == "" {
		return nil
	}
	path, existing, err := readUserConfigAt(configDir)
	if err != nil {
		return fmt.Errorf("reading user config to seed the role bindings: %w", err)
	}
	next, wrote := config.SeedRoleBindings(existing, func(r config.Role) string {
		return sources.DefaultBinding(string(r))
	})
	if !wrote {
		return nil
	}
	if err := writeFileAtomic(path, next); err != nil {
		return fmt.Errorf("seeding the role bindings: %w", err)
	}
	return nil
}

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
