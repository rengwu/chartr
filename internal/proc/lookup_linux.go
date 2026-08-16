//go:build linux

package proc

import (
	"errors"
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/rengwu/chartr/internal/adapter"
)

// Linux publishes all three facts under /proc, so this reader is three file
// reads and no exec. The one piece of arithmetic is the start time: /proc
// reports it as ticks since boot rather than as a wall clock, so it is added to
// the boot time /proc/stat carries.
//
// errShape is what a /proc file that is not the documented layout reads as —
// a kernel that changed its stat format is unavailable rather than parsed on a
// guess, the same rule the transcript adapters hold for provider schemas.
var errShape = errors.New("proc: unrecognised /proc layout")

// userHZ is the unit /proc reports process times in. It is fixed at 100 for the
// /proc interface regardless of the kernel's own tick rate — the kernel converts
// on the way out precisely so this number is a constant of the interface rather
// than of the build.
const userHZ = 100

// Lookup reads one process's identity, start time, working directory and
// allowlisted environment. Only the variables named in allow are returned; the
// rest of the process's environment is discarded here, before this function
// returns, and never reaches a caller.
//
// Any failure — a process that has exited, an environment belonging to another
// user, a working directory that cannot be read — is an error, never a partial
// answer. A caller cannot tell a guessed fact from a read one, so there are no
// guessed facts.
func Lookup(pid int, allow []string) (Info, error) {
	if pid <= 0 {
		return Info{}, fmt.Errorf("proc: no such process %d", pid)
	}
	root := "/proc/" + strconv.Itoa(pid)

	stat, err := os.ReadFile(root + "/stat")
	if err != nil {
		return Info{}, fmt.Errorf("proc: reading process %d: %w", pid, err)
	}
	pgid, ticks, err := parseStat(string(stat))
	if err != nil {
		return Info{}, err
	}
	boot, err := bootTime()
	if err != nil {
		return Info{}, err
	}

	// /proc/<pid>/environ is readable only by the process's own owner, so an
	// agent chartr has no business reading is an error here rather than a
	// partial read — which is what makes "unreadable environment" resolve to
	// unavailable instead of to the provider's default root.
	raw, err := os.ReadFile(root + "/environ")
	if err != nil {
		return Info{}, fmt.Errorf("proc: reading process %d's environment: %w", pid, err)
	}
	// Everything but the allowlisted variables is dropped right here: the
	// buffer and its split entries are unreachable the moment this returns.
	env := adapter.AllowedEnv(splitNUL(raw), allow)

	dir, err := os.Readlink(root + "/cwd")
	if err != nil {
		return Info{}, fmt.Errorf("proc: reading process %d's working directory: %w", pid, err)
	}

	return Info{
		PID:  pid,
		PGID: pgid,
		// Divided before multiplying: a tick is exactly 10ms, and doing it
		// this way keeps a long-running host's tick count away from the
		// int64 nanosecond ceiling.
		Started: boot.Add(time.Duration(ticks) * (time.Second / userHZ)),
		Dir:     dir,
		Env:     env,
	}, nil
}

// parseStat pulls the process group id (field 5) and the start time in ticks
// since boot (field 22) out of a /proc/<pid>/stat line.
//
// Fields are counted from the last `)` rather than from the start of the line:
// field 2 is the executable's name in parentheses, and an executable named
// `my prog) (x` would otherwise shift every field after it. This is the one
// documented hazard of the format, and the reason for the whole function.
func parseStat(stat string) (pgid int, ticks uint64, err error) {
	end := strings.LastIndexByte(stat, ')')
	if end < 0 {
		return 0, 0, errShape
	}
	// Fields from here are 3 onward, so index 0 below is field 3.
	fields := strings.Fields(stat[end+1:])
	const (
		pgrp      = 5 - 3
		starttime = 22 - 3
	)
	if len(fields) <= starttime {
		return 0, 0, errShape
	}
	pgid, err = strconv.Atoi(fields[pgrp])
	if err != nil {
		return 0, 0, errShape
	}
	ticks, err = strconv.ParseUint(fields[starttime], 10, 64)
	if err != nil {
		return 0, 0, errShape
	}
	return pgid, ticks, nil
}

// bootTime reads the wall clock the kernel booted at, which is what a
// ticks-since-boot start time has to be added to. /proc/stat carries it as a
// `btime <seconds>` line.
func bootTime() (time.Time, error) {
	raw, err := os.ReadFile("/proc/stat")
	if err != nil {
		return time.Time{}, fmt.Errorf("proc: reading boot time: %w", err)
	}
	for _, line := range strings.Split(string(raw), "\n") {
		secs, ok := strings.CutPrefix(line, "btime ")
		if !ok {
			continue
		}
		n, err := strconv.ParseInt(strings.TrimSpace(secs), 10, 64)
		if err != nil {
			return time.Time{}, errShape
		}
		return time.Unix(n, 0), nil
	}
	return time.Time{}, errShape
}

// splitNUL reads a NUL-separated /proc buffer into its entries, dropping the
// empty tail the final separator leaves.
func splitNUL(raw []byte) []string {
	var out []string
	for _, entry := range strings.Split(string(raw), "\x00") {
		if entry != "" {
			out = append(out, entry)
		}
	}
	return out
}
