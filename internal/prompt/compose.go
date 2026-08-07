package prompt

import (
	_ "embed"
	"fmt"

	"strings"

	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/sources"
)

// spaceCoreText is chartr's own account of what it is, embedded rather than
// sourced because a session told nothing about how to behave must still be told
// what it is sitting in. It is one sentence, and it is held to the same test as
// every other sentence chartr writes in its own voice — it is still true if the
// agent does nothing about it.
//
// It says nothing about how *this* reader arrived, because it has two readers
// with different histories: a free session chartr launched, and an agent chartr
// never touched reading the standing `CHARTR.md` at the root of a space. What is
// true of both is the middle — maps derived from `.plan/maps/`, one session per
// ticket — and that is the whole of this asset.
//
//go:embed assets/core-space.md
var spaceCoreText string

// freeCoreTail is the one clause only a free session can be told: chartr opened
// its terminal, and opened it with nothing in hand. It sits beside the asset
// rather than in it because the standing document must not carry it — a `/new`
// agent's shell was not opened by chartr at all, and a document whose first
// paragraph is false to its reader is worth less than no document.
const freeCoreTail = "This shell is one chartr opened with no ticket and no role."

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

// ComposeInput is everything Compose needs: the role, the config root the
// contract files live under, the source registry and binding table the role
// resolves through, and the assembled bundle.
type ComposeInput struct {
	Role string
	// Bundle is the ticket-specific half — the map, the ticket, the blockers.
	Bundle Bundle
	// ConfigDir is the operator's config root. Composition reconciles the
	// generated conventions and the operator's preferences against it before it
	// assembles anything. Empty skips the reconcile, which is for tests that
	// compose without a config root at all.
	ConfigDir string
	// Sources and Bindings are how the *role* half of the payload resolves: the
	// operator's ordered source list, and the `[roles]` table saying which
	// source-qualified skill each role spawns with. Sources also renders the
	// payload's sources block, so it is required even for a free session.
	Sources  *sources.Registry
	Bindings config.RoleBindings
}

// Compose assembles the payload a session for this ticket and role would be
// told: chartr's embedded core, the bound role skill's body concatenated whole,
// the conventions pointer, the operator's preferences, and then the context
// region — the sources block, the map, this ticket, and each blocker's answer.
// Instructions, then data.
func Compose(in ComposeInput) (Payload, error) {
	if !config.IsRole(in.Role) {
		return Payload{}, fmt.Errorf("unknown role %q; want one of %v", in.Role, config.Roles)
	}

	// Every composition — a spawn and a preview alike — reconciles the config
	// root's two contract files first: the generated conventions are restored to
	// the canonical bytes, and an unreadable `preferences.md` fails the compose
	// here rather than quietly dropping the operator's instructions.
	contract, err := ReconcileContract(in.ConfigDir)
	if err != nil {
		return Payload{}, err
	}

	// The role is the operator's, and resolves through their binding table into
	// their sources. An unresolvable binding stops here, which is the whole
	// enforcement: the spawn path aborts on this error before the claim commit.
	role, err := resolveRoleSkill(in.Sources, in.Bindings, in.Role)
	if err != nil {
		return Payload{}, err
	}

	core := embeddedCore()
	parts := []Part{
		{Name: CoreSkill, Kind: "prompt", Origin: OriginChartr, Label: "core", Text: core.Body},
		// The part keeps the *role's* name, not the bound skill's: the preview reads
		// as "what the grill role got", and a role may be bound to a skill called
		// something else entirely. The body is concatenated rather than pointed at,
		// which is what makes the claim trailer's skill record mean *this text ran*.
		{Name: in.Role, Kind: "prompt", Origin: role.Source, Label: "skill", Text: role.Body},
	}
	parts = append(parts, contractParts(contract)...)

	srcPart, warnings := sourcesPart(in.Sources)
	parts = append(parts, srcPart,
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
		Skills:    []Skill{core, role},
		Warnings:  warnings,
		Markdown:  renderMarkdown(parts),
	}, nil
}

// ComposeFree assembles the payload a free session is told — an agent chartr
// launched into a space with no ticket and no role. Four parts: chartr's free
// core, the conventions pointer, the operator's preferences, and the sources
// block.
//
// **It carries no live fact about the space.** No map list, no frontier, no
// branch for a space without `.plan/` — the same bytes in an empty tree and a
// tree mid-effort. The agent is already in the tree and can list a directory,
// while a list composed at spawn is wrong the moment another session resolves a
// ticket, and a free session outlives that snapshot. What varies is the sources
// block and the preferences bytes, and that is all — which is why this takes no
// space at all, only the config root and the registry.
func ComposeFree(configDir string, reg *sources.Registry) (Payload, error) {
	return composeSpace(configDir, reg, freeCoreTail)
}

