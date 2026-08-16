package proc

import (
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"testing"
	"time"
)

// The tests below drive the pure half of resolution — which process in a
// foreground group is the agent, and what its state root comes out as — through
// the host seam, so the rules are exercised on every platform including the ones
// with no process reader at all. The platform readers themselves are proved
// against real processes in resolve_unix_test.go.

var epoch = time.Date(2026, 8, 16, 9, 0, 0, 0, time.UTC)

// fakeHost is a process table written by hand: a group of members, the facts
// each of them reads back, and the directories that exist.
type fakeHost struct {
	members []Member
	facts   map[int]Info
	errs    map[int]error
	dirs    map[string]bool
	home    string
	homeErr error
	// asked records the allowlist every lookup was made with, so a test can
	// prove the reader was never handed a wider one than the adapter declares.
	asked [][]string
}

func (f *fakeHost) host() host {
	return host{
		group: func(int) []Member { return f.members },
		lookup: func(pid int, allow []string) (Info, error) {
			f.asked = append(f.asked, allow)
			if err := f.errs[pid]; err != nil {
				return Info{}, err
			}
			info, ok := f.facts[pid]
			if !ok {
				return Info{}, fmt.Errorf("no such process %d", pid)
			}
			return info, nil
		},
		isDir: func(path string) bool { return f.dirs[path] },
		home:  func() (string, error) { return f.home, f.homeErr },
	}
}

// identifyByToken is the detection engine's contract in miniature: an argv names
// an agent when one of its tokens is that agent's name, and a generic runtime
// never speaks for the agent it launched.
func identifyByToken(argv []string) string {
	for _, tok := range argv {
		base := tok
		if i := strings.LastIndexByte(base, '/'); i >= 0 {
			base = base[i+1:]
		}
		switch base {
		case "node", "sh", "bash", "npx":
			continue
		case "claude", "kimi", "codex":
			return base
		}
	}
	return ""
}

// oneClaude is the ordinary case: a single process, the group leader, running
// claude with its default root.
func oneClaude(home string) *fakeHost {
	return &fakeHost{
		members: []Member{{PID: 100, Argv: []string{"claude"}}},
		facts: map[int]Info{
			100: {PID: 100, PGID: 100, Started: epoch, Dir: "/work/repo"},
		},
		errs: map[int]error{},
		dirs: map[string]bool{home + "/.claude": true},
		home: home,
	}
}

func TestForegroundResolvesTheDefaultRoot(t *testing.T) {
	f := oneClaude("/home/operator")
	got, ok := foreground(100, identifyByToken, f.host())
	if !ok {
		t.Fatal("a plain claude in the foreground resolved to unavailable")
	}
	want := Agent{
		Adapter:   "claude",
		PID:       100,
		PGID:      100,
		Started:   epoch,
		Dir:       "/work/repo",
		StateRoot: "/home/operator/.claude",
	}
	if got.Adapter != want.Adapter || got.PID != want.PID || got.PGID != want.PGID ||
		!got.Started.Equal(want.Started) || got.Dir != want.Dir || got.StateRoot != want.StateRoot {
		t.Fatalf("resolved %+v, want %+v", got, want)
	}
}

// The allowlist handed to the reader is the adapter's own and nothing more —
// the guarantee the whole environment rule rests on, checked at the call site
// rather than only at the reader's return.
func TestForegroundAsksOnlyForTheAdaptersAllowlist(t *testing.T) {
	f := oneClaude("/home/operator")
	if _, ok := foreground(100, identifyByToken, f.host()); !ok {
		t.Fatal("unavailable")
	}
	if len(f.asked) == 0 {
		t.Fatal("no lookup was made")
	}
	for _, allow := range f.asked {
		if len(allow) != 1 || allow[0] != "CLAUDE_CONFIG_DIR" {
			t.Fatalf("reader was handed %v, want claude's allowlist alone", allow)
		}
	}
}

// A wrapper shell's argv carries the agent's name as a token, so it identifies
// too — but it is not the agent. The process actually running the binary is.
func TestForegroundPrefersTheProcessRunningTheBinary(t *testing.T) {
	f := oneClaude("/home/operator")
	f.members = []Member{
		{PID: 100, Argv: []string{"sh", "-c", "claude"}}, // the leader, and not the agent
		{PID: 101, Argv: []string{"claude"}},
	}
	f.facts[101] = Info{PID: 101, PGID: 100, Started: epoch.Add(time.Second), Dir: "/work/repo"}

	got, ok := foreground(100, identifyByToken, f.host())
	if !ok {
		t.Fatal("unavailable")
	}
	if got.PID != 101 {
		t.Fatalf("bound to pid %d, want the claude process 101 rather than the shell that named it", got.PID)
	}
}

