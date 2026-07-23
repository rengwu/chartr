package env

import (
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
)

func TestParsePATH(t *testing.T) {
	tests := []struct {
		name string
		out  string
		want string
		ok   bool
	}{
		{
			name: "plain",
			out:  beginMarker + "HOME=/home/x\nPATH=/usr/bin:/bin\nSHELL=/bin/zsh\n" + endMarker,
			want: "/usr/bin:/bin",
			ok:   true,
		},
		{
			// A login shell writing a MOTD or a prompt framework's preamble is
			// ordinary, not an error. The markers exist to discard it.
			name: "startup chatter outside the markers is discarded",
			out:  "Welcome to the machine\nlast login: today\n" + beginMarker + "PATH=/usr/bin\n" + endMarker + "\nsome trailing noise",
			want: "/usr/bin",
			ok:   true,
		},
		{
			// A variable whose value contains a newline puts arbitrary text at the
			// start of a line inside the payload. Only an exact assignment counts.
			name: "a PATH-like line inside another variable is not the answer",
			out:  beginMarker + "LS_COLORS=a:b\nPATH_HELPER=/usr/libexec\nMSG=hello\nPATH=/real\n" + endMarker,
			want: "/real",
			ok:   true,
		},
		{
			name: "carriage returns are trimmed",
			out:  beginMarker + "PATH=/usr/bin\r\n" + endMarker,
			want: "/usr/bin",
			ok:   true,
		},
		{
			// A shell that died before the payload, or one whose -c was never run.
			name: "no markers",
			out:  "zsh: command not found\n",
			ok:   false,
		},
		{
			name: "truncated output loses the end marker",
			out:  beginMarker + "PATH=/usr/bin\n",
			ok:   false,
		},
		{
			name: "no PATH in the payload",
			out:  beginMarker + "HOME=/home/x\n" + endMarker,
			ok:   false,
		},
		{
			name: "empty PATH is not an answer",
			out:  beginMarker + "PATH=\n" + endMarker,
			ok:   false,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			got, ok := parsePATH(tc.out)
			if ok != tc.ok {
				t.Fatalf("ok = %v, want %v", ok, tc.ok)
			}
			if got != tc.want {
				t.Errorf("got %q, want %q", got, tc.want)
			}
		})
	}
}

func TestMergePATHIsAdditiveAndOrderPreserving(t *testing.T) {
	sep := string(os.PathListSeparator)
	join := func(parts ...string) string { return strings.Join(parts, sep) }

	tests := []struct {
		name    string
		current string
		shell   string
		want    string
		changed bool
	}{
		{
			name:    "missing entries are appended",
			current: join("/usr/bin", "/bin"),
			shell:   join("/opt/homebrew/bin", "/usr/bin"),
			want:    join("/usr/bin", "/bin", "/opt/homebrew/bin"),
			changed: true,
		},
		{
			// The safety property: a name that already resolved must resolve to the
			// same file afterwards. Appending guarantees it; prepending would not,
			// because a shell PATH routinely holds the same directories in a
			// different order and would silently change which copy runs.
			name:    "a reordered shell PATH does not reorder ours",
			current: join("/first", "/second"),
			shell:   join("/second", "/first"),
			want:    join("/first", "/second"),
			changed: false,
		},
		{
			name:    "nothing new is reported unchanged",
			current: join("/usr/bin", "/bin"),
			shell:   join("/bin"),
			want:    join("/usr/bin", "/bin"),
			changed: false,
		},
		{
			// An empty entry means "the current directory". That is a resolution rule
			// chartr will not carry into a list of places to find an agent.
			name:    "empty entries are dropped, not propagated",
			current: join("/usr/bin", ""),
			shell:   join("", "/opt/bin"),
			want:    join("/usr/bin", "/opt/bin"),
			changed: true,
		},
		{
			name:    "an empty inherited PATH takes the shell's whole answer",
			current: "",
			shell:   join("/usr/bin", "/bin"),
			want:    join("/usr/bin", "/bin"),
			changed: true,
		},
		{
			name:    "duplicates in the shell PATH are collapsed",
			current: "/usr/bin",
			shell:   join("/opt/bin", "/opt/bin"),
			want:    join("/usr/bin", "/opt/bin"),
			changed: true,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			got, changed := mergePATH(tc.current, tc.shell)
			if changed != tc.changed {
				t.Fatalf("changed = %v, want %v", changed, tc.changed)
			}
			if got != tc.want {
				t.Errorf("got %q, want %q", got, tc.want)
			}
		})
	}
}

