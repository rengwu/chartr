package config

import (
	"fmt"
	"os"
	"strings"
)

// An agent's environment: the variables set on the process before its binary
// runs, which is the other half of "how I am willing to run this harness" that
// flags alone cannot express. `CLAUDE_CONFIG_DIR=~/.claude2 claude` is the
// motivating case — a second account, an alternate config root, a proxy — and none
// of it is expressible as a flag, because it is the environment the CLI reads
// rather than anything it parses.
//
// It is as opaque as Args, for the same reason (ADR 0002): chartr knows what a
// variable *is* — a name, an equals sign, a value — and nothing whatever about
// what any particular name means to the harness that reads it. There is no list of
// blessed variables and no per-CLI form.
//
// The one interpretation is a leading `~/` in the value, and it is load-bearing.
// Nothing expands a tilde but a shell, and there is no shell between chartr and
// the binary: passed through raw, `CLAUDE_CONFIG_DIR=~/.claude2` makes the agent
// create a directory literally named `~` inside the space and start against a
// blank config. That failure is silent and looks exactly like a working launch,
// which is precisely the kind an operator cannot debug from the outside. Expanding
// is not a convenience here; it is the difference between the value meaning what
// it plainly says and meaning nothing at all.
//
// Nothing else expands. A `$HOME` or a `*` in a value reaches the agent exactly as
// typed, because the moment this package starts interpreting values it has to be
// right about a shell's whole grammar, and it is not a shell.

// resolveEnv validates and expands one agent's environment, dropping an entry it
// cannot read and returning a warning for each. A malformed entry costs only
// itself: an agent with one bad variable still launches with its good ones, the
// same way the library keeps its good agents when one table is bad.
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
		// PATH is the one name worth a word, because setting it here does nothing an
		// operator would predict: Go resolves the binary out of *this* process's PATH
		// before the child's environment is ever consulted (the env package), so a
		// PATH set on an agent cannot change which binary runs — only what the agent
		// finds once it is running. That is occasionally what someone wants, so it is
		// surfaced and still passed, never refused (the library warns, it does not
		// enforce).
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
	// The name is checked, the value never is. A name with a space or an equals
	// sign in it cannot be set by any means chartr has, so it is a typo caught here
	// rather than a variable that silently never arrives.
	if strings.ContainsAny(key, " \t\n\x00") {
		return "", "", fmt.Errorf("%q is not a variable name", key)
	}
	if strings.ContainsRune(entry, '\x00') {
		return "", "", fmt.Errorf("an environment entry cannot contain a NUL byte")
	}
	return key, value, nil
}

// expandHome replaces a leading `~/` — and a bare `~` — with home. Only at the
// front, and only when what follows is a separator or nothing: a `~` anywhere else
// in a value is an ordinary character, and a `~user` form names someone else's
// home, which requires a passwd lookup this package has no business doing.
//
// With no home to expand to, the value is left exactly as typed. That is the
// honest failure: the operator sees the literal tilde in the command preview and
// can read what went wrong, which a silently swallowed path would not give them.
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

// homeDir is the home directory tilde expansion resolves against, empty when the
// OS will not say. It is looked up per resolve rather than cached: resolves are
// rare, and a cached home is a value that can only ever be staler than the one
// asked for fresh.
func homeDir() string {
	h, err := os.UserHomeDir()
	if err != nil {
		return ""
	}
	return h
}

// ValidAgentEnv checks an environment the surface is about to write, refusing at
// the gate what resolve would otherwise have to drop with a warning. Values are
// unchecked and unexpanded: what the operator typed is what is stored, and the
// expansion happens on the way out (resolveEnv), so the file keeps reading as the
// shell line they had in mind.
func ValidAgentEnv(env []string) error {
	for _, e := range env {
		if _, _, err := splitEnv(e); err != nil {
			return err
		}
	}
	return nil
}