// ComposeStanding assembles the standing document chartr keeps at the root of
// every registered space — the same four parts a free session is told, minus the
// one clause that is only true of a shell chartr opened.
//
// Its reader is an agent chartr never launched: one started with `/new`, one the
// operator opened in their own terminal, one that compacted its brief away and
// came back for it. Nothing routes such an agent here; the file works on the
// agent's curiosity, and failing that on the operator saying "read CHARTR.md".
//
// Like the free payload it carries **no live fact about the space** — no map
// list, no frontier, no branch. That rule binds harder here, not softer: a
// standing file is written at startup and at a sources change and read at some
// unrelated later moment, so anything in it that could go stale certainly will.
func ComposeStanding(configDir string, reg *sources.Registry) (Payload, error) {
	return composeSpace(configDir, reg, "")
}

// composeSpace is the shared body of the two space-level payloads: chartr's
// core plus an optional tail, the conventions pointer, the operator's
// preferences, and the sources block. Both take no space at all, only the config
// root and the registry, which is what makes the standing document identical in
// every space and composable once for all of them.
func composeSpace(configDir string, reg *sources.Registry, tail string) (Payload, error) {
	contract, err := ReconcileContract(configDir)
	if err != nil {
		return Payload{}, err
	}

	core := strings.TrimSpace(spaceCoreText)
	if tail != "" {
		core += "\n\n" + tail
	}
	parts := []Part{{
		Name: CoreSkill, Kind: "prompt", Origin: OriginChartr, Label: "core",
		Text: core,
	}}
	parts = append(parts, contractParts(contract)...)
	srcPart, warnings := sourcesPart(reg)
	parts = append(parts, srcPart)

	return Payload{
		Parts:    parts,
		Warnings: warnings,
		Markdown: renderMarkdown(parts),
	}, nil
}

// ticketCoreText is the ticket session's core: the ground rules every session
// working a ticket inherits. Like the free core it is embedded rather than
// sourced — it is chartr's own voice, and not a source's to shadow (ADR 0017).
//
//go:embed assets/core-ticket.md
var ticketCoreText string

// embeddedCore is the ticket session's core, read straight out of the binary. It
// resolves through no source, so the claim trailer records it as `core=chartr`;
// which bytes that was is fixed by `Payload-SHA256` on the same commit.
func embeddedCore() Skill {
	meta, body := splitFrontmatter(ticketCoreText)
	return Skill{
		Name:        CoreSkill,
		Source:      OriginChartr,
		Description: meta["description"],
		Body:        strings.TrimSpace(body),
	}
}

// contractParts are the two config-root documents every payload carries, in
// order: the conventions as a *path*, and the operator's preferences as bytes.
//
// The conventions are pointed at rather than inlined because a location is a
// capability and survives being ignored, where "read this" is an instruction
// whose whole content is what the agent should do. The sentence therefore states
// the consequence — a map file chartr cannot parse is a map file chartr does not
// see — and carries the entire reason to open the path.
//
// `preferences.md` is the exception to all of it: the operator's own voice,
// unwrapped, unlabelled and unranked, and permitted to contradict everything
// above it. It is always a part, even when empty, so the preview shows the
// operator where their file lands.
func contractParts(c Contract) []Part {
	var parts []Part
	if c.ConventionsPath != "" {
		parts = append(parts, Part{
			Name: "conventions", Kind: "prompt", Origin: OriginChartr, Label: "contract",
			Text: fmt.Sprintf(
				"A file under `.plan/maps/` is read by chartr only where it follows the format stated at `%s`.",
				c.ConventionsPath),
		})
	}
	return append(parts, Part{
		Name: "preferences", Kind: "prompt", Origin: OriginOperator, Label: "preferences",
		Text: c.Preferences,
	})
}