// TestHydratePATHFindsABinaryOnlyTheShellKnows is the whole point of the package,
// exercised end to end against real shells: a directory named only in a startup
// file is unfindable until the probe runs, and findable after. It writes real rc
// files and runs a real shell rather than stubbing the probe, because the bug
// being fixed lives precisely in which startup files a shell reads.
//
// The two shells are set up differently on purpose, because they disagree about
// that, and the disagreement is what `-l -i` is chosen to straddle:
//
//   - zsh reads `.zshrc` for any *interactive* shell, login or not. An agent
//     installer's `export PATH=…` line lands there and is read directly. This is
//     the macOS default shell and the case that motivated the package.
//   - bash reads `.bashrc` only for an interactive *non-login* shell; as a login
//     shell it reads `.bash_profile` instead. The near-universal convention — and
//     the distro default — is for `.bash_profile` to source `.bashrc`, so the line
//     is still reached, one hop further along.
//
// Dropping `-l` would break zsh users whose PATH is set in `.zprofile`; dropping
// `-i` would break the far larger group whose PATH is set in `.zshrc`. Both flags
// are load-bearing, and this test fails if either is removed.
func TestHydratePATHFindsABinaryOnlyTheShellKnows(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("no login shell to probe on Windows; HydratePATH is a no-op there")
	}

	shells := []struct {
		name string
		// setup writes the startup files that put agentDir on PATH, and returns any
		// extra environment the shell needs to find them.
		setup func(t *testing.T, home, agentDir string) map[string]string
	}{
		{
			name: "zsh",
			setup: func(t *testing.T, home, agentDir string) map[string]string {
				write(t, filepath.Join(home, ".zshrc"), "export PATH="+agentDir+":$PATH\n")
				// ZDOTDIR is where zsh looks for its startup files; setting it keeps the
				// fixture from depending on HOME being honoured.
				return map[string]string{"ZDOTDIR": home}
			},
		},
		{
			name: "bash",
			setup: func(t *testing.T, home, agentDir string) map[string]string {
				write(t, filepath.Join(home, ".bashrc"), "export PATH="+agentDir+":$PATH\n")
				write(t, filepath.Join(home, ".bash_profile"), ". \"$HOME/.bashrc\"\n")
				return nil
			},
		},
	}

	for _, sh := range shells {
		t.Run(sh.name, func(t *testing.T) {
			shell, err := exec.LookPath(sh.name)
			if err != nil {
				t.Skipf("%s not available to act as the operator's shell", sh.name)
			}

			// The agent binary, in a directory nothing but the startup files mention.
			agentDir := t.TempDir()
			write(t, filepath.Join(agentDir, "fake-agent-cli"), "#!/bin/sh\nexit 0\n")
			if err := os.Chmod(filepath.Join(agentDir, "fake-agent-cli"), 0o755); err != nil {
				t.Fatal(err)
			}

			home := t.TempDir()
			extra := sh.setup(t, home, agentDir)

			t.Setenv("HOME", home)
			t.Setenv("SHELL", shell)
			for k, v := range extra {
				t.Setenv(k, v)
			}
			t.Setenv("PATH", "/usr/bin:/bin")

			if _, err := exec.LookPath("fake-agent-cli"); err == nil {
				t.Fatal("precondition: fake-agent-cli should not resolve before hydration")
			}

			HydratePATH()

			if _, err := exec.LookPath("fake-agent-cli"); err != nil {
				t.Fatalf("after HydratePATH, fake-agent-cli still does not resolve: %v\nPATH=%s",
					err, os.Getenv("PATH"))
			}
		})
	}
}

func write(t *testing.T, path, content string) {
	t.Helper()
	if err := os.WriteFile(path, []byte(content), 0o644); err != nil {
		t.Fatal(err)
	}
}

func TestHydratePATHLeavesPATHAloneWhenItCannotAsk(t *testing.T) {
	const inherited = "/usr/bin:/bin"

	t.Run("no shell to ask", func(t *testing.T) {
		t.Setenv("SHELL", "")
		t.Setenv("PATH", inherited)
		HydratePATH()
		if got := os.Getenv("PATH"); got != inherited {
			t.Errorf("PATH = %q, want the inherited %q", got, inherited)
		}
	})

	t.Run("the shell does not exist", func(t *testing.T) {
		t.Setenv("SHELL", filepath.Join(t.TempDir(), "no-such-shell"))
		t.Setenv("PATH", inherited)
		HydratePATH()
		if got := os.Getenv("PATH"); got != inherited {
			t.Errorf("PATH = %q, want the inherited %q", got, inherited)
		}
	})

	// An operator who refuses the probe keeps exactly what they were launched with.
	t.Run("the probe is switched off", func(t *testing.T) {
		shell, err := exec.LookPath("sh")
		if err != nil {
			t.Skip("no sh available")
		}
		t.Setenv(disableVar, "1")
		t.Setenv("SHELL", shell)
		t.Setenv("PATH", inherited)
		HydratePATH()
		if got := os.Getenv("PATH"); got != inherited {
			t.Errorf("PATH = %q, want the inherited %q", got, inherited)
		}
	})
}

// TestLookPathAcceptsAFullPath pins the behaviour the registration error message
// now promises: a name containing a separator bypasses PATH entirely, so a full
// path is always a valid adapter however chartr was launched.
func TestLookPathAcceptsAFullPath(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("shell-script fixture is not executable on Windows")
	}
	dir := t.TempDir()
	bin := filepath.Join(dir, "somewhere-odd")
	if err := os.WriteFile(bin, []byte("#!/bin/sh\nexit 0\n"), 0o755); err != nil {
		t.Fatal(err)
	}
	t.Setenv("PATH", "/usr/bin:/bin")

	if _, err := exec.LookPath("somewhere-odd"); err == nil {
		t.Fatal("precondition: the bare name should not resolve")
	}
	if _, err := exec.LookPath(bin); err != nil {
		t.Errorf("a full path should resolve regardless of PATH: %v", err)
	}
}
