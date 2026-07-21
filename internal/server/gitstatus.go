package server

// gitDirty reports whether the working tree at root carries uncommitted changes —
// anything `git status --porcelain` lists: modified, staged, or untracked files
// (gitignored paths, like the harness's own `run/` payloads, never count). It is
// the dirty badge (spec, Git and the gate; story 68): surfaced so the operator can
// judge whether a session's or a shell's debris is harmless, and never a spawn
// gate. A label, not a guarantee — a repo it cannot read reads clean rather than
// erroring, since the badge is advisory and its absence must never block a spawn.
//
// It runs under `--no-optional-locks`: `git status` normally takes `index.lock`
// to write back its refreshed stat cache, and this runs on every rebuild — which
// a lifecycle write itself triggers, by touching a ticket the `.plan/` watch is
// on. Without the flag the badge's read would race the gate's own `git add` for
// the lock and fail the commit. Nothing the harness reads for a badge is worth an
// index lock.
func gitDirty(root string) bool {
	out, err := git(root, "--no-optional-locks", "status", "--porcelain")
	if err != nil {
		return false
	}
	return out != ""
}
