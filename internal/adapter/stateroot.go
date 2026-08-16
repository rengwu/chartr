package adapter

import (
	"path/filepath"
	"strings"
)

// This file is the adapter package's fourth kind of per-agent knowledge, beside
// prompt delivery (adapter.go), launch preflight (preflight.go) and headless
// generation (titlegen.go): where a *running* agent persists its sessions, and
// which environment variables select that place.
//
// Transcript-backed titling has to match a live tab to one persisted
// conversation, which means knowing the state root the foreground process is
// actually using — not the one chartr would have used. An operator running
// `CLAUDE_CONFIG_DIR=~/.claude2 claude` from an alias gets a working feature
// without telling chartr anything, and two such agents side by side stay
// isolated, because the root is read out of each process's own environment.
//
// Two rules make that safe rather than clever:
//
//   - The environment is read through an allowlist, never scanned. An adapter
//     names the variables that may be looked at; everything else about the
//     process's environment is discarded by the reader before it returns
//     (internal/proc). This table *is* that allowlist.
//   - chartr never goes looking for a root. There is no directory scan for
//     plausibly named state directories and no setting to register one: a
//     variable is set and names the root, or it is unset and the provider's
//     documented default stands. Anything else is unavailable, which costs a
//     title and nothing more.
//
// Like the other three, it is pure data plus one small resolver: teaching chartr
// a new provider's root is a row, never a change to the caller.

// stateRootVar is one environment variable that selects a root, plus the fixed
// segment the provider appends to it.
//
// Most variables name the root outright and carry no suffix. Pi does not:
// PI_CODING_AGENT_DIR names the parent of its `sessions` directory. That is the
// provider's own path arithmetic, so it belongs beside the variable rather than
// in a caller that would have to know which providers are special.
type stateRootVar struct {
	name   string
	suffix string
}

// stateRootSpec is one adapter's state-root knowledge: the environment variables
// that select the root, in precedence order, and the home-relative path that
// stands when none of them is set.
type stateRootSpec struct {
	vars []stateRootVar
	// fallback is relative to the user's home, which is the only anchor every
	// one of these providers documents its default against. It already includes
	// whatever suffix the variables carry, since a default is the whole path.
	fallback string
}

// stateRoots is the table. A provider is listed only once its root has been
// observed first-hand on a real install — the same bar the delivery and
// generation tables hold, and for the same reason: a wrong root is a false
// positive that reads the wrong conversation, while a missing row is a tab that
// stays untitled.
//
// A "root" here is the directory a transcript adapter builds its paths from,
// which is not always the provider's own idea of its home: Pi's root is the
// sessions directory rather than the agent directory, because that is what its
// two variables and its default all agree on naming.
var stateRoots = map[string]stateRootSpec{
	// Claude Code keeps projects/, history and its session JSONL under
	// CLAUDE_CONFIG_DIR, defaulting to ~/.claude. This is the variable the
	// agent library's own documentation names for a second account.
	"claude": {vars: []stateRootVar{{name: "CLAUDE_CONFIG_DIR"}}, fallback: ".claude"},
	// Codex keeps sessions/<Y>/<M>/<D>/rollout-*.jsonl under CODEX_HOME,
	// defaulting to ~/.codex — the root its own --config help names. Observed
	// against codex-cli 0.147.0.
	"codex": {vars: []stateRootVar{{name: "CODEX_HOME"}}, fallback: ".codex"},
	// Pi resolves its session directory in a documented order:
	// PI_CODING_AGENT_SESSION_DIR names it outright, PI_CODING_AGENT_DIR names
	// the agent directory it sits under, and the default is
	// ~/.pi/agent/sessions. A --session-dir flag and a settings.json sessionDir
	// also exist and are invisible to an environment reader; an operator using
	// either simply gets no candidate, which costs a title and nothing else.
	// Observed against pi 0.78.0.
	"pi": {
		vars: []stateRootVar{
			{name: "PI_CODING_AGENT_SESSION_DIR"},
			{name: "PI_CODING_AGENT_DIR", suffix: "sessions"},
		},
		fallback: ".pi/agent/sessions",
	},
	// Kimi Code keeps its data under KIMI_CODE_HOME, defaulting to
	// ~/.kimi-code — the same root preflight.go writes a workspace-trust
	// marker into, measured on this host against Kimi Code 0.29.0 and
	// unchanged at 0.36.1.
	"kimi": {vars: []stateRootVar{{name: "KIMI_CODE_HOME"}}, fallback: ".kimi-code"},
	// Grok keeps sessions/<percent-encoded-cwd>/<uuid>/ under GROK_HOME,
	// defaulting to ~/.grok, which its shipped README states outright.
	// Observed against grok 1.0.0.
	"grok": {vars: []stateRootVar{{name: "GROK_HOME"}}, fallback: ".grok"},
}

// StateRootVars returns the environment variables that select adapter's state
// root, in precedence order — the whole of what a process-environment reader is
// permitted to keep, and the argument every reader takes. Nil for an adapter
// chartr has no state-root knowledge of, which reads as "keep nothing".
//
// The slice is a copy: an allowlist a caller could sort, truncate or overwrite
// in place would be an allowlist that means something different on the next
// lookup.
func StateRootVars(adapter string) []string {
	spec, ok := stateRoots[adapter]
	if !ok || len(spec.vars) == 0 {
		return nil
	}
	names := make([]string, 0, len(spec.vars))
	for _, v := range spec.vars {
		names = append(names, v.name)
	}
	return names
}

