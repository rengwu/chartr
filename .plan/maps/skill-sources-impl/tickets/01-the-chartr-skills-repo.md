---
type: task
blocked_by: []
---

# The chartr-skills repo

## Question

Create the repo whose bytes chartr vendors as its default source. It is prose, not
Go, and it is the effort's longest-lead item — nothing in this repo compiles
against it, so it runs concurrently with 02, 03 and 04 and gates only ticket 05.

Seven skills ship: `grill`, `prototype`, `research`, `implement`, `wayfinder`,
`to-spec`, `to-tickets`. Each is re-authored from the copies currently vendored
under `internal/prompt/assets/skills/`, against the spec's two-clause acceptance
test — **no sentence whose truth depends on chartr running**, and **no rule
`conventions.md` states**. Today's copies fail the first clause repeatedly and
concretely: `wayfinder` says *"In a chartr space the cockpit does this driving"*
and names `tracker-convention` as the format contract; `to-tickets` says *"what
turns the ticket green in the cockpit"*. Every one of those is a re-authoring, not
a word swap — the first two name a product the reader may not have, the third names
a skill that no longer exists.

Naming is allowed; specifying is not. A skill may write `map.md`, "the frontier",
"a blocker", and the closing-section token — a method cannot discuss its own
outputs without naming them. What it may not do is be the place a rule is *stated*.
**No skill carries a section skeleton**: the conventions file states both skeletons,
and the method guidance currently living *inside* those placeholders — *"⟨what
reaching the end of this map looks like … every session orients to it before
choosing a ticket⟩"* — is method rather than format, so it must be re-authored as
prose rather than disappearing with the template. That re-authoring is part of this
ticket, not a follow-up.

Frontmatter is `name` and `description` only. The launcher-era `on-ramp:` and
`needs-context:` keys are stripped from all four role skills — they die with the
launcher in ticket 08. Write `CONTRACT.md` stating: the seven skills and what each
is for; the two-clause test with its naming allowance and its ban on skeletons;
frontmatter is two fields; no Claude-Code framing (slash commands, hooks, loaders);
no relative links between skill directories, since a source may be registered as a
subset; role skills short, method skills long; and that `conventions.md` wins on any
disagreement. MIT, carrying the attribution the existing skills repo already
carries — `wayfinder`, `to-spec` and `to-tickets` derive from Matt Pocock's skills
and the notice travels with them. This is not optional housekeeping: ticket 05 puts
these bytes inside a distributed binary.

Nothing in this ticket touches the chartr tree, so the repo's own bar is a human
read against `CONTRACT.md` — chartr validates nothing about a source's skills by
design, so there is no test in this repo to write.

## Done when

The repo exists with seven `SKILL.md` directories, `CONTRACT.md`, and MIT plus
attribution; every skill is correct for a reader who has never run chartr, given
the same `conventions.md`; no skill contains a section skeleton or restates a rule
the conventions state; and the placeholder method guidance survives as prose. The
commit to vendor is identified for ticket 05.
