package config

import (
	"fmt"
	"os"
	"strings"
)

// An agent's environment: variables set on the process before its binary
// runs — the other half of "how I am willing to run this harness" that
// flags can't express. `CLAUDE_CONFIG_DIR=~/.claude2 claude` is the
// motivating case: a second account, an alternate config root, a proxy —
// none of it expressible as a flag since it's environment the CLI reads,
// not anything it parses.
//
// As opaque as Args, for the same reason (ADR 0002): chartr knows what a
// variable *is* — name, equals sign, value — and nothing about what any
// name means to the harness reading it. No blessed-variable list, no
// per-CLI form.
//
// The one interpretation is a leading `~/` in the value, and it's
// load-bearing. Nothing expands a tilde but a shell, and there's no shell
// between chartr and the binary: passed raw, `CLAUDE_CONFIG_DIR=~/.claude2`
// makes the agent create a directory literally named `~` and start against
// a blank config — a silent failure that looks like a working launch.
// Expanding isn't a convenience here; it's the difference between the
// value meaning what it says and meaning nothing at all.
//
// Nothing else expands. A `$HOME` or `*` in a value reaches the agent
// exactly as typed — the moment this package interprets values it has to
// get a shell's whole grammar right, and it isn't a shell.

// resolveEnv validates and expands one agent's environment, dropping any
// entry it can't read and returning a warning for each. A malformed entry
// costs only itself — an agent with one bad variable still launches with
// its good ones.
func resolveEnv(name string, env []string, home string) ([]string, []string) {
	if len(env) == 0 {
		return nil, nil
	}
	var (
		out      []string
		warnings []string
	)
	for _, e := range env {
		key, value, err := splitEnv(e)
		if err != nil {
			warnings = append(warnings, fmt.Sprintf("agent %q has an unreadable environment entry: %s; it is ignored", name, err))
			continue
		}
		// PATH is worth a word: Go resolves the binary out of this
		// process's PATH before the child's environment is consulted, so
		// setting PATH here can't change which binary runs — only what the
		// agent finds once running. Occasionally wanted, so it's surfaced
		// and still passed, never refused.
		if key == "PATH" {
			warnings = append(warnings, fmt.Sprintf(
				"agent %q sets PATH; that does not change which binary chartr launches (it is resolved before the agent's environment applies), only what the agent finds once running", name))
		}
		out = append(out, key+"="+expandHome(value, home))
	}
	return out, warnings
}

// splitEnv reads one `KEY=VALUE` entry. The value may be anything at all,
// including empty — an empty value is a variable set to nothing, which is
// meaningfully different from one not set, and some CLIs read it that way.
func splitEnv(entry string) (string, string, error) {
	key, value, found := strings.Cut(entry, "=")
	if !found {
		return "", "", fmt.Errorf("%q has no `=`; an environment entry is KEY=VALUE", entry)
	}
	if key == "" {
		return "", "", fmt.Errorf("%q has no name before its `=`", entry)
	}
	// The name is checked, the value never is: a name with a space or `=`
	// can't be set by any means chartr has, so it's a typo caught here
	// rather than a variable that silently never arrives.
	if strings.ContainsAny(key, " \t\n\x00") {
		return "", "", fmt.Errorf("%q is not a variable name", key)
	}
	if strings.ContainsRune(entry, '\x00') {
		return "", "", fmt.Errorf("an environment entry cannot contain a NUL byte")
	}
	return key, value, nil
}

// expandHome replaces a leading `~/` — or a bare `~` — with home. Only at
// the front: `~` elsewhere is an ordinary character, and `~user` names
// someone else's home, which needs a passwd lookup this package won't do.
//
// With no home to expand to, the value is left exactly as typed — the
// operator sees the literal tilde in the command preview and can read what
// went wrong, unlike a silently swallowed path.
func expandHome(value, home string) string {
	if home == "" || !strings.HasPrefix(value, "~") {
		return value
	}
	switch {
	case value == "~":
		return home
	case strings.HasPrefix(value, "~/"):
		return home + value[1:]
	}
	return value
}

// homeDir is the home directory tilde expansion resolves against, empty
// when the OS won't say. Looked up per resolve rather than cached — resolves
// are rare, and a cached home can only be staler than one asked fresh.
func homeDir() string {
	h, err := os.UserHomeDir()
	if err != nil {
		return ""
	}
	return h
}

// ValidAgentEnv checks an environment the surface is about to write,
// refusing at the gate what resolve would otherwise drop with a warning.
// Values are unchecked and unexpanded — expansion happens on the way out
// (resolveEnv), so the file keeps reading as the shell line the operator
// had in mind.
func ValidAgentEnv(env []string) error {
	for _, e := range env {
		if _, _, err := splitEnv(e); err != nil {
			return err
		}
	}
	return nil
}
