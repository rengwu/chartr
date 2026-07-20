package server

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"

	"github.com/rengwu/wayfinder-harness/internal/config"
)

// claim is everything a claim commit records: the session it claims for, the
// resolved binding driving it, and the hash of the exact payload that session was
// told. The layer provenance travels too, so the audit trail answers not just
// which agent and model ran but where in the config stack that choice was made.
type claim struct {
	SessionID   string
	Role        string
	Agent       string
	Model       string
	PayloadSHA  string
	AdapterFrom config.Layer
	ModelFrom   config.Layer
	ArgsFrom    config.Layer
}

// writeClaimCommit is the harness's one lifecycle write at spawn (ADR 0008): it
// stamps the ticket file with claimed_by/claimed_at so the ticket derives
// `claimed`, then commits *only that file* with structured trailers. The commit is
// pathspec-limited — `git commit --only -- <ticket>` builds it from HEAD plus the
// one path, so whatever else sits staged in the operator's index (their own work,
// a live session's edits) can never be swept into the harness's claim. It handles
// a not-yet-tracked ticket (a freshly charted map) and an unborn HEAD (a first
// commit); the caller has already resolved the binding and composed the payload,
// so this only records them.
//
// It never rewrites history and never pushes (ADR 0008): the claim is a new
// commit, full stop. The returned error surfaces to the operator — a repo with no
// git identity, say — with nothing launched.
func writeClaimCommit(repo, ticketPath, sessionID string, at string, c claim) error {
	rel, err := filepath.Rel(repo, ticketPath)
	if err != nil {
		return fmt.Errorf("locating ticket under the space: %w", err)
	}

	src, err := os.ReadFile(ticketPath)
	if err != nil {
		return fmt.Errorf("reading ticket: %w", err)
	}
	next := stampClaim(string(src), sessionID, at)
	if err := os.WriteFile(ticketPath, []byte(next), 0o644); err != nil {
		return fmt.Errorf("stamping the claim onto the ticket: %w", err)
	}

	// Stage then partial-commit the one path. `git add` makes an untracked ticket
	// known to git; `--only -- <path>` commits from HEAD plus that path alone,
	// leaving the rest of the index untouched.
	if out, err := git(repo, "add", "--", rel); err != nil {
		return fmt.Errorf("staging the claim: %w\n%s", err, out)
	}
	if out, err := git(repo, "commit", "--only", "-m", claimMessage(rel, c), "--", rel); err != nil {
		return fmt.Errorf("committing the claim: %w\n%s", err, out)
	}
	return nil
}

// git runs one git command in repo and returns its combined output. It shells out
// (rather than reading .git directly, as the branch label does) because a commit
// must honour the operator's git config — identity, hooks, signing — exactly as
// their own commits do.
func git(repo string, args ...string) (string, error) {
	cmd := exec.Command("git", args...)
	cmd.Dir = repo
	out, err := cmd.CombinedOutput()
	return strings.TrimSpace(string(out)), err
}

// stampClaim inserts claimed_by/claimed_at into the ticket's YAML frontmatter so
// it derives `claimed` (wayfinder reads the claim from these two keys). It adds
// them just inside the closing fence of an existing frontmatter block, preserving
// the operator's other keys and ordering; a ticket with no frontmatter at all
// (rare, a legacy loose-header ticket) gets a fresh block prepended so the claim
// is still expressible.
func stampClaim(src, sessionID, at string) string {
	fields := fmt.Sprintf("claimed_by: %s\nclaimed_at: %s\n", sessionID, at)

	lines := strings.Split(src, "\n")
	if len(lines) > 0 && strings.TrimSpace(lines[0]) == "---" {
		for i := 1; i < len(lines); i++ {
			if strings.TrimSpace(lines[i]) == "---" {
				// Insert the claim keys on the line before the closing fence.
				out := append([]string{}, lines[:i]...)
				out = append(out, "claimed_by: "+sessionID, "claimed_at: "+at)
				out = append(out, lines[i:]...)
				return strings.Join(out, "\n")
			}
		}
	}
	// No frontmatter block — prepend one.
	return "---\n" + fields + "---\n\n" + src
}

// claimMessage renders the claim commit's message: a human subject naming the
// ticket, then the trailer block ADR 0008 makes git the audit trail with —
// session, agent, model, role, the payload content hash, and the layer each
// binding field resolved from. The trailers are a contiguous `Key: value` block so
// `git interpret-trailers` and `%(trailers)` parse them.
func claimMessage(rel string, c claim) string {
	var b strings.Builder
	fmt.Fprintf(&b, "Claim %s for %s (%s · %s)\n\n", rel, c.Role, c.Agent, c.Model)
	fmt.Fprintf(&b, "Session: %s\n", c.SessionID)
	fmt.Fprintf(&b, "Agent: %s\n", c.Agent)
	fmt.Fprintf(&b, "Model: %s\n", c.Model)
	fmt.Fprintf(&b, "Role: %s\n", c.Role)
	fmt.Fprintf(&b, "Payload-SHA256: %s\n", c.PayloadSHA)
	fmt.Fprintf(&b, "Adapter-From: %s\n", c.AdapterFrom)
	fmt.Fprintf(&b, "Model-From: %s\n", c.ModelFrom)
	fmt.Fprintf(&b, "Args-From: %s", c.ArgsFrom)
	return b.String()
}

var reTicketFile = regexp.MustCompile(`^(\d+)-(.+)\.md$`)

// ticketFilePath finds the on-disk file for a ticket number under a map's
// tickets/ directory, tolerant of zero-padding (09 vs 9) since the number alone
// does not fix the filename. It returns an error when no file matches, which the
// spawn path turns into a 404.
func ticketFilePath(mapDir string, num int) (string, error) {
	dir := filepath.Join(mapDir, "tickets")
	entries, err := os.ReadDir(dir)
	if err != nil {
		return "", fmt.Errorf("no tickets directory for this map: %w", err)
	}
	for _, e := range entries {
		if e.IsDir() {
			continue
		}
		m := reTicketFile.FindStringSubmatch(e.Name())
		if m == nil {
			continue
		}
		n, convErr := strconv.Atoi(strings.TrimLeft(m[1], "0"))
		if convErr == nil && n == num {
			return filepath.Join(dir, e.Name()), nil
		}
	}
	return "", fmt.Errorf("no file for ticket %d under %s", num, dir)
}
