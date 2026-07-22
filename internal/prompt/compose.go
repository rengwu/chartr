package prompt

import (
	"fmt"
	"path/filepath"
	"strings"

	"github.com/rengwu/chartr/internal/config"
)

// Blocker is one of a ticket's blockers as the context bundle carries it: its
// number, title, and its resolved answer pulled inline (ADR 0005). The server
// fills these from the derived map, so composition needs no second load.
type Blocker struct {
	Num    int
	Title  string
	Answer string
}

// Bundle is the space- and ticket-specific material a payload is assembled from,
// gathered fresh at compose time (ADR 0005): the map body, this ticket, and its
// blockers' answers.
type Bundle struct {
	MapName     string
	MapBody     string
	TicketNum   int
	TicketTitle string
	TicketBody  string
	Blockers    []Blocker
}

// ComposeInput is everything Compose needs: the role, the skill-library roots
// resolution walks, and the assembled bundle.
type ComposeInput struct {
	Role   string
	Roots  Roots
	Bundle Bundle
}

// Compose assembles the payload a session for this ticket and role would be told:
// the resolved core and role skill bodies (frontmatter stripped), then the
// context bundle. It returns the parts with provenance, the skills that won,
// any stale-fork warnings, and the single markdown document they render to.
func Compose(in ComposeInput) (Payload, error) {
	if !validRole(in.Role) {
		return Payload{}, fmt.Errorf("unknown role %q; want one of %v", in.Role, config.Roles)
	}

	var warnings []string
	var skills []Skill
	var parts []Part
	for _, name := range []string{CoreSkill, in.Role} {
		s, ok := Resolve(name, in.Roots)
		if !ok {
			return Payload{}, fmt.Errorf("no %q skill in any layer", name)
		}
		if s.Stale {
			warnings = append(warnings, staleWarning(s))
		}
		skills = append(skills, s)
		parts = append(parts, Part{
			Name: name, Kind: "prompt",
			Segments: []Segment{{Layer: s.Layer, Label: "skill", Text: s.Body}},
		})
	}

	// The context bundle, assembled fresh (ADR 0005): the glossary sourced from
	// the resolved `tracker-convention` skill, the skill-library manifest, the
	// map body, this ticket, and each blocker's answer inline. The bundle is
	// composed, never a skill: the ticket a session was handed must not be
	// mistaken for durable skill content.
	var gloss string
	if tc, ok := Resolve(TrackerSkill, in.Roots); ok {
		if g, ok := tc.Support(GlossaryFile); ok {
			gloss = g
		}
		if tc.Stale {
			warnings = append(warnings, staleWarning(tc))
		}
	}
	parts = append(parts,
		ctxPart("glossary", "Glossary", gloss),
		ctxPart("skill-library", "Skill library", skillManifest(in.Roots)),
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

	return Payload{
		Role:      in.Role,
		TicketNum: in.Bundle.TicketNum,
		Parts:     parts,
		Skills:    skills,
		Warnings:  warnings,
		Markdown:  renderMarkdown(parts),
	}, nil
}

// AnswerSection returns a ticket's closing answer prose for inlining as a
// blocker's answer in the context bundle — its Answer, else its Ruled out.
// Empty when the blocker carries none, which Compose renders as an explicit
// "not resolved" note rather than a silent gap. An in-flight `## Proposed
// Answer` is deliberately *not* read: it is an unknown heading no one blessed,
// and handing it to a dependent as though it were the answer is the one failure
// this narrowing exists to prevent.
func AnswerSection(body string) string {
	return firstSection(body, "Answer", "Ruled out")
}

func ctxPart(name, label, text string) Part {
	return Part{
		Name:     name,
		Kind:     "context",
		Segments: []Segment{{Layer: LayerContext, Label: label, Text: strings.TrimRight(text, "\n")}},
	}
}

// skillManifest renders the library a session may reach for: every shipped
// skill's name, its one-line use, and the path its winning layer sits at — so
// when the map's Notes or a ticket names a skill, the session can read its
// SKILL.md off disk. Existence and reach only, never content: the bodies stay
// out of the payload, and a stale fork is surfaced through LibraryWarnings,
// not here.
func skillManifest(roots Roots) string {
	var b strings.Builder
	b.WriteString("The skills the chartr ships. When the map or your ticket names one, read its `SKILL.md` under the path shown and apply it — do not work from memory of this list.")
	for _, name := range Names() {
		s, ok := Resolve(name, roots)
		if !ok {
			continue
		}
		desc := s.Description
		if desc == "" {
			desc = "(no description)"
		}
		dir := s.Dir
		if dir == "" && roots.Builtin != "" {
			// The embedded floor is materialized under the built-in root.
			dir = filepath.Join(roots.Builtin, name)
		}
		if dir != "" {
			fmt.Fprintf(&b, "\n- `%s` — %s (`%s`)", name, desc, dir)
		} else {
			fmt.Fprintf(&b, "\n- `%s` — %s", name, desc)
		}
	}
	return b.String()
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
