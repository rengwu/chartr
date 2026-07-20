package server

// gitDirty reports whether the working tree at root carries uncommitted changes —
// anything `git status --porcelain` lists: modified, staged, or untracked files
// (gitignored paths, like the harness's own `run/` payloads, never count). It is
// the dirty badge (spec, Git and the gate; story 68): surfaced so the operator can
// judge whether a session's or a shell's debris is harmless, and never a spawn
// gate. A label, not a guarantee — a repo it cannot read reads clean rather than
// erroring, since the badge is advisory and its absence must never block a spawn.
func gitDirty(root string) bool {
	out, err := git(root, "status", "--porcelain")
	if err != nil {
		return false
	}
	return out != ""
}
