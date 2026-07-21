package server

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// The gate's two lifecycle writes (ticket 12, ADR 0008). Beside the claim at
// spawn, these are the only commits the harness makes: the **promotion** at
// approval and the **demotion** at abandonment. Both hold the same discipline —
// edit exactly one ticket file, `git commit --only -- <ticket>` so a live
// session's staged work can never be swept in, structured trailers so git alone
// answers who blessed what, and never an amend, a reset, or a push.
//
// Promotion is its own commit rather than an amend of the session's (story 65):
// proposed-then-blessed stays visible and no SHA is ever rewritten. It also never
// waits on a live session — the narrow write is safe against the shared index —
// which leaves exactly one residual race: an agent's `git commit -a` sweeping our
// edit into *its* commit between the write and the commit. That degrades to an
// attribution smear, not a lost promotion, and the harness detects it (its own
// commit comes up empty) and reports it (ADR 0008).

// promoteAnswer rewrites a ticket's `## Proposed Answer` heading to `## Answer`,
// leaving the prose under it untouched — the promotion is a blessing, not an
// edit. The ticket then derives `resolved` (wayfinder reads closure from the
// heading), which is what unblocks its dependents on the stricter frontier.
// Reports false when the ticket carries no proposal to promote.
func promoteAnswer(src string) (string, bool) {
	lines := strings.Split(src, "\n")
	for i, l := range lines {
		if strings.EqualFold(strings.TrimSpace(l), "## Proposed Answer") {
			lines[i] = "## Answer"
			return strings.Join(lines, "\n"), true
		}
	}
	return src, false
}

// demoteProposal turns a rejected `## Proposed Answer` into dated `### Rejected`
// prose carrying the human's reason (story 60). The proposal's own text is kept
// verbatim beneath it — a failed attempt informs the next one rather than
// vanishing — and because the closing heading is gone the ticket derives `open`
// again and returns to the frontier.
//
// The demoted section is filed under a `## Rejected attempts` heading so a second
// abandonment stacks beneath the same parent rather than scattering loose `###`s
// through the ticket. Neither heading matches wayfinder's closure scan
// (`^## (Answer|Ruled out)`), so a vanilla tool reads the ticket as open too.
func demoteProposal(src, reason, date string) (string, bool) {
	lines := strings.Split(src, "\n")
	at := -1
	hasParent := false
	for i, l := range lines {
		t := strings.TrimSpace(l)
		if strings.EqualFold(t, rejectedAttemptsHeading) {
			hasParent = true
		}
		if at < 0 && strings.EqualFold(t, "## Proposed Answer") {
			at = i
		}
	}
	if at < 0 {
		return src, false
	}

	head := []string{fmt.Sprintf("### Rejected — %s", date)}
	if !hasParent {
		head = append([]string{rejectedAttemptsHeading, ""}, head...)
	}

	// The reason leads the demoted section: the next attempt reads why this one
	// failed before it reads what it tried.
	head = append(head, "", "**Why this was rejected:** "+strings.TrimSpace(reason), "",
		"The rejected proposal, kept verbatim:")

	out := append([]string{}, lines[:at]...)
	out = append(out, head...)
	out = append(out, lines[at+1:]...)
	return strings.Join(out, "\n"), true
}

const rejectedAttemptsHeading = "## Rejected attempts"

// gateCommit records one of the gate's lifecycle writes. Subject is the commit's
// human subject line; Trailers is the structured block beneath it, in order.
type gateCommit struct {
	Subject  string
	Trailers []trailer
}

type trailer struct{ Key, Value string }

func (g gateCommit) message() string {
	var b strings.Builder
	b.WriteString(g.Subject)
	b.WriteString("\n\n")
	for i, t := range g.Trailers {
		if i > 0 {
			b.WriteString("\n")
		}
		fmt.Fprintf(&b, "%s: %s", t.Key, t.Value)
	}
	return b.String()
}

// writeGateCommit applies next to the ticket file and commits *only* that path
// with the given message. It returns the new commit's sha, or — when the commit
// comes up empty — the smear report ADR 0008 promises.
//
// An empty commit here has exactly one meaning: between our write and our commit,
// something else (an agent's `git commit -a`, the operator's own) swept the edit
// into its commit. The content landed; only the attribution is wrong. That is
// surfaced honestly as `smeared` with the sha that actually carries the edit,
// rather than retried (a retry would commit nothing) or reported as a failure
// (the promotion did happen).
func writeGateCommit(repo, ticketPath, next string, gc gateCommit) (sha string, smearedInto string, err error) {
	rel, err := filepath.Rel(repo, ticketPath)
	if err != nil {
		return "", "", fmt.Errorf("locating ticket under the space: %w", err)
	}
	if err := os.WriteFile(ticketPath, []byte(next), 0o644); err != nil {
		return "", "", fmt.Errorf("writing the ticket: %w", err)
	}
	if out, err := git(repo, "add", "--", rel); err != nil {
		return "", "", fmt.Errorf("staging the gate write: %w\n%s", err, out)
	}
	out, err := git(repo, "commit", "--only", "-m", gc.message(), "--", rel)
	if err != nil {
		// Empty commit — the edit is already in history under someone else's name.
		// Confirm that before claiming a smear: if the working tree still differs
		// from HEAD, this is a real failure and must surface as one.
		if status, serr := git(repo, "--no-optional-locks", "status", "--porcelain", "--", rel); serr == nil && strings.TrimSpace(status) == "" {
			carrier, _ := git(repo, "log", "-1", "--format=%H", "--", rel)
			return "", strings.TrimSpace(carrier), nil
		}
		return "", "", fmt.Errorf("committing the gate write: %w\n%s", err, out)
	}
	head, err := git(repo, "rev-parse", "HEAD")
	if err != nil {
		return "", "", fmt.Errorf("reading the gate commit: %w\n%s", err, head)
	}
	return strings.TrimSpace(head), "", nil
}