// sourcesPart renders the inventory both payloads carry identically: every
// enabled source in resolution order with its local checkout path and the skill
// names the walk found, and a closing sentence stating what happens when two
// sources carry the same name.
//
// **A git source's URL is never printed.** It is a fetchable address in a
// document handed to an agent running with permissions skipped, on an effort
// whose standing decision is that nothing fetches unattended. The path is what a
// session can act on; the URL is what it should not.
//
// Descriptions are omitted deliberately: names fall out of the walk for free,
// where a description costs one file read per skill at every compose.
//
// The second return is the composition's warnings, with the one case that
// silently changes what a payload says — a registered source whose directory is
// not there.
func sourcesPart(reg *sources.Registry) (Part, []string) {
	var b strings.Builder
	b.WriteString("The skills chartr can resolve, in the order it resolves them.\n")

	var warnings []string
	if reg != nil {
		for _, st := range reg.States() {
			if st.Status == sources.StatusUnavailable {
				warnings = append(warnings, fmt.Sprintf(
					"the source %q is unavailable — nothing is at %s, so every skill it carried resolves to nothing",
					st.Name, st.Path))
			}
			if !st.Enabled {
				continue
			}
			names := make([]string, 0, len(st.Skills))
			for _, sk := range st.Skills {
				names = append(names, sk.Name)
			}
			list := strings.Join(names, ", ")
			if list == "" {
				list = "no skills"
			}
			fmt.Fprintf(&b, "\n- `%s` at `%s` — %s", st.Name, st.Path, list)
		}
	}

	b.WriteString("\n\nWhere two of them carry a skill of the same name, the earlier one is what a bare name reaches, and the later one is reached as `source/skill`.")
	return ctxPart("sources", "Skill sources", b.String()), warnings
}

// AnswerSection returns a ticket's closing answer prose for inlining as a
// blocker's answer in the context bundle — its Answer, else its Ruled out,
// together with any amendment sections appended below it. Empty when the blocker
// carries none, which Compose renders as an explicit "not resolved" note rather
// than a silent gap. An in-flight `## Proposed Answer` is deliberately *not*
// read: it is an unknown heading no one blessed, and handing it to a dependent as
// though it were the answer is the one failure this narrowing exists to prevent.
func AnswerSection(body string) string {
	return answerThroughAmendments(body, "Answer", "Ruled out")
}

// isAmendment reports whether a `## ` heading opens a section that corrects the
// answer above it rather than starting an unrelated one.
//
// This repo's convention is to amend a resolved ticket by *appending* a
// `## Correction …` or `## Amendment (…)` section rather than editing the answer
// in place, so the answer and its corrections are one artifact split across
// sibling headings. Reading only up to the next `## ` would hand a dependent the
// superseded text and silently drop the retraction — the worst of both, since the
// wrong statement travels with the authority of a blessed answer. Prefix-matched
// because both headings carry trailing prose (`## Correction — the glossary …`,
// `## Amendment (2026-07-19 — operator-approved)`).
func isAmendment(heading string) bool {
	h := strings.ToLower(strings.TrimSpace(strings.TrimPrefix(heading, "## ")))
	return strings.HasPrefix(h, "correction") || strings.HasPrefix(h, "amendment")
}

func ctxPart(name, label, text string) Part {
	return Part{
		Name:   name,
		Kind:   "context",
		Origin: OriginContext,
		Label:  label,
		Text:   strings.TrimRight(text, "\n"),
	}
}

func orDash(s string) string {
	if strings.TrimSpace(s) == "" {
		return "—"
	}
	return s
}

// renderMarkdown is the single-document view of the payload: the prompt parts
// (core, role, conventions, preferences) run first as the instruction, then a
// `# Context` section gathers every context part under its own heading. An empty
// part contributes nothing — an operator with no preferences leaves no gap. This
// is what gets written to the gitignored payload file and pointed at with a
// one-line opener, and what the preview shows.
func renderMarkdown(parts []Part) string {
	var prompts, context []Part
	for _, p := range parts {
		if strings.TrimSpace(p.Text) == "" {
			continue
		}
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
		b.WriteString(strings.TrimSpace(p.Text))
	}
	if len(context) > 0 {
		b.WriteString("\n\n---\n\n# Context\n")
		for _, p := range context {
			b.WriteString("\n## ")
			b.WriteString(p.Label)
			b.WriteString("\n\n")
			b.WriteString(strings.TrimSpace(p.Text))
			b.WriteString("\n")
		}
	}
	return strings.TrimSpace(b.String()) + "\n"
}

// answerThroughAmendments returns the body under the first matching `## <name>`
// heading, continuing through any amendment sections that follow it and stopping
// at the first unrelated `## ` heading. Case-insensitive on the heading text.
//
// An amendment's own heading is kept — it carries the correction's point
// ("the glossary that rides the bundle is not `CONTEXT.md`") and marks the prose
// below it as superseding rather than continuing what came before. The matched
// answer heading itself is dropped, as the part is already labelled.
func answerThroughAmendments(body string, names ...string) string {
	lines := strings.Split(body, "\n")
	for _, name := range names {
		want := "## " + name
		for i, l := range lines {
			if !strings.EqualFold(strings.TrimSpace(l), want) {
				continue
			}
			var out []string
			for j := i + 1; j < len(lines); j++ {
				if strings.HasPrefix(lines[j], "## ") && !isAmendment(lines[j]) {
					break
				}
				out = append(out, lines[j])
			}
			return strings.TrimSpace(strings.Join(out, "\n"))
		}
	}
	return ""
}
