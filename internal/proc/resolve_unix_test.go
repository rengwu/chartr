//go:build darwin || linux

package proc

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"syscall"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/terminal/detect"
)

// These tests drive the platform readers against processes that really exist,
// on the two platforms that have a reader. Everything below is a real pid, a
// real environment and a real working directory; the selection and
// normalization rules they exercise on the way through are the same ones
// resolve_test.go proves in isolation.
//
// The identification seam is the shipped detection engine rather than a stub,
// so a manifest that stopped naming claude fails here too.

var identify = detect.Builtin().Identify

// spawnAgent starts a live process that a real chartr install would identify as
// adapterName: an ordinary `sleep` wearing the agent's name as its argv[0],
// which is exactly what identification reads. It gets its own process group, so
// its pid is also the foreground group id a tab would report.
//
// The binary is a *copy* of sleep rather than sleep itself, because macOS
// answers kern.procargs2 for a SIP-protected platform binary with its argv and
// no environment at all (lookup_darwin.go). Agent CLIs are node scripts and
// Homebrew binaries, never platform binaries, so a copy is the faithful
// stand-in and /bin/sleep is the misleading one.
func spawnAgent(t *testing.T, adapterName, dir string, env ...string) int {
	t.Helper()
	cmd := exec.Command(unprotectedSleep(t), "60")
	cmd.Args[0] = adapterName // the process's own idea of what it is
	cmd.Dir = dir
	cmd.Env = env
	cmd.SysProcAttr = &syscall.SysProcAttr{Setpgid: true}
	if err := cmd.Start(); err != nil {
		t.Fatalf("starting the stand-in agent: %v", err)
	}
	t.Cleanup(func() {
		_ = cmd.Process.Kill()
		_, _ = cmd.Process.Wait()
	})
	return cmd.Process.Pid
}

// unprotectedSleep copies the system's sleep into a directory of the test's own,
// once per package run, so the processes above are ordinary user binaries.
func unprotectedSleep(t *testing.T) string {
	t.Helper()
	copyOnce.Do(func() {
		src, err := exec.LookPath("sleep")
		if err != nil {
			copyErr = err
			return
		}
		body, err := os.ReadFile(src)
		if err != nil {
			copyErr = err
			return
		}
		dir, err := os.MkdirTemp("", "chartr-proc-test")
		if err != nil {
			copyErr = err
			return
		}
		copied = filepath.Join(dir, "sleep")
		copyErr = os.WriteFile(copied, body, 0o755)
	})
	if copyErr != nil {
		t.Skipf("no usable sleep binary on this host: %v", copyErr)
	}
	return copied
}

var (
	copyOnce sync.Once
	copied   string
	copyErr  error
)

// realPath is what the platform reader will report for dir. macOS hands back
// temporary directories through /var and reads them back through /private/var,
// so a test comparing paths has to compare the resolved ones.
func realPath(t *testing.T, dir string) string {
	t.Helper()
	resolved, err := filepath.EvalSymlinks(dir)
	if err != nil {
		t.Fatalf("resolving %s: %v", dir, err)
	}
	return resolved
}

func TestLookupReadsARealProcess(t *testing.T) {
	dir := t.TempDir()
	root := t.TempDir()
	before := time.Now()
	pid := spawnAgent(t, "claude", dir, "CLAUDE_CONFIG_DIR="+root)
	after := time.Now()

	info, err := Lookup(pid, []string{"CLAUDE_CONFIG_DIR"})
	if err != nil {
		t.Fatalf("Lookup: %v", err)
	}
	if info.PID != pid {
		t.Errorf("PID = %d, want %d", info.PID, pid)
	}
	// Started under its own process group, so the group is the process itself.
	if info.PGID != pid {
		t.Errorf("PGID = %d, want %d", info.PGID, pid)
	}
	if got, want := info.Dir, realPath(t, dir); got != want {
		t.Errorf("Dir = %q, want %q", got, want)
	}
	if info.Env["CLAUDE_CONFIG_DIR"] != root {
		t.Errorf("Env[CLAUDE_CONFIG_DIR] = %q, want %q", info.Env["CLAUDE_CONFIG_DIR"], root)
	}
	// A second's slack each way: the clocks the kernel and the test read are
	// the same one, but the process was started between the two samples.
	if info.Started.Before(before.Add(-time.Second)) || info.Started.After(after.Add(time.Second)) {
		t.Errorf("Started = %v, want it between %v and %v", info.Started, before, after)
	}
}