// An agent that runs a copy of itself must not hand the tab its child's
// identity: the leader is the tab's own agent, and the subprocess is passing
// through.
func TestForegroundPrefersTheGroupLeaderOverItsOwnSubprocess(t *testing.T) {
	f := oneClaude("/home/operator")
	f.members = []Member{
		{PID: 100, Argv: []string{"claude"}},
		{PID: 101, Argv: []string{"claude", "-p", "summarise"}},
	}
	f.facts[101] = Info{PID: 101, PGID: 100, Started: epoch.Add(time.Minute), Dir: "/work/repo"}

	got, ok := foreground(100, identifyByToken, f.host())
	if !ok {
		t.Fatal("unavailable")
	}
	if got.PID != 100 {
		t.Fatalf("bound to pid %d, want the group leader 100", got.PID)
	}
}

// A generic runtime is what the agent's own binary is, so the leader stands when
// nothing in the group is a bare exec of the agent's name.
func TestForegroundAcceptsARuntimeLaunchedAgent(t *testing.T) {
	f := oneClaude("/home/operator")
	f.members = []Member{{PID: 100, Argv: []string{"node", "/opt/bin/claude"}}}

	got, ok := foreground(100, identifyByToken, f.host())
	if !ok {
		t.Fatal("unavailable")
	}
	if got.PID != 100 || got.Adapter != "claude" {
		t.Fatalf("resolved pid %d adapter %q, want 100/claude", got.PID, got.Adapter)
	}
}

func TestForegroundUnavailable(t *testing.T) {
	const home = "/home/operator"
	for _, tc := range []struct {
		name string
		with func(*fakeHost)
	}{{
		name: "no process in the group",
		with: func(f *fakeHost) { f.members = nil },
	}, {
		name: "nothing in the group names an agent",
		with: func(f *fakeHost) { f.members = []Member{{PID: 100, Argv: []string{"vim", "notes.md"}}} },
	}, {
		// Two vendors in one group cannot both own the tab's transcript, and
		// picking either would be a coin flip that reads the wrong conversation.
		name: "two adapters in one group are ambiguous",
		with: func(f *fakeHost) {
			f.members = []Member{
				{PID: 100, Argv: []string{"claude"}},
				{PID: 101, Argv: []string{"codex", "exec"}},
			}
			f.facts[101] = Info{PID: 101, PGID: 100, Started: epoch, Dir: "/work/repo"}
		},
	}, {
		// Neither is the leader and both are a bare exec of the agent: there is
		// no rule left that picks one, so no binding is made.
		name: "two same-adapter peers with no leader are ambiguous",
		with: func(f *fakeHost) {
			f.members = []Member{
				{PID: 101, Argv: []string{"claude"}},
				{PID: 102, Argv: []string{"claude"}},
			}
			f.facts[101] = Info{PID: 101, PGID: 100, Started: epoch, Dir: "/work/repo"}
			f.facts[102] = Info{PID: 102, PGID: 100, Started: epoch, Dir: "/work/repo"}
		},
	}, {
		// The case the whole allowlist rule turns on: an environment chartr
		// cannot read must never fall through to the documented default, since
		// the process may well be running under a custom root.
		name: "an inaccessible process environment",
		with: func(f *fakeHost) { f.errs[100] = errors.New("operation not permitted") },
	}, {
		name: "the process is gone",
		with: func(f *fakeHost) { f.facts = map[int]Info{} },
	}, {
		// codex has no state-root row until ticket 04 measures one.
		name: "an adapter with no state-root knowledge",
		with: func(f *fakeHost) {
			f.members = []Member{{PID: 100, Argv: []string{"codex"}}}
		},
	}, {
		name: "the resolved root is not a directory",
		with: func(f *fakeHost) { f.dirs = nil },
	}, {
		name: "the OS will not say where home is",
		with: func(f *fakeHost) { f.home, f.homeErr = "", errors.New("no home") },
	}, {
		name: "the platform has no process reader",
		with: func(f *fakeHost) {
			f.errs[100] = ErrUnsupported
		},
	}} {
		t.Run(tc.name, func(t *testing.T) {
			f := oneClaude(home)
			tc.with(f)
			if got, ok := foreground(100, identifyByToken, f.host()); ok {
				t.Fatalf("resolved %+v, want unavailable", got)
			}
		})
	}
}

