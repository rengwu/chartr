package prompt

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"

	"github.com/rengwu/wayfinder-harness/internal/config"
)

// Blocker is one of a ticket's blockers as the context bundle carries it: its
// number, title, and its resolved answer pulled inline (ADR 0005). The server
// fills these from the derived map, so composition needs no second load.
type Blocker struct {
	Num    int
	Title  string
	Answer string
}

// Steer is one block of live steering the operator attached to a session at the
// review gate (ticket 12): a blocking finding to clear, an advisory they chose to
// pass on, or a note in their own words. It rides the injected payload and its
// archive and nowhere else (story 59) — the ticket file is the permanent record
// and only abandonment writes there, so steering a live attempt and amending the
// ticket's history stay deliberately separate acts.
type Steer struct {
	Label string
	Text  string
}

// Bundle is the space- and ticket-specific material a payload is assembled from,
// gathered fresh at compose time (ADR 0005): the map body, this ticket, and its
// blockers' answers. MapDir anchors the review payload's spec lookup.
type Bundle struct {
	MapName     string
	MapBody     string
	MapDir      string
	TicketNum   int
	TicketTitle string
	TicketBody  string
	Blockers    []Blocker
	// Steering is the review gate's follow-up briefing, empty on a fresh spawn.
	Steering []Steer
}

// ComposeInput is everything Compose needs: the role, the config roots the prompt
// layers resolve from, and the assembled bundle.
type ComposeInput struct {
	Role    string
	DataDir string
	RepoDir string
	Bundle  Bundle
}

// Compose assembles the payload a session for this ticket and role would be told:
// the resolved core and role prompts, then the context bundle, then — for the
// review role only — the ticket's Done-when and the spec, guaranteed by assembly
// so the reviewer is never handed only a diff (story 53). It returns the parts
// with provenance, any behind-default warnings, and the single markdown document
// they render to.
func Compose(in ComposeInput) (Payload, error) {
	if !validRole(in.Role) {
		return Payload{}, fmt.Errorf("unknown role %q; want one of %v", in.Role, config.Roles)
	}

	var warnings []string
	parts := []Part{
		{Name: CorePart, Kind: "prompt", Segments: resolvePart(CorePart, in.DataDir, in.RepoDir, &warnings)},
		{Name: in.Role, Kind: "prompt", Segments: resolvePart(in.Role, in.DataDir, in.RepoDir, &warnings)},
	}

	// The context bundle, assembled fresh (ADR 0005): glossary, map body, this
	// ticket, and each blocker's answer inline.
	gloss, _ := baseText(in.DataDir, GlossaryPart)
	parts = append(parts,
		ctxPart("glossary", "Glossary", gloss),
		ctxPart("map", "Map: "+orDash(in.Bundle.MapName), in.Bundle.MapBody),
		ctxPart("ticket", fmt.Sprintf("Ticket #%02d — %s", in.Bundle.TicketNum, orDash(in.Bundle.TicketTitle)), in.Bundle.TicketBody),
	)
	for _, b := range in.Bundle.Blockers {
		answer := b.Answer
		if strings.TrimSpace(answer) == "" {
			answer = "_(no answer yet — this blocker is not resolved)_"
		}
		parts = append(parts, ctxPart(
			fmt.Sprintf("blocker #%02d", b.Num),
			fmt.Sprintf("Blocker #%02d — %s", b.Num, orDash(b.Title)),
			answer,
		))
	}

	// The gate's steering, last in the bundle so it reads as the most recent word
	// on an attempt the rest of the bundle already describes (ticket 12).
	for _, st := range in.Bundle.Steering {
		parts = append(parts, ctxPart(steerName(st.Label), st.Label, st.Text))
	}

	// The review payload always carries the Done-when and the spec by assembly.
	if in.Role == string(config.RoleReview) {
		dw := doneWhen(in.Bundle.TicketBody)
		if strings.TrimSpace(dw) == "" {
			// The guarantee is presence: if the ticket states its Done-when in a
			// shape the extractor does not recognise, fall back to the whole ticket
			// so the reviewer's contract is never silently dropped.
			dw = in.Bundle.TicketBody
		}
		parts = append(parts, ctxPart("done-when", "Done-when (the review contract)", dw))

		specText, specLabel := resolveSpec(in.Bundle.MapDir, in.Bundle.MapBody)
		parts = append(parts, ctxPart("spec", specLabel, specText))
	}

	return Payload{
		Role:      in.Role,
		TicketNum: in.Bundle.TicketNum,
		Parts:     parts,
		Warnings:  warnings,
		Markdown:  renderMarkdown(parts),
	}, nil
}

// AnswerSection returns a ticket's closing answer prose for inlining as a
// blocker's answer in the context bundle — its Answer, else its Proposed Answer,
// else its Ruled out. Empty when the blocker carries none, which Compose renders
// as an explicit "not resolved" note rather than a silent gap.
func AnswerSection(body string) string {
	return firstSection(body, "Answer", "Proposed Answer", "Ruled out")
}

// ProposedAnswerSection returns a ticket's `## Proposed Answer` prose verbatim —
// the work a review brief carries under review (ticket 11). Unlike AnswerSection
// it never falls back to a resolved Answer or a Ruled out: the brief judges a
// proposal specifically, so a ticket with no proposal returns empty and the caller
// refuses rather than review the wrong section.
func ProposedAnswerSection(body string) string {
	return firstSection(body, "Proposed Answer")
}

