// Package proc reads the facts about a live process that transcript binding
// needs: which process a tab is actually running, when it started, where it is
// working, and the state root it persists its sessions under.
//
// It is the widening of the seam the activity sampler already uses. That one
// answers "is a known agent in this tab's foreground", which needs no more than
// the group's argv tokens; matching a tab to *one persisted conversation* needs
// the process itself — an identity stable against pid reuse (pid plus start
// time), the directory it is working in, and the root its provider writes
// sessions to.
//
// # The environment is read through an allowlist, never scanned
//
// A state root comes from the running agent's own environment, because that is
// the only place the truth is: an operator with `CLAUDE_CONFIG_DIR=~/.claude2
// claude` behind an alias never told chartr anything, and two such agents side
// by side must not be confused for each other. chartr does not go looking for
// roots — no scan of plausibly named directories, no setting to register one.
//
// A process environment is sensitive material, so the reader is handed the
// active adapter's allowlist (adapter.StateRootVars) and returns only those
// variables. The raw environment is discarded inside the platform reader, before
// it returns, and a resolved Agent carries no environment at all — only the root
// the environment selected. Nothing downstream can log, serialize or publish
// what it was never given.
//
// # Failing closed
//
// Every unreadable condition is the same condition: unavailable. An environment
// chartr cannot read never falls through to the provider's documented default,
// since a process that will not show its environment may well be the one running
// under a custom root; a group holding two adapters, or two indistinguishable
// copies of one, resolves to nothing rather than to a coin flip; and a root that
// is not a directory is not a root. A missing title costs nothing, while a wrong
// binding spends the operator's money on the wrong conversation.
//
// # Platforms
//
// macOS and Linux, matching the foreground-process seam. Elsewhere the reader
// reports ErrUnsupported and every resolution is unavailable — the build still
// compiles, and the cockpit is unchanged but for a title that never arrives.
package proc

import (
	"errors"
	"os"
	"time"

	"github.com/rengwu/chartr/internal/adapter"
)

// ErrUnsupported is what the reader returns on a platform with no process
// lookup. It is not special-cased anywhere — it resolves to unavailable exactly
// as a dead process or an unreadable environment does — and exists so a caller
// debugging a quiet feature can tell "this platform cannot" from "this process
// would not".
var ErrUnsupported = errors.New("proc: no process reader on this platform")

// Member is one process in a foreground process group: its identity and the argv
// tokens agent identification scores.
type Member struct {
	PID  int
	Argv []string
}

// Info is one process's facts as the platform reader returns them. Env holds the
// allowlisted variables the reader was asked for and nothing else — the raw
// environment never leaves the reader.
type Info struct {
	PID     int
	PGID    int
	Started time.Time
	Dir     string
	Env     map[string]string
}

// Agent is a tab's foreground agent, resolved: which adapter it runs, the
// process identity that stays stable against pid reuse (PID with Started), the
// directory it works in, and the validated state root it persists sessions
// under.
//
// It deliberately carries no environment. The root is the only thing the
// environment was read for, and a subprocess that must run under the same
// profile gets there through adapter.StateRootEnv rather than by holding a copy
// of what chartr read.
type Agent struct {
	Adapter   string
	PID       int
	PGID      int
	Started   time.Time
	Dir       string
	StateRoot string
}

// host is the seam resolution reads the world through: the platform process
// reader and the two filesystem questions a root is validated with. It exists so
// the selection and normalization rules are exercised on every platform,
// including the ones with no reader at all.
type host struct {
	group  func(pgid int) []Member
	lookup func(pid int, allow []string) (Info, error)
	isDir  func(path string) bool
	home   func() (string, error)
}

func realHost() host {
	return host{
		group:  Group,
		lookup: Lookup,
		isDir: func(path string) bool {
			st, err := os.Stat(path)
			return err == nil && st.IsDir()
		},
		home: os.UserHomeDir,
	}
}