// Two claude tabs, same binary, same working directory, same adapter name, two
// configuration directories — the isolation two parallel accounts depend on,
// end to end through resolution rather than only through the pure table.
func TestForegroundSeparatesTwoConcurrentClaudeTabs(t *testing.T) {
	const home = "/home/operator"
	f := &fakeHost{
		facts: map[int]Info{
			100: {PID: 100, PGID: 100, Started: epoch, Dir: "/work/repo",
				Env: map[string]string{"CLAUDE_CONFIG_DIR": home + "/.claude"}},
			200: {PID: 200, PGID: 200, Started: epoch, Dir: "/work/repo",
				Env: map[string]string{"CLAUDE_CONFIG_DIR": home + "/.claude2"}},
		},
		dirs: map[string]bool{home + "/.claude": true, home + "/.claude2": true},
		home: home,
	}
	h := f.host()
	h.group = func(pgid int) []Member { return []Member{{PID: pgid, Argv: []string{"claude"}}} }

	first, ok := foreground(100, identifyByToken, h)
	if !ok {
		t.Fatal("first tab unavailable")
	}
	second, ok := foreground(200, identifyByToken, h)
	if !ok {
		t.Fatal("second tab unavailable")
	}
	if first.StateRoot == second.StateRoot {
		t.Fatalf("both tabs collapsed onto %q", first.StateRoot)
	}
	if first.StateRoot != home+"/.claude" || second.StateRoot != home+"/.claude2" {
		t.Fatalf("roots = %q and %q", first.StateRoot, second.StateRoot)
	}
}

// A tab chartr launched needs no identification — the launch already said which
// adapter runs — but it needs the same facts and the same allowlisted read.
func TestLaunchedResolvesWithoutIdentification(t *testing.T) {
	const home = "/home/operator"
	f := oneClaude(home)
	f.facts[100] = Info{PID: 100, PGID: 100, Started: epoch, Dir: "/work/repo",
		Env: map[string]string{"CLAUDE_CONFIG_DIR": "~/.claude2"}}
	f.dirs[home+"/.claude2"] = true

	got, ok := launched("claude", 100, f.host())
	if !ok {
		t.Fatal("unavailable")
	}
	if got.StateRoot != home+"/.claude2" {
		t.Fatalf("StateRoot = %q, want the tilde value expanded against home", got.StateRoot)
	}
	if got.Adapter != "claude" || got.PID != 100 {
		t.Fatalf("resolved %+v", got)
	}
}

func TestLaunchedUnavailableForAnUnknownAdapter(t *testing.T) {
	f := oneClaude("/home/operator")
	if got, ok := launched("codex", 100, f.host()); ok {
		t.Fatalf("resolved %+v, want unavailable until codex has a state-root row", got)
	}
}

// A relative configuration directory means what it meant to the process: a path
// under the directory that process is working in.
func TestForegroundNormalizesARelativeRoot(t *testing.T) {
	const home = "/home/operator"
	f := oneClaude(home)
	f.facts[100] = Info{PID: 100, PGID: 100, Started: epoch, Dir: "/work/repo",
		Env: map[string]string{"CLAUDE_CONFIG_DIR": ".claude-local"}}
	f.dirs["/work/repo/.claude-local"] = true

	got, ok := foreground(100, identifyByToken, f.host())
	if !ok {
		t.Fatal("unavailable")
	}
	if got.StateRoot != "/work/repo/.claude-local" {
		t.Fatalf("StateRoot = %q, want it resolved against the process working directory", got.StateRoot)
	}
}

// A resolved agent holds no environment at all — only the root the environment
// selected. That is what keeps an environment out of anything downstream logs,
// serializes or pushes to the browser: there is nothing there to leak. The check
// is made against both renderings a value like this reaches a log or a socket
// through, and it fails the day an Env field is added back.
func TestAgentCarriesNoEnvironment(t *testing.T) {
	const leaked = "sk-not-an-allowlisted-variable"
	f := oneClaude("/home/operator")
	f.facts[100] = Info{PID: 100, PGID: 100, Started: epoch, Dir: "/work/repo",
		// Only an allowlisted read ever populates Info.Env; a reader that
		// leaked would put something like this here.
		Env: map[string]string{"CLAUDE_CONFIG_DIR": "/home/operator/.claude", "ANTHROPIC_API_KEY": leaked},
	}

	got, ok := foreground(100, identifyByToken, f.host())
	if !ok {
		t.Fatal("unavailable")
	}
	body, err := json.Marshal(got)
	if err != nil {
		t.Fatal(err)
	}
	for name, rendered := range map[string]string{
		"Go form":   fmt.Sprintf("%#v", got),
		"JSON form": string(body),
	} {
		if strings.Contains(rendered, leaked) {
			t.Errorf("a resolved agent's %s carries a non-allowlisted value: %s", name, rendered)
		}
	}
}