// The allowlist is the whole of what leaves the reader. A variable outside it
// must not appear in the returned map, nor anywhere else in the value — the
// reader holds the only copy of a process environment chartr ever sees, and it
// drops it before returning.
func TestLookupKeepsOnlyTheAllowlistedVariables(t *testing.T) {
	const sentinel = "sk-lookup-must-not-return-this"
	root := t.TempDir()
	pid := spawnAgent(t, "claude", t.TempDir(),
		"CLAUDE_CONFIG_DIR="+root,
		"ANTHROPIC_API_KEY="+sentinel,
		"AWS_SECRET_ACCESS_KEY="+sentinel,
	)

	info, err := Lookup(pid, []string{"CLAUDE_CONFIG_DIR"})
	if err != nil {
		t.Fatalf("Lookup: %v", err)
	}
	if len(info.Env) != 1 {
		t.Errorf("Env = %v, want claude's one allowlisted variable", info.Env)
	}
	if rendered := fmt.Sprintf("%#v", info); strings.Contains(rendered, sentinel) {
		t.Errorf("a non-allowlisted value escaped the reader: %s", rendered)
	}
}

// An adapter with no state-root row asks for no variables, and a reader asked
// for none must return none.
func TestLookupWithAnEmptyAllowlistReturnsNoEnvironment(t *testing.T) {
	pid := spawnAgent(t, "claude", t.TempDir(), "CLAUDE_CONFIG_DIR="+t.TempDir())
	info, err := Lookup(pid, nil)
	if err != nil {
		t.Fatalf("Lookup: %v", err)
	}
	if len(info.Env) != 0 {
		t.Errorf("Env = %v, want nothing", info.Env)
	}
}

// An environment chartr is not entitled to read is an error, never a partial
// answer — the condition that must resolve to unavailable rather than to the
// provider's documented default.
func TestLookupOnAnInaccessibleEnvironmentFails(t *testing.T) {
	pid := foreignPID(t)
	if _, err := Lookup(pid, []string{"CLAUDE_CONFIG_DIR"}); err == nil {
		t.Fatalf("Lookup(%d) succeeded; another user's process environment must not be readable", pid)
	}
}

// foreignPID finds a live process belonging to another user, or skips. Running
// as root there is no such thing, and a container may hold nothing but this
// test's own uid — both are legitimate hosts to have no inaccessible case on,
// and neither is a reason to assert against a pid that happens to be our own.
func foreignPID(t *testing.T) int {
	t.Helper()
	if os.Geteuid() == 0 {
		t.Skip("running as root: every process's environment is readable, so there is no inaccessible case here")
	}
	out, err := exec.Command("ps", "-A", "-o", "uid=,pid=").Output()
	if err != nil {
		t.Skipf("cannot list processes: %v", err)
	}
	for _, line := range strings.Split(string(out), "\n") {
		fields := strings.Fields(line)
		if len(fields) < 2 {
			continue
		}
		uid, err := strconv.Atoi(fields[0])
		if err != nil || uid == os.Geteuid() {
			continue
		}
		if pid, err := strconv.Atoi(fields[1]); err == nil && pid > 0 {
			return pid
		}
	}
	t.Skip("every visible process belongs to this user, so there is no inaccessible case here")
	return 0
}

func TestLookupOnADeadProcessFails(t *testing.T) {
	sleep, err := exec.LookPath("sleep")
	if err != nil {
		t.Skipf("no sleep on this host: %v", err)
	}
	cmd := exec.Command(sleep, "0")
	if err := cmd.Run(); err != nil {
		t.Fatal(err)
	}
	if _, err := Lookup(cmd.Process.Pid, []string{"CLAUDE_CONFIG_DIR"}); err == nil {
		t.Fatal("Lookup on an exited process succeeded")
	}
}

func TestGroupListsTheProcessesInAGroup(t *testing.T) {
	pid := spawnAgent(t, "claude", t.TempDir())
	members := Group(pid)
	if len(members) == 0 {
		t.Fatalf("Group(%d) found nothing", pid)
	}
	for _, m := range members {
		if m.PID == pid && identify(m.Argv) == "claude" {
			return
		}
	}
	t.Fatalf("Group(%d) = %+v, want the claude process among them", pid, members)
}

