// Package prompt composes the payload a session is told. chartr ships no skill
// library: the two cores are chartr's own voice, embedded in the binary, and
// every *role* body comes from a source the operator registered (ADR 0017).
//
// A payload is assembled from interchangeable parts in one order — core, role,
// conventions, preferences, then the context region — under one rule:
// **instructions, then data.** chartr keeps composing it itself (ADR 0002,
// reaffirmed) and assembles the context fresh every time, never accumulated
// (ADR 0005): it reads its embedded core, concatenates the bound role skill's
// body with its frontmatter stripped, points at the generated conventions,
// appends the operator's preferences verbatim, and closes with the sources block
// and — for a ticket session — the map body, the ticket, and its blockers'
// answers.
//
// Two payloads come out of it. A **ticket session** gets the core plus the bound
// role's body; a **free session** is told what chartr is and what skills exist,
// and nothing about how to behave.
package prompt

import "strings"

// CoreSkill is the name both cores carry in the payload's part list and on the
// claim trailer. chartr reads it straight out of the binary rather than through
// a source: it is chartr's own voice, and not the operator's to shadow.
const CoreSkill = "core"

// Part origins — where a block of the payload came from, as the preview badges
// it. The set is open on purpose: a resolved skill body's origin is the
// *registered source's name*, so the badge answers the one silent failure source
// order can cause (which source's copy of this skill actually ran). These three
// are the fixed members: chartr's own embedded text, the operator's preferences,
// and anything assembled fresh at compose time.
const (
	OriginChartr   = "chartr"
	OriginOperator = "operator"
	OriginContext  = "context"
)

// skillFile is the standard entry point of a skill directory — the one thing
// chartr knows about the markdown a source carries.
const skillFile = "SKILL.md"

// Skill is one skill composed into a payload: where it resolved from and the
// body that was concatenated, with its frontmatter stripped.
//
// Source is the registered source's name, and Commit the pin that source carries
// where it is a git checkout — the pair is what the claim trailer records. It is
// deliberately weaker than the content hash it replaced and honest about being
// so: a source name plus a commit identifies bytes a teammate can *fetch*, where
// a hash could only ever have told them something differed, and a `dir` source
// has no pin at all (ADR 0017). chartr's own embedded cores carry Source
// "chartr" and no commit; `Payload-SHA256` on the same commit still fixes the
// exact bytes for the machine that composed them.
type Skill struct {
	Name        string `json:"name"`
	Dir         string `json:"dir,omitempty"`
	Description string `json:"description,omitempty"`
	Source      string `json:"source,omitempty"`
	Commit      string `json:"commit,omitempty"`

	Body string `json:"-"`
}

// Part is one labelled block of the payload — chartr's core, a resolved role
// skill, the conventions pointer, the operator's preferences, or a context
// artifact (the sources block, the map, this ticket, a blocker's answer). Kind is
// "prompt" or "context"; Origin is where the text came from, and is what the
// preview badges.
//
// A part is one contiguous block of text. It was a list of tagged segments while
// a per-field merge across the skill layers looked likely; whole-skill shadowing
// never produced a second segment, and resolution through sources cannot, so the
// machinery is gone.
type Part struct {
	Name   string `json:"name"`
	Kind   string `json:"kind"`
	Origin string `json:"origin"`
	Label  string `json:"label,omitempty"`
	Text   string `json:"text"`
}

// Payload is the whole composed result for one ticket and role: the parts with
// their provenance, the skills that were composed into it (the claim commit's
// provenance trailers), any warnings, and the single markdown document the parts
// render to — exactly what a session would be told.
type Payload struct {
	Role      string   `json:"role"`
	TicketNum int      `json:"ticketNum"`
	Parts     []Part   `json:"parts"`
	Skills    []Skill  `json:"skills,omitempty"`
	Warnings  []string `json:"warnings,omitempty"`
	Markdown  string   `json:"markdown"`
}

// splitFrontmatter peels a leading `---` delimited block off a SKILL.md,
// returning its simple `key: value` pairs and the body below it. chartr reads
// only `description`, and only for the preview; the frontmatter never reaches
// the payload. A file without frontmatter is all body.
//
// chartr validates nothing else about a source's skills: discovery is the whole
// test, and rejecting someone else's markdown for not being written the way
// chartr writes is exactly what registering your own source is meant to avoid.
func splitFrontmatter(src string) (map[string]string, string) {
	meta := map[string]string{}
	lines := strings.Split(src, "\n")
	if len(lines) == 0 || strings.TrimSpace(lines[0]) != "---" {
		return meta, src
	}
	end := -1
	for i := 1; i < len(lines); i++ {
		if strings.TrimSpace(lines[i]) == "---" {
			end = i
			break
		}
	}
	if end < 0 {
		return meta, src
	}
	for _, l := range lines[1:end] {
		i := strings.Index(l, ":")
		if i < 0 {
			continue
		}
		key := strings.ToLower(strings.TrimSpace(l[:i]))
		val := strings.Trim(strings.TrimSpace(l[i+1:]), `"'`)
		if key != "" {
			meta[key] = val
		}
	}
	return meta, strings.Join(lines[end+1:], "\n")
}
