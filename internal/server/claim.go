package server

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"

	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/prompt"
)

// claim is everything a claim commit records: the session it claims for, the
// resolved binding driving it, the hash of the exact payload that session was
// told, and the skills composed into it. The layer provenance travels too, so the
// audit trail answers not just which agent ran but where in the config stack that
// choice was made — and, for content, which layer won each skill.
//
// The binding is recorded as the *argv* rather than an agent-and-model pair: a
// model is a flag like any other, and the flags are what actually ran. `Args:`
// therefore says strictly more than the `Model:` trailer it replaces — it carries
// the model where one was asked for, and the permission and sandbox flags beside
// it, which is exactly what an audit trail is read for.
type claim struct {
	SessionID   string
	Role        string
	Agent       string
	Args        []string
	PayloadSHA  string
	Skills      []prompt.Skill
	AdapterFrom config.Layer
	ArgsFrom    config.Layer
}

// writeClaimCommit is the chartr's one lifecycle write at spawn (ADR 0008): it
// stamps the ticket file with claimed_by/claimed_at so the ticket derives
// `claimed`, then commits *only that file* with structured trailers. The commit is
// pathspec-limited — `git commit --only -- <ticket>` builds it from HEAD plus the
// one path, so whatever else sits staged in the operator's index (their own work,
// a live session's edits) can never be swept into the chartr's claim. It handles
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
// it derives `claimed` (wayfinder reads the claim from these two keys). It first
// strips any existing claim keys, so re-stamping is idempotent — a respawn onto a
// ticket that already carries a dead session's stale claim replaces it cleanly
// (ticket 10), never doubling the keys. It adds them just inside the closing fence
// of an existing frontmatter block, preserving the operator's other keys and
// ordering; a ticket with no frontmatter at all (rare, a legacy loose-header
// ticket) gets a fresh block prepended so the claim is still expressible.
func stampClaim(src, sessionID, at string) string {
	fields := fmt.Sprintf("claimed_by: %s\nclaimed_at: %s\n", sessionID, at)

	lines := strings.Split(stripClaim(src), "\n")
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

// stripClaim removes the claimed_by/claimed_at keys from the ticket's frontmatter,
// so the ticket derives open again (wayfinder reads the claim from exactly these
// two keys). It touches only the frontmatter block — a `claimed_by:` that somehow
// appeared in the ticket body is left alone — and preserves every other key and
// the operator's ordering. Releasing a claim (ticket 10) is this plus a commit.
func stripClaim(src string) string {
	lines := strings.Split(src, "\n")
	if len(lines) == 0 || strings.TrimSpace(lines[0]) != "---" {
		return src
	}
	out := make([]string, 0, len(lines))
	out = append(out, lines[0])
	inFrontmatter := true
	for i := 1; i < len(lines); i++ {
		if inFrontmatter {
			if strings.TrimSpace(lines[i]) == "---" {
				inFrontmatter = false
				out = append(out, lines[i])
				continue
			}
			key := strings.TrimSpace(lines[i])
			if strings.HasPrefix(key, "claimed_by:") || strings.HasPrefix(key, "claimed_at:") {
				continue
			}
		}
		out = append(out, lines[i])
	}
	return strings.Join(out, "\n")
}

// writeReleaseCommit is the death-halt's third choice: it releases a dead
// session's claim back to the frontier (ticket 10). It strips claimed_by/claimed_at
// from the ticket — so the ticket derives open and takeable again — and commits
// *only that file*, the same pathspec-limited, never-amending, never-pushing
// discipline the claim uses (ADR 0008). Recording the release as its own commit
// keeps git the whole audit trail: the ticket's history reads claim → release, and
// the stale claim is cleared by an operator act, never on its own.
func writeReleaseCommit(repo, ticketPath, sessionID string) error {
	rel, err := filepath.Rel(repo, ticketPath)
	if err != nil {
		return fmt.Errorf("locating ticket under the space: %w", err)
	}
	src, err := os.ReadFile(ticketPath)
	if err != nil {
		return fmt.Errorf("reading ticket: %w", err)
	}
	if err := os.WriteFile(ticketPath, []byte(stripClaim(string(src))), 0o644); err != nil {
		return fmt.Errorf("clearing the claim on the ticket: %w", err)
	}
	if out, err := git(repo, "add", "--", rel); err != nil {
		return fmt.Errorf("staging the release: %w\n%s", err, out)
	}
	msg := fmt.Sprintf("Release %s back to the frontier\n\nSession: %s\nChartr-Write: true", rel, sessionID)
	if out, err := git(repo, "commit", "--only", "-m", msg, "--", rel); err != nil {
		return fmt.Errorf("committing the release: %w\n%s", err, out)
	}
	return nil
}

// claimMessage renders the claim commit's message: a human subject naming the
// ticket, then the trailer block ADR 0008 makes git the audit trail with —
// session, agent, model, role, the payload content hash, one `Skill:` line per
// composed skill (`<name>=<layer>:<hash>`, the provenance re-keyed from prompt
// parts to skills), and the layer each binding field resolved from. The trailers
// are a contiguous `Key: value` block so `git interpret-trailers` and
// `%(trailers)` parse them; a repeated key is legal and reads as a list.
func claimMessage(rel string, c claim) string {
	var b strings.Builder
	fmt.Fprintf(&b, "Claim %s for %s (%s)\n\n", rel, c.Role, c.Agent)
	fmt.Fprintf(&b, "Session: %s\n", c.SessionID)
	fmt.Fprintf(&b, "Agent: %s\n", c.Agent)
	if len(c.Args) > 0 {
		fmt.Fprintf(&b, "Args: %s\n", strings.Join(c.Args, " "))
	}
	fmt.Fprintf(&b, "Role: %s\n", c.Role)
	fmt.Fprintf(&b, "Payload-SHA256: %s\n", c.PayloadSHA)
	for _, sk := range c.Skills {
		fmt.Fprintf(&b, "Skill: %s=%s:%s\n", sk.Name, sk.Layer, sk.Hash)
	}
	fmt.Fprintf(&b, "Adapter-From: %s\n", c.AdapterFrom)
	fmt.Fprintf(&b, "Args-From: %s\n", c.ArgsFrom)
	fmt.Fprintf(&b, "Chartr-Write: true")
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