// The isolation two parallel Claude accounts depend on, proved against two live
// processes: same executable, same argv, same working directory, same adapter
// name, two configuration directories — and two roots that share nothing.
func TestForegroundSeparatesTwoLiveClaudeRoots(t *testing.T) {
	dir := t.TempDir()
	first, second := t.TempDir(), t.TempDir()
	pidA := spawnAgent(t, "claude", dir, "CLAUDE_CONFIG_DIR="+first)
	pidB := spawnAgent(t, "claude", dir, "CLAUDE_CONFIG_DIR="+second)

	a, ok := Foreground(pidA, identify)
	if !ok {
		t.Fatal("the first claude resolved to unavailable")
	}
	b, ok := Foreground(pidB, identify)
	if !ok {
		t.Fatal("the second claude resolved to unavailable")
	}
	if a.StateRoot != first || b.StateRoot != second {
		t.Fatalf("roots = %q and %q, want %q and %q", a.StateRoot, b.StateRoot, first, second)
	}
	if a.PID == b.PID || a.Dir != b.Dir {
		t.Fatalf("the two tabs should differ only by root: %+v vs %+v", a, b)
	}
	if a.Adapter != "claude" || b.Adapter != "claude" {
		t.Fatalf("adapters = %q and %q", a.Adapter, b.Adapter)
	}
}

// A relative or user-relative value means to chartr what it meant to the
// process that was started with it, so both are normalized before the root is
// used and both land somewhere real.
func TestForegroundNormalizesRelativeAndUserRelativeRoots(t *testing.T) {
	home := t.TempDir()
	t.Setenv("HOME", home) // what os.UserHomeDir, and so a `~`, resolves to
	dir := t.TempDir()
	if err := os.Mkdir(filepath.Join(dir, "local-root"), 0o700); err != nil {
		t.Fatal(err)
	}
	if err := os.Mkdir(filepath.Join(home, ".claude2"), 0o700); err != nil {
		t.Fatal(err)
	}

	relative, ok := Foreground(spawnAgent(t, "claude", dir, "CLAUDE_CONFIG_DIR=./local-root"), identify)
	if !ok {
		t.Fatal("a relative configuration directory resolved to unavailable")
	}
	if want := filepath.Join(realPath(t, dir), "local-root"); relative.StateRoot != want {
		t.Errorf("StateRoot = %q, want %q", relative.StateRoot, want)
	}

	userRelative, ok := Foreground(spawnAgent(t, "claude", dir, "CLAUDE_CONFIG_DIR=~/.claude2"), identify)
	if !ok {
		t.Fatal("a user-relative configuration directory resolved to unavailable")
	}
	if want := filepath.Join(home, ".claude2"); userRelative.StateRoot != want {
		t.Errorf("StateRoot = %q, want %q", userRelative.StateRoot, want)
	}
}

// A root that is not there is not a root: the value is normalized, found to name
// nothing, and the tab stays unbound rather than being handed a path nothing can
// be read from.
func TestForegroundRejectsARootThatIsNotADirectory(t *testing.T) {
	dir := t.TempDir()
	pid := spawnAgent(t, "claude", dir, "CLAUDE_CONFIG_DIR="+filepath.Join(dir, "never-created"))
	if got, ok := Foreground(pid, identify); ok {
		t.Fatalf("resolved %+v, want unavailable", got)
	}
}

func TestForegroundWithoutAnAgentIsUnavailable(t *testing.T) {
	pid := spawnAgent(t, "definitely-not-an-agent", t.TempDir(), "CLAUDE_CONFIG_DIR="+t.TempDir())
	if got, ok := Foreground(pid, identify); ok {
		t.Fatalf("resolved %+v, want unavailable — nothing in the group is an agent", got)
	}
}

// A tab chartr launched skips identification and reaches the same facts.
func TestLaunchedResolvesALiveProcess(t *testing.T) {
	dir := t.TempDir()
	root := t.TempDir()
	// Named for nothing in particular: the launch is what says it is claude.
	pid := spawnAgent(t, "some-wrapper", dir, "CLAUDE_CONFIG_DIR="+root)

	got, ok := Launched("claude", pid)
	if !ok {
		t.Fatal("unavailable")
	}
	if got.StateRoot != root || got.PID != pid || got.Dir != realPath(t, dir) {
		t.Fatalf("resolved %+v, want root %q pid %d dir %q", got, root, pid, realPath(t, dir))
	}
}