// Foreground resolves the agent running in a tab's foreground process group.
// identify is the detection engine's own Identify: it names the agent an argv
// belongs to, or "" for none, and never lets a generic runtime speak for the
// agent it launched.
//
// It reports false whenever the answer is not unique and readable — no agent in
// the group, more than one, an unreadable process, an adapter chartr holds no
// state-root knowledge for, or a root that is not there.
func Foreground(pgid int, identify func(argv []string) string) (Agent, bool) {
	return foreground(pgid, identify, realHost())
}

// Launched resolves the agent in a tab chartr started itself, where the launch
// already said which adapter runs and pid is the process it started. It skips
// identification and nothing else: the same allowlisted read, the same
// normalization, the same validation.
func Launched(adapterName string, pid int) (Agent, bool) {
	return launched(adapterName, pid, realHost())
}

func foreground(pgid int, identify func(argv []string) string, h host) (Agent, bool) {
	if pgid <= 0 || identify == nil {
		return Agent{}, false
	}
	adapterName, pid, ok := pick(pgid, h.group(pgid), identify)
	if !ok {
		return Agent{}, false
	}
	return resolve(adapterName, pid, h)
}

func launched(adapterName string, pid int, h host) (Agent, bool) {
	if adapterName == "" || pid <= 0 {
		return Agent{}, false
	}
	return resolve(adapterName, pid, h)
}

// pick decides which process in a foreground group is the agent, in three steps,
// each of which exists for a case the others get wrong.
//
// First, candidates: every member whose argv names an agent. A group holding two
// different adapters is ambiguous and yields nothing — one of them would be the
// wrong conversation, and there is no rule that says which.
//
// Then, the exec test: a member whose *first* argv token alone names the agent is
// running the agent's binary, while one that merely mentions it in a later token
// is a wrapper — `sh -c claude` carries "claude" and is not claude. When any
// member passes, only those members remain. When none does the whole candidate
// set stands, which is what keeps a runtime-launched agent (`node
// /opt/bin/claude`, whose first token is the runtime) resolvable.
//
// Finally, the leader: among what is left, the group leader is the tab's own
// agent and anything else is passing through — an agent running a copy of itself
// must not hand the tab its child's identity. With no leader among them, one
// candidate is the answer and several are ambiguous.
func pick(pgid int, members []Member, identify func([]string) string) (string, int, bool) {
	var (
		adapterName string
		candidates  []Member
	)
	for _, m := range members {
		name := identify(m.Argv)
		if name == "" {
			continue
		}
		if adapterName != "" && name != adapterName {
			return "", 0, false // two vendors in one group: no binding
		}
		adapterName = name
		candidates = append(candidates, m)
	}
	if len(candidates) == 0 {
		return "", 0, false
	}

	var execs []Member
	for _, m := range candidates {
		if len(m.Argv) > 0 && identify(m.Argv[:1]) == adapterName {
			execs = append(execs, m)
		}
	}
	if len(execs) > 0 {
		candidates = execs
	}

	for _, m := range candidates {
		if m.PID == pgid {
			return adapterName, m.PID, true
		}
	}
	if len(candidates) == 1 {
		return adapterName, candidates[0].PID, true
	}
	return "", 0, false
}

// resolve turns an (adapter, pid) into the validated fact set, or into nothing.
// The allowlist the process environment is read through is the adapter's own and
// is never widened — an adapter with no state-root knowledge asks for no
// variables and resolves to unavailable, which is where the four providers
// awaiting measurement sit today.
func resolve(adapterName string, pid int, h host) (Agent, bool) {
	allow := adapter.StateRootVars(adapterName)
	if len(allow) == 0 {
		return Agent{}, false
	}
	info, err := h.lookup(pid, allow)
	if err != nil {
		return Agent{}, false
	}
	home, err := h.home()
	if err != nil {
		home = ""
	}
	root, ok := adapter.StateRoot(adapterName, info.Env, info.Dir, home)
	if !ok || !h.isDir(root) {
		return Agent{}, false
	}
	return Agent{
		Adapter:   adapterName,
		PID:       info.PID,
		PGID:      info.PGID,
		Started:   info.Started,
		Dir:       info.Dir,
		StateRoot: root,
	}, true
}