// AllowedEnv narrows a `KEY=VALUE` environment down to allow, dropping every
// other variable. It is the chokepoint the allowlist rule is enforced at: a
// reader that has just pulled a whole process environment out of the kernel
// calls this and keeps only the result, so the raw environment is gone before
// anything can log, serialize or return it. Entries with no `=` and variables
// outside allow are discarded; a repeated variable keeps its last value, which
// is what execve's own lookup does.
//
// Nil when nothing was allowed or nothing matched, so a caller never has to tell
// "no allowlist" from "allowlist matched nothing" — both mean the same to
// StateRoot.
func AllowedEnv(env, allow []string) map[string]string {
	if len(env) == 0 || len(allow) == 0 {
		return nil
	}
	want := make(map[string]bool, len(allow))
	for _, name := range allow {
		want[name] = true
	}
	var kept map[string]string
	for _, entry := range env {
		key, value, found := strings.Cut(entry, "=")
		if !found || !want[key] {
			continue
		}
		if kept == nil {
			kept = make(map[string]string, len(allow))
		}
		kept[key] = value
	}
	return kept
}

// StateRoot resolves where a running adapter process persists its sessions.
//
// env is what an allowlisted read of that process's environment returned —
// StateRootVars(adapter) and nothing else. dir is the process's own working
// directory, which is what a relative value is relative to, and home is the
// user's home, which is what a `~` expands to and what the documented default
// hangs off. The result is always absolute and cleaned, so a caller never has to
// normalize a root a second time before reading from it.
//
// It reports false rather than guessing: an adapter with no row, a value naming
// another user's home, a relative value with no working directory to resolve it
// against, and a default with no home are all unavailable. Pure — validating
// that the resolved root exists is the caller's business, since only the caller
// knows whether it is about to read from it.
func StateRoot(adapter string, env map[string]string, dir, home string) (string, bool) {
	spec, ok := stateRoots[adapter]
	if !ok {
		return "", false
	}
	for _, v := range spec.vars {
		// Set-but-empty reads as unset. An empty path is no path, and clearing
		// a variable by emptying it is how a launch environment says "use the
		// default" without being able to remove the entry.
		value := env[v.name]
		if value == "" {
			continue
		}
		root, ok := normalizeRoot(value, dir, home)
		if !ok {
			return "", false
		}
		// The provider's own arithmetic, applied after normalization so a
		// relative or user-relative value reaches the same place either way.
		return filepath.Join(root, v.suffix), true
	}
	if home == "" {
		return "", false
	}
	return filepath.Join(home, spec.fallback), true
}

// StateRootEnv renders the `KEY=VALUE` entries that put a subprocess on root —
// what a title generation must be given so it runs under the same account and
// state root as the live agent it is summarising, rather than inheriting
// whichever profile chartr itself was started under.
//
// It is deliberately how a resolved root reaches a subprocess, instead of a copy
// of the environment that was read: the first declared variable set to the
// already-normalized root says exactly the same thing to the provider, and
// nothing that was read has to be retained to say it. Nil for an adapter with no
// state-root knowledge, or an empty root — in both cases there is nothing to
// select and the caller should not be inventing one.
//
// A variable that names a parent is given the parent, since that is what the
// provider will append its own segment to. A root that does not end in the
// segment the provider would have added cannot be expressed through that
// variable at all, and rendering it anyway would put the subprocess on a
// different root than the tab it is summarising — so it resolves to nothing.
func StateRootEnv(adapter, root string) []string {
	spec, ok := stateRoots[adapter]
	if !ok || len(spec.vars) == 0 || root == "" {
		return nil
	}
	v := spec.vars[0]
	value := root
	if v.suffix != "" {
		parent, last := filepath.Split(filepath.Clean(root))
		if last != v.suffix || parent == "" {
			return nil
		}
		value = filepath.Clean(parent)
	}
	return []string{v.name + "=" + value}
}

// normalizeRoot turns one raw environment value into the absolute, cleaned path
// the provider itself would have used.
//
// A leading `~/` — or a bare `~` — expands to home, exactly as expandHome does
// for a launch environment (config/agentenv.go), and for the same reason: the
// value can reach the process with the tilde intact, since chartr sets some of
// these itself and no shell stands between it and the binary. `~user` names
// someone else's home and needs a passwd lookup this package will not do, so it
// is unavailable rather than mangled into a relative path.
//
// Anything still relative is resolved against the process's working directory,
// which is how the process itself would have read it.
func normalizeRoot(value, dir, home string) (string, bool) {
	switch {
	case value == "~", strings.HasPrefix(value, "~/"):
		if home == "" {
			return "", false
		}
		return filepath.Clean(home + value[1:]), true
	case strings.HasPrefix(value, "~"):
		return "", false
	case filepath.IsAbs(value):
		return filepath.Clean(value), true
	case dir == "":
		return "", false
	}
	return filepath.Join(dir, value), true
}
