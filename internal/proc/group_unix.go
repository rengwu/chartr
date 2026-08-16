//go:build !windows

package proc

import (
	"os/exec"
	"strconv"
	"strings"
)

// Group returns every process in process group pgid, each with the argv tokens
// agent identification scores. It reads the whole group rather than its leader
// alone, and the *arguments* rather than the executable name, because that is
// the only place an agent's own name survives: a `node`-launched `claude`
// reports comm `node`, and a shell-script agent reports comm `/bin/sh`, with
// `claude` visible only in the command line.
//
// One `ps` covers Linux and the BSDs/macOS, which is why the group listing is
// `!windows` while reading a single process's facts (Lookup) is not: `ps` is
// portable, `/proc` and `sysctl` are not. Callers pay for this exec only when a
// tab's foreground group actually changes.
func Group(pgid int) []Member {
	if pgid <= 0 {
		return nil
	}
	out, err := exec.Command("ps", "-A", "-o", "pgid=,pid=,args=").Output()
	if err != nil {
		return nil
	}
	want := strconv.Itoa(pgid)
	var members []Member
	for _, line := range strings.Split(string(out), "\n") {
		fields := strings.Fields(line)
		if len(fields) < 3 || fields[0] != want {
			continue
		}
		pid, err := strconv.Atoi(fields[1])
		if err != nil {
			continue
		}
		members = append(members, Member{PID: pid, Argv: fields[2:]})
	}
	return members
}
