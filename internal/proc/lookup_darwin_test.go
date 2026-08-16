//go:build darwin

package proc

import (
	"bytes"
	"encoding/binary"
	"errors"
	"os"
	"os/exec"
	"reflect"
	"syscall"
	"testing"
	"time"
)

// args2 builds a kern.procargs2 buffer: the argument count, the executable path
// with the NUL padding that follows it, the argv strings, and whatever the
// kernel put after them.
func args2(argc int32, execPath string, argv []string, tail ...string) []byte {
	var b bytes.Buffer
	_ = binary.Write(&b, binary.NativeEndian, argc)
	b.WriteString(execPath)
	b.WriteString("\x00\x00\x00\x00") // terminator plus word padding
	for _, a := range argv {
		b.WriteString(a)
		b.WriteByte(0)
	}
	for _, s := range tail {
		b.WriteString(s)
		b.WriteByte(0)
	}
	return b.Bytes()
}

// dyld is what macOS always appends after the environment. Its presence is how
// a withheld environment is told from an empty one, so the fixtures carry it.
var dyld = []string{"", "executable_file=0x1a01000011,0x982339b", "dyld_file=0x1a01000011,0xfff"}

func TestProcArgs2Env(t *testing.T) {
	for _, tc := range []struct {
		name string
		buf  []byte
		want []string
	}{{
		name: "the environment between argv and the dyld strings",
		buf: args2(2, "/opt/homebrew/bin/claude", []string{"claude", "--resume"},
			append([]string{"CLAUDE_CONFIG_DIR=/home/op/.claude2", "PATH=/usr/bin"}, dyld...)...),
		want: []string{"CLAUDE_CONFIG_DIR=/home/op/.claude2", "PATH=/usr/bin"},
	}, {
		// An argument that looks exactly like a variable is why argc is walked
		// rather than the tail being sniffed for `KEY=VALUE`.
		name: "an argument shaped like a variable is not read as one",
		buf: args2(3, "/opt/homebrew/bin/claude", []string{"claude", "--model=x", "CLAUDE_CONFIG_DIR=/spoofed"},
			append([]string{"CLAUDE_CONFIG_DIR=/real"}, dyld...)...),
		want: []string{"CLAUDE_CONFIG_DIR=/real"},
	}, {
		name: "a process with no variables set still reports the dyld strings",
		buf:  args2(1, "/opt/homebrew/bin/claude", []string{"claude"}, dyld...),
		want: nil,
	}} {
		t.Run(tc.name, func(t *testing.T) {
			got, err := procArgs2Env(tc.buf)
			if err != nil {
				t.Fatalf("procArgs2Env: %v", err)
			}
			if !reflect.DeepEqual(got, tc.want) {
				t.Fatalf("env = %q, want %q", got, tc.want)
			}
		})
	}
}

func TestProcArgs2EnvRejectsAWithheldEnvironment(t *testing.T) {
	// Argv and nothing after it: the shape macOS returns for a SIP-protected
	// platform binary. Read as "no variables set" it would resolve to the
	// provider's default root, so it has to be an error.
	buf := args2(2, "/bin/sleep", []string{"claude", "60"})
	if got, err := procArgs2Env(buf); !errors.Is(err, errWithheld) {
		t.Fatalf("procArgs2Env = %q, %v; want errWithheld", got, err)
	}
}

func TestProcArgs2EnvRejectsAnUnknownShape(t *testing.T) {
	for _, tc := range []struct {
		name string
		buf  []byte
	}{
		{"shorter than the argument count", []byte{1, 2}},
		{"an executable path with no terminator", append([]byte{1, 0, 0, 0}, []byte("/opt/claude")...)},
		{"fewer argv strings than the count claims", args2(4, "/opt/claude", []string{"claude"})},
	} {
		t.Run(tc.name, func(t *testing.T) {
			if got, err := procArgs2Env(tc.buf); !errors.Is(err, errShape) {
				t.Fatalf("procArgs2Env = %q, %v; want errShape", got, err)
			}
		})
	}
}

// The same rule against a live process: macOS declines to show a platform
// binary's environment, and a declined read is unavailable rather than a
// process that happens to have set nothing.
func TestLookupOnAPlatformBinaryFails(t *testing.T) {
	if _, err := os.Stat("/bin/sleep"); err != nil {
		t.Skipf("no /bin/sleep to test the SIP case against: %v", err)
	}
	cmd := exec.Command("/bin/sleep", "60")
	cmd.Env = []string{"CLAUDE_CONFIG_DIR=/tmp/whatever"}
	cmd.SysProcAttr = &syscall.SysProcAttr{Setpgid: true}
	if err := cmd.Start(); err != nil {
		t.Fatalf("starting /bin/sleep: %v", err)
	}
	t.Cleanup(func() {
		_ = cmd.Process.Kill()
		_, _ = cmd.Process.Wait()
	})
	// Give the kernel a moment to publish the process's arguments.
	time.Sleep(50 * time.Millisecond)

	if info, err := Lookup(cmd.Process.Pid, []string{"CLAUDE_CONFIG_DIR"}); err == nil {
		t.Fatalf("Lookup returned %+v; a withheld environment must not read as an empty one", info)
	}
}
