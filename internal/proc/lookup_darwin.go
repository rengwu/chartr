//go:build darwin

package proc

import (
	"bytes"
	"encoding/binary"
	"errors"
	"fmt"
	"os/exec"
	"strconv"
	"strings"
	"time"

	"golang.org/x/sys/unix"

	"github.com/rengwu/chartr/internal/adapter"
)

// macOS keeps no /proc, and the libproc calls that would answer all three
// questions at once are a C library rather than a syscall interface, which a
// cgo-free binary (ADR 0011) cannot reach. So the three facts come from the
// three places macOS actually publishes them:
//
//   - identity and start time from the kern.proc.pid sysctl, which returns the
//     kernel's own process record;
//   - the environment from the kern.procargs2 sysctl, the same buffer `ps -E`
//     reads, and readable only for one's own processes — which is exactly the
//     access boundary wanted here;
//   - the working directory from `lsof`, part of the macOS base system at
//     /usr/sbin/lsof, because it is the only interface to a process's cwd that
//     does not require linking libproc.
//
// errShape is what a buffer that is not the layout below reads as. It is not a
// distinct outcome for a caller — everything here resolves to unavailable — but
// it keeps a truncated read from being parsed as a short environment.
var errShape = errors.New("proc: unrecognised kern.procargs2 layout")

// errWithheld is the case that makes this file's parsing worth doing carefully.
// macOS answers kern.procargs2 for a SIP-protected platform binary — /bin/sleep,
// /bin/zsh — with its argv and *nothing else*: no environment, and none of the
// dyld strings that always follow one. That is the kernel declining, not a
// process with an empty environment, and the difference matters enormously.
// Read as "no variables set", it would resolve to the provider's documented
// default and could bind a tab running under a custom root to the wrong
// conversation. So it is an error, and the tab stays untitled.
//
// It costs nothing in practice: the agent CLIs this is pointed at are node
// scripts and Homebrew binaries under the operator's own prefixes, none of them
// platform binaries.
var errWithheld = errors.New("proc: the kernel withheld this process's environment")

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
	kp, err := unix.SysctlKinfoProc("kern.proc.pid", pid)
	if err != nil {
		return Info{}, fmt.Errorf("proc: reading process %d: %w", pid, err)
	}
	// A dead pid can still return a zeroed record rather than an error, so the
	// record has to be checked against the pid that was asked for.
	if int(kp.Proc.P_pid) != pid {
		return Info{}, fmt.Errorf("proc: no such process %d", pid)
	}
	start := kp.Proc.P_starttime
	env, err := environ(pid, allow)
	if err != nil {
		return Info{}, err
	}
	dir, err := workDir(pid)
	if err != nil {
		return Info{}, err
	}
	return Info{
		PID:     pid,
		PGID:    int(kp.Eproc.Pgid),
		Started: time.Unix(int64(start.Sec), int64(start.Usec)*int64(time.Microsecond)),
		Dir:     dir,
		Env:     env,
	}, nil
}

// environ reads pid's environment and keeps only the allowlisted variables. The
// sysctl answers for one's own processes and refuses for anyone else's, so an
// agent chartr has no business reading is an error here rather than a partial
// read — which is what makes "unreadable environment" resolve to unavailable
// instead of to the provider's default root.
func environ(pid int, allow []string) (map[string]string, error) {
	buf, err := unix.SysctlRaw("kern.procargs2", pid)
	if err != nil {
		return nil, fmt.Errorf("proc: reading process %d's environment: %w", pid, err)
	}
	entries, err := procArgs2Env(buf)
	if err != nil {
		return nil, err
	}
	return adapter.AllowedEnv(entries, allow), nil
}

// procArgs2Env pulls the environment entries out of a kern.procargs2 buffer,
// whose layout is:
//
//	int32 argc
//	the executable path, NUL-terminated
//	NUL padding to the next word
//	argc argv strings, each NUL-terminated
//	the environment, each entry NUL-terminated, ending at an empty entry
//	the dyld strings (executable_file=, dyld_file=, …), which are not the
//	process's environment and stop at the empty entry above
//
// argc is walked rather than the environment being sniffed for `KEY=VALUE`,
// because an argument may look exactly like one — `claude --model=x` — and an
// argument mistaken for a variable is precisely the false positive this whole
// path is built to avoid.
func procArgs2Env(buf []byte) ([]string, error) {
	if len(buf) < 4 {
		return nil, errShape
	}
	argc := int(int32(binary.NativeEndian.Uint32(buf[:4])))
	if argc < 0 {
		return nil, errShape
	}
	rest := buf[4:]

	// The executable path, then the NUL padding that separates it from argv[0].
	i := bytes.IndexByte(rest, 0)
	if i < 0 {
		return nil, errShape
	}
	rest = rest[i:]
	for len(rest) > 0 && rest[0] == 0 {
		rest = rest[1:]
	}

	for range argc {
		i := bytes.IndexByte(rest, 0)
		if i < 0 {
			return nil, errShape
		}
		rest = rest[i+1:]
	}

	// Nothing at all after argv is the kernel declining rather than a process
	// with no variables set: a real one always carries the dyld strings here,
	// even when its environment is empty.
	if len(bytes.Trim(rest, "\x00")) == 0 {
		return nil, errWithheld
	}

	var env []string
	for {
		i := bytes.IndexByte(rest, 0)
		if i <= 0 { // an empty entry, or an unterminated tail: the environment ends
			return env, nil
		}
		env = append(env, string(rest[:i]))
		rest = rest[i+1:]
	}
}

// workDir reads pid's working directory. `-d cwd` narrows lsof to the one
// descriptor wanted and `-Fn` puts it in the machine-readable form: lsof emits
// its process and descriptor fields alongside, so the answer arrives as an
// `n`-prefixed line. `-w` silences the warnings lsof otherwise writes for
// filesystems it cannot stat, which are not this question's business.
func workDir(pid int) (string, error) {
	out, err := exec.Command(lsof(), "-a", "-w", "-d", "cwd", "-p", strconv.Itoa(pid), "-Fn").Output()
	if err != nil {
		return "", fmt.Errorf("proc: reading process %d's working directory: %w", pid, err)
	}
	for _, line := range strings.Split(string(out), "\n") {
		if dir, ok := strings.CutPrefix(line, "n"); ok && dir != "" {
			return dir, nil
		}
	}
	return "", fmt.Errorf("proc: process %d reports no working directory", pid)
}

// lsof resolves the binary, preferring PATH so a host with a newer one installed
// uses it, and falling back to the base-system location — which is where it is
// on a stock macOS, and not on the default PATH of every launch context chartr
// can be started from (a .app bundle among them).
func lsof() string {
	if p, err := exec.LookPath("lsof"); err == nil {
		return p
	}
	return "/usr/sbin/lsof"
}
