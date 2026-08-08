// Package prompt composes the payload a session is told. chartr ships no
// skill library: the two cores are chartr's own voice, embedded in the
// binary, and every role body comes from a source the operator registered
// (ADR 0017).
//
// A payload assembles from interchangeable parts in one order — core, role,
// conventions, preferences, then context — under one rule: instructions,
// then data. Assembled fresh every time, never accumulated (ADR 0005): the
// embedded core, the bound role skill's body with frontmatter stripped, a
// pointer to the generated conventions, the operator's preferences
// verbatim, then the sources block and — for a ticket session — the map
// body, the ticket, and its blockers' answers.
//
// Two payloads come out of it. A ticket session gets the core plus the
// bound role's body; a free session is told what chartr is and what skills
// exist, nothing about how to behave.
package prompt

import "strings"

// CoreSkill is the name both cores carry in the payload's part list and on
// the claim trailer. Read straight out of the binary rather than through a
// source — chartr's own voice, not the operator's to shadow.
const CoreSkill = "core"

// Part origins — where a block of the payload came from, as the preview
// badges it. Open on purpose: a resolved skill body's origin is the
// registered source's name, answering which source's copy of a skill
// actually ran. These three are the fixed members: chartr's own embedded
// text, the operator's preferences, anything assembled fresh at compose time.
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
// Source is the registered source's name, Commit the pin it carries when
// it's a git checkout — the pair the claim trailer records. Deliberately
// weaker than the content hash it replaced: a source name plus commit
// identifies bytes a teammate can fetch, where a hash could only say
// something differed, and a `dir` source has no pin at all (ADR 0017).
// chartr's own embedded cores carry Source "chartr" and no commit;
// `Payload-SHA256` still fixes the exact bytes for the machine that
// composed them.
type Skill struct {
	Name        string `json:"name"`
	Dir         string `json:"dir,omitempty"`
	Description string `json:"description,omitempty"`
	Source      string `json:"source,omitempty"`
	Commit      string `json:"commit,omitempty"`

	Body string `json:"-"`
}

// Part is one labelled block of the payload — chartr's core, a resolved
// role skill, the conventions pointer, the operator's preferences, or a
// context artifact (sources block, map, ticket, blocker's answer). Kind is
// "prompt" or "context"; Origin is what the preview badges.
type Part struct {
	Name   string `json:"name"`
	Kind   string `json:"kind"`
	Origin string `json:"origin"`
	Label  string `json:"label,omitempty"`
	Text   string `json:"text"`
}

// Payload is the whole composed result for one ticket and role: parts with
// provenance, the skills composed into it (the claim commit's provenance
// trailers), warnings, and the single markdown document the parts render
// to — exactly what a session is told.
type Payload struct {
	Role      string   `json:"role"`
	TicketNum int      `json:"ticketNum"`
	Parts     []Part   `json:"parts"`
	Skills    []Skill  `json:"skills,omitempty"`
	Warnings  []string `json:"warnings,omitempty"`
	Markdown  string   `json:"markdown"`
}

// splitFrontmatter peels a leading `---` delimited block off a SKILL.md,
// returning its `key: value` pairs and the body below. chartr reads only
// `description`, only for the preview; frontmatter never reaches the
// payload. A file without frontmatter is all body.
//
// Nothing else about a source's skills is validated: discovery is the whole
// test, and rejecting markdown for not being written chartr's way is
// exactly what registering your own source is meant to avoid.
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
