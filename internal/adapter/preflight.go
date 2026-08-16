package adapter

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"time"
)

// This file is the adapter package's other half: host-side preparation an
// agent needs before its TUI comes up in a directory. Prompt delivery
// (adapter.go) decides how the opener reaches the agent; preflight decides
// what must already be true on the host for the agent to survive its own
// startup gates when no human is at the keyboard to answer them.
//
// The one gate that motivated it: Kimi Code's workspace-trust prompt. When a
// session starts in a folder Kimi has never seen trusted, its TUI stops on a
// "Trust this folder?" dialog whose default is "Don't trust" — and "Don't
// trust" exits the process. A chartr launch types its opener into that dialog
// (or simply never answers it), so the session dies before its first turn.
// Trusting is precisely what the operator already decided by launching an
// agent in the space, so chartr records the same marker Kimi's own TUI would
// have written, ahead of launch, and the dialog never appears.

// Preflight runs the launch-time preparation registered for an adapter, if
// any. dir is the working directory the agent's TUI will start in; env is the
// exact environment the launch will hand the process (already tilde-expanded),
// so an agent relocated by an env var — KIMI_CODE_HOME for kimi — is prepared
// where it will actually look. An unrecognised adapter has no preparation and
// returns nil. A failure is not fatal to the launch: the caller logs it and
// lets the agent fend for itself, which is exactly the pre-preflight
// behaviour.
func Preflight(adapterName, dir string, env []string) error {
	switch adapterName {
	case "kimi":
		return kimiTrustWorkspace(dir, env)
	}
	return nil
}

// kimiHome is where the launched kimi will keep its data. It is the same
// question the transcript reader asks of a *running* kimi, so it is the same
// table that answers it (stateroot.go) rather than a second copy of
// "KIMI_CODE_HOME, else ~/.kimi-code" that could drift from it. dir is the
// directory the agent will start in, which is what a relative value would mean.
func kimiHome(dir string, env []string) (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	root, ok := StateRoot("kimi", AllowedEnv(env, StateRootVars("kimi")), dir, home)
	if !ok {
		return "", fmt.Errorf("kimi's home cannot be resolved from its launch environment")
	}
	return root, nil
}

// kimiTrustWorkspace records Kimi's workspace-trust marker for dir, byte-for-byte
// what its TUI writes when the operator picks "Trust this folder": a document
// `{"root":…,"trustedAt":…}` named by the working-directory key under
// $KIMI_CODE_HOME/workspace-trust/. An existing marker stands — trust once
// given keeps its original timestamp. A host with no kimi home yet has nothing
// to prepare: kimi's first run builds its own house, and a home chartr
// half-created would only confuse it.
func kimiTrustWorkspace(dir string, env []string) error {
	home, err := kimiHome(dir, env)
	if err != nil {
		return err
	}
	if st, err := os.Stat(home); err != nil || !st.IsDir() {
		return nil
	}

	trustDir := filepath.Join(home, "workspace-trust")
	if err := os.MkdirAll(trustDir, 0o700); err != nil {
		return fmt.Errorf("creating kimi's workspace-trust directory: %w", err)
	}
	marker := filepath.Join(trustDir, kimiWorkDirKey(dir))
	if _, err := os.Stat(marker); err == nil {
		return nil
	}

	body, err := json.Marshal(struct {
		Root      string `json:"root"`
		TrustedAt int64  `json:"trustedAt"`
	}{Root: dir, TrustedAt: time.Now().UnixMilli()})
	if err != nil {
		return err
	}
	// Written the way kimi writes its own state: a temp file renamed into
	// place, so a crash mid-launch never leaves half a marker kimi would
	// misread.
	tmp, err := os.CreateTemp(trustDir, ".tmp-*")
	if err != nil {
		return err
	}
	defer os.Remove(tmp.Name())
	if _, err := tmp.Write(body); err != nil {
		tmp.Close()
		return err
	}
	if err := tmp.Chmod(0o600); err != nil {
		tmp.Close()
		return err
	}
	if err := tmp.Close(); err != nil {
		return err
	}
	if err := os.Rename(tmp.Name(), marker); err != nil {
		return fmt.Errorf("placing kimi's workspace-trust marker: %w", err)
	}
	return nil
}

// kimiWorkDirKey derives the key kimi derives for a working directory —
// `wd_<slug>_<first 12 hex of sha256(path)>` — matching the agent-core-v2
// engine's encodeWorkDirKey: backslashes become slashes, trailing slashes are
// stripped, and the last segment is slugified. The hash is over the path as
// given, never symlink-resolved, because that is what kimi hashes.
func kimiWorkDirKey(dir string) string {
	normalized := strings.TrimRight(strings.ReplaceAll(dir, `\`, "/"), "/")
	base := normalized
	if i := strings.LastIndex(normalized, "/"); i >= 0 {
		base = normalized[i+1:]
	}
	sum := sha256.Sum256([]byte(normalized))
	return "wd_" + kimiSlug(base) + "_" + hex.EncodeToString(sum[:])[:12]
}

var (
	kimiSlugChars  = regexp.MustCompile(`[^a-z0-9._-]+`)
	kimiSlugDashes = regexp.MustCompile(`^-+|-+$`)
)

// kimiSlug renders a directory name the way kimi's slugifyWorkDirName does:
// lowercased, unsafe runs folded to a dash, dashes trimmed, capped at forty
// characters — and "workspace" when nothing usable remains.
func kimiSlug(name string) string {
	slug := kimiSlugChars.ReplaceAllString(strings.ToLower(name), "-")
	slug = kimiSlugDashes.ReplaceAllString(slug, "")
	if len(slug) > 40 {
		slug = kimiSlugDashes.ReplaceAllString(slug[:40], "")
	}
	if slug == "" || slug == "." || slug == ".." {
		return "workspace"
	}
	return slug
}