// steerName is a steering block's short part name in the preview's provenance
// list — the label, lowercased, so the operator recognises it there as the block
// they attached in the send-back dialog.
func steerName(label string) string {
	return "steering: " + strings.ToLower(strings.TrimSpace(label))
}

func ctxPart(name, label, text string) Part {
	return Part{
		Name:     name,
		Kind:     "context",
		Segments: []Segment{{Layer: LayerContext, Label: label, Text: strings.TrimRight(text, "\n")}},
	}
}

func orDash(s string) string {
	if strings.TrimSpace(s) == "" {
		return "—"
	}
	return s
}

// renderMarkdown is the single-document view of the payload: the prompt parts
// (core, then role) run first as the instruction, then a `# Context` section
// gathers every context part under its own heading. This is what would be written
// to the gitignored payload file and pointed at with a one-line opener (a later
// ticket); here it is what the preview shows and what the tests assert over.
func renderMarkdown(parts []Part) string {
	var prompts, context []Part
	for _, p := range parts {
		if p.Kind == "prompt" {
			prompts = append(prompts, p)
		} else {
			context = append(context, p)
		}
	}

	var b strings.Builder
	for i, p := range prompts {
		if i > 0 {
			b.WriteString("\n\n")
		}
		b.WriteString(partText(p))
	}
	if len(context) > 0 {
		b.WriteString("\n\n---\n\n# Context\n")
		for _, p := range context {
			b.WriteString("\n## ")
			b.WriteString(p.Segments[0].Label)
			b.WriteString("\n\n")
			b.WriteString(partText(p))
			b.WriteString("\n")
		}
	}
	return strings.TrimSpace(b.String()) + "\n"
}

// partText joins a part's non-empty segments with a blank line between them.
func partText(p Part) string {
	var segs []string
	for _, s := range p.Segments {
		if t := strings.TrimSpace(s.Text); t != "" {
			segs = append(segs, t)
		}
	}
	return strings.Join(segs, "\n\n")
}

var reForkMarker = regexp.MustCompile(`(?i)^\s*<!--\s*forked from\s+([0-9a-f]{4,64})\s*-->\s*\n?`)

// splitForkMarker peels an optional `<!-- forked from <hash> -->` first line off
// a replacement, returning the recorded fork hash (lowercased, "" if none) and
// the body with the marker stripped so it never reaches the payload.
func splitForkMarker(s string) (hash, body string) {
	m := reForkMarker.FindStringSubmatch(s)
	if m == nil {
		return "", s
	}
	return strings.ToLower(m[1]), s[len(m[0]):]
}

var reSpecLink = regexp.MustCompile(`\]\(([^)\s]*spec\.md)\)`)

// resolveSpec finds the spec the review payload must carry. An implementation
// map names its spec in its own body — a markdown link to a `spec.md` — so the
// spec is discovered from the map rather than a hard-coded path (the harness
// follows wayfinder's conventions, never wires a layout). When the link resolves
// to a readable file, that is the spec; when there is no link or it does not
// resolve, the map body stands in as the spec of record, so the guarantee still
// holds.
func resolveSpec(mapDir, mapBody string) (text, label string) {
	if m := reSpecLink.FindStringSubmatch(mapBody); m != nil && mapDir != "" {
		rel := m[1]
		p := rel
		if !filepath.IsAbs(p) {
			p = filepath.Join(mapDir, rel)
		}
		if b, err := os.ReadFile(p); err == nil {
			return string(b), "Spec (" + rel + ", carried for review)"
		}
	}
	return mapBody, "Spec (map body stands in — no linked spec.md, carried for review)"
}

// doneWhen extracts a ticket's Done-when. It prefers an explicit `## Done-when`
// (or `## Done when`) section; failing that it reads from the first line that
// opens with "Done when" / "Done-when" to the next blank line or heading — the
// prose shape wayfinder tickets actually use. Returns "" when neither is found,
// which Compose treats as "fall back to the whole ticket" so the contract is
// never dropped.
func doneWhen(body string) string {
	if s := firstSection(body, "Done-when", "Done when"); s != "" {
		return s
	}
	lines := strings.Split(body, "\n")
	for i, l := range lines {
		low := strings.ToLower(strings.TrimSpace(strings.TrimLeft(l, "*_> ")))
		if strings.HasPrefix(low, "done when") || strings.HasPrefix(low, "done-when") {
			out := []string{strings.TrimSpace(l)}
			for j := i + 1; j < len(lines); j++ {
				if strings.TrimSpace(lines[j]) == "" || strings.HasPrefix(lines[j], "#") {
					break
				}
				out = append(out, lines[j])
			}
			return strings.TrimSpace(strings.Join(out, "\n"))
		}
	}
	return ""
}

// firstSection returns the body under the first matching `## <name>` heading, up
// to the next `## ` heading. Case-insensitive on the heading text.
func firstSection(body string, names ...string) string {
	lines := strings.Split(body, "\n")
	for _, name := range names {
		want := "## " + name
		for i, l := range lines {
			if strings.EqualFold(strings.TrimSpace(l), want) {
				var out []string
				for j := i + 1; j < len(lines); j++ {
					if strings.HasPrefix(lines[j], "## ") {
						break
					}
					out = append(out, lines[j])
				}
				return strings.TrimSpace(strings.Join(out, "\n"))
			}
		}
	}
	return ""
}
