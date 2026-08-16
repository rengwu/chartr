//go:build !windows

package terminal

import (
	"os/exec"
	"strconv"
	"strings"

	"github.com/aymanbagabas/go-pty"
	"golang.org/x/sys/unix"

	"github.com/rengwu/chartr/internal/proc"
)

// foreground returns the process-group id currently in the foreground of the
// PTY — what tcgetpgrp(3) reports on the master end. It equals the shell's own
// pid while the shell sits at its prompt, and the running command's group while
// one holds the foreground, which is how the manager tells idle from working
// without any shell integration. The read goes through the master's SyscallConn
// (never a bare Fd), so it can't disturb the concurrent PTY read loop's poller.
// Zero when the platform or this PTY can't report it — treated as idle.
func foreground(p pty.Pty) int {
	u, ok := p.(pty.UnixPty)
	if !ok {
		return 0
	}
	var pgrp int
	var ierr error
	if cerr := u.Control(func(fd uintptr) {
		pgrp, ierr = unix.IoctlGetInt(int(fd), unix.TIOCGPGRP)
	}); cerr != nil || ierr != nil {
		return 0
	}
	return pgrp
}

// procName returns the executable name of the process leading group pid, or ""
// when it can't be read. `ps` is used rather than a platform-specific /proc or
// sysctl path so one implementation covers Linux and the BSDs/macOS; the
// foreground group changes rarely, so the sampler only pays for this exec when a
// new command takes the foreground.
func procName(pid int) string {
	out, err := exec.Command("ps", "-o", "comm=", "-p", strconv.Itoa(pid)).Output()
	if err != nil {
		return ""
	}
	name := strings.TrimSpace(string(out))
	if i := strings.LastIndexAny(name, `/\`); i >= 0 {
		name = name[i+1:]
	}
	return name
}

// procGroupNames returns every argv token of every process in process group pgid —
// the raw material agent identification scores. The group listing itself lives in
// internal/proc, which reads it per-process for transcript binding; identification
// wants only the flat token soup, so it flattens what that returns rather than
// running a second `ps` with a second parser to drift from.
func procGroupNames(pgid int) []string {
	var names []string
	for _, m := range proc.Group(pgid) {
		names = append(names, m.Argv...)
	}
	return names
}
