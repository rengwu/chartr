//go:build !windows

package terminal

import (
	"os/exec"
	"strconv"
	"strings"

	"github.com/aymanbagabas/go-pty"
	"golang.org/x/sys/unix"
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
// the raw material agent identification scores. It reads the whole group rather
// than its leader alone, and the *arguments* rather than the executable name,
// because that is the only place the agent's own name survives: a `node`-launched
// `claude` reports comm `node`, and a shell-script agent reports comm `/bin/sh`,
// with `claude` visible only in the command line.
//
// One `ps` covers Linux and the BSDs/macOS (as procName does), and the sampler
// only pays for it when the foreground group actually changes.
func procGroupNames(pgid int) []string {
	if pgid <= 0 {
		return nil
	}
	out, err := exec.Command("ps", "-A", "-o", "pgid=,args=").Output()
	if err != nil {
		return nil
	}
	want := strconv.Itoa(pgid)
	var names []string
	for _, line := range strings.Split(string(out), "\n") {
		fields := strings.Fields(line)
		if len(fields) < 2 || fields[0] != want {
			continue
		}
		names = append(names, fields[1:]...)
	}
	return names
}
