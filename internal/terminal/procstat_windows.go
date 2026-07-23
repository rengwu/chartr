//go:build windows

package terminal

import "github.com/aymanbagabas/go-pty"

// ConPTY exposes no foreground-process-group notion the way a Unix TTY does, so
// the sampler treats every live Windows shell as idle under its own title. The
// sidebar still shows the shell and its liveness; the working/idle refinement is
// a Unix affordance.
func foreground(p pty.Pty) int { return 0 }

func procName(pid int) string { return "" }

// With no foreground-process-group notion there is no group to enumerate, so
// agent identification finds nothing and every Windows tab reads the shell
// grammar — the same Unix-only refinement foreground/procName already are.
func procGroupNames(pgid int) []string { return nil }
