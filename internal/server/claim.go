package server

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
	"time"

	"github.com/rengwu/chartr/internal/prompt"
)

// claim is everything a claim's audit record carries: the session it claims for,
// the agent driving it, the hash of the exact payload that session was told, and
// the skills composed into it (with which layer won each).
//
// Execution is recorded as the *argv* rather than an agent-and-model pair: a
// model is a flag like any other, and the flags are what actually ran. `args`
// therefore says strictly more than a `model` field would — it carries the
// model where one was asked for, and the permission and sandbox flags beside it,
// which is exactly what an audit trail is read for.
type claim struct {
	SessionID string
	Role      string
	// Agent is the *registered agent's name* the operator chose. Adapter is the
	// binary that actually ran. Both travel: a local name says which of the
	// operator's agents was picked but means nothing on a teammate's machine, while
	// the adapter and args are what the trailer means anywhere (stories 30, 31).
	Agent      string
	Adapter    string
	Args       []string
	PayloadSHA string
	Skills     []prompt.Skill
}

// writeClaim is chartr's one lifecycle write at spawn: it stamps the ticket file
// with claimed_by/claimed_at so the ticket derives `claimed`, then records the
// spawn's provenance in the space's audit log. It runs no version-control command
// (ADR 0008, revised) — the ticket stamp is a plain working-tree edit and the
// audit line is an append, so the operator bundles both into their own first
// work commit whenever they commit, under whatever VCS they use (git, jujutsu,
// mercurial) or none at all. It handles a respawn onto a ticket that still
// carries a dead session's stale claim; stampClaim replaces it cleanly.
//
// The caller has already settled the agent and composed the payload, so this only
// records them. The returned error surfaces to the operator with nothing launched
// — a ticket on an unwritable volume, say.
func writeClaim(space, ticketPath, sessionID string, at string, c claim) error {
	rel, err := filepath.Rel(space, ticketPath)
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

	if err := appendAudit(space, claimEvent(rel, at, c)); err != nil {
		return fmt.Errorf("recording the claim in the audit log: %w", err)
	}
	return nil
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

// writeRelease is the death-halt's third choice: it releases a dead session's
// claim back to the frontier (ticket 10). It strips claimed_by/claimed_at from
// the ticket — so the ticket derives open and takeable again — and records the
// release in the space's audit log. Like the claim, it runs no version-control
// command (ADR 0008, revised): the ticket edit and the audit line ride into the
// operator's own next commit. The stale claim is cleared by an operator act,
// never on its own, and the audit log reads claim → release for the ticket.
func writeRelease(space, ticketPath, sessionID string) error {
	rel, err := filepath.Rel(space, ticketPath)
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
	at := time.Now().UTC().Format(time.RFC3339)
	if err := appendAudit(space, auditEvent{At: at, Event: "release", Ticket: rel, Session: sessionID}); err != nil {
		return fmt.Errorf("recording the release in the audit log: %w", err)
	}
	return nil
}

// auditEvent is one record in the space's audit log — the provenance a claim or
// release used to carry in its own VCS commit, now written to `.plan/audit.jsonl`
// (ADR 0008, revised). The log lives in the committed maps tree beside the tickets
// it records, so it travels into the operator's own commits with no chartr-driven
// VCS operation and under whatever VCS they use. One JSON object per line: a
// claim carries the full spawn provenance, a release carries only the ticket and
// the session whose stale claim was cleared.
type auditEvent struct {
	At      string `json:"at"`
	Event   string `json:"event"` // "claim" or "release"
	Ticket  string `json:"ticket"`
	Session string `json:"session"`
	// Claim-only provenance. Execution is the argv rather than an agent-and-model
	// pair: the flags are what actually ran. Omitted on a release.
	Role       string   `json:"role,omitempty"`
	Agent      string   `json:"agent,omitempty"`
	Adapter    string   `json:"adapter,omitempty"`
	Args       []string `json:"args,omitempty"`
	PayloadSHA string   `json:"payloadSHA,omitempty"`
	Skills     []string `json:"skills,omitempty"`
}

// claimEvent builds the audit record for a spawn — session, agent, adapter, args,
// role, the payload content hash, and one entry per composed skill, the same
// provenance the claim commit's trailers used to carry.
func claimEvent(rel, at string, c claim) auditEvent {
	skills := make([]string, 0, len(c.Skills))
	for _, sk := range c.Skills {
		skills = append(skills, skillProvenance(sk))
	}
	return auditEvent{
		At:         at,
		Event:      "claim",
		Ticket:     rel,
		Session:    c.SessionID,
		Role:       c.Role,
		Agent:      c.Agent,
		Adapter:    c.Adapter,
		Args:       c.Args,
		PayloadSHA: c.PayloadSHA,
		Skills:     skills,
	}
}

// skillProvenance renders one composed skill's provenance for the audit log.
//
// A skill reads `<name>=<source>`, with `@<commit>` appended where that source
// carries a pin — the commit is what makes the entry *fetchable*: a teammate
// reading the log can get the exact bytes the session was told. A `dir` source
// has no commit, and the entry honestly stops at the source's name; chartr's own
// embedded core reads `core=chartr` for the same reason (ADR 0017). The record's
// payloadSHA still fixes the exact bytes.
func skillProvenance(sk prompt.Skill) string {
	if sk.Commit == "" {
		return fmt.Sprintf("%s=%s", sk.Name, sk.Source)
	}
	return fmt.Sprintf("%s=%s@%s", sk.Name, sk.Source, sk.Commit)
}

// auditLogRel is the space-relative path of the audit log: the committed maps
// tree, beside the tickets whose claims and releases it records.
var auditLogRel = filepath.Join(".plan", "audit.jsonl")

// appendAudit appends one event as a JSON line to the space's audit log, creating
// `.plan/` and the file if absent. The append is the whole write — no rewrite, no
// truncation — so the log only ever grows and each spawn or release adds exactly
// one line for the operator's next commit to pick up. O_APPEND makes concurrent
// spawns' single-line writes safe without a lock.
func appendAudit(space string, ev auditEvent) error {
	line, err := json.Marshal(ev)
	if err != nil {
		return err
	}
	path := filepath.Join(space, auditLogRel)
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
	f, err := os.OpenFile(path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err != nil {
		return err
	}
	defer f.Close()
	if _, err := f.Write(append(line, '\n')); err != nil {
		return err
	}
	return nil
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
