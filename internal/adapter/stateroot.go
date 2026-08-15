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

// stateRootSpec is one adapter's state-root knowledge: the environment variables
// that select the root, in precedence order, and the home-relative path that
// stands when none of them is set.
type stateRootSpec struct {
	vars []string
	// fallback is relative to the user's home, which is the only anchor every
	// one of these providers documents its default against.
	fallback string
}

// stateRoots is the table. A provider is listed only once its root has been
// observed first-hand on a real install — the same bar the delivery and
// generation tables hold, and for the same reason: a wrong root is a false
// positive that reads the wrong conversation, while a missing row is a tab that
// stays untitled.
//
// Codex, OpenCode, Pi and Grok are deliberately absent. Their stores are what
// ticket 04 of the transcript-backed-auto-titles map measures, and each lands
// here as a row when it does.
var stateRoots = map[string]stateRootSpec{
	// Claude Code keeps projects/, history and its session JSONL under
	// CLAUDE_CONFIG_DIR, defaulting to ~/.claude. This is the variable the
	// agent library's own documentation names for a second account.
	"claude": {vars: []string{"CLAUDE_CONFIG_DIR"}, fallback: ".claude"},
	// Kimi Code keeps its data under KIMI_CODE_HOME, defaulting to
	// ~/.kimi-code — the same root preflight.go writes a workspace-trust
	// marker into, measured on this host against Kimi Code 0.29.0.
	"kimi": {vars: []string{"KIMI_CODE_HOME"}, fallback: ".kimi-code"},
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
	return append([]string(nil), spec.vars...)
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
	for _, name := range spec.vars {
		// Set-but-empty reads as unset. An empty path is no path, and clearing
		// a variable by emptying it is how a launch environment says "use the
		// default" without being able to remove the entry.
		if v := env[name]; v != "" {
			return normalizeRoot(v, dir, home)
		}
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
func StateRootEnv(adapter, root string) []string {
	spec, ok := stateRoots[adapter]
	if !ok || len(spec.vars) == 0 || root == "" {
		return nil
	}
	return []string{spec.vars[0] + "=" + root}
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
