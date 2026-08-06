---
type: task
blocked_by: []
claimed_by: sc617a7e5557c
claimed_at: 2026-08-06T11:17:15Z
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

## Answer

Created a new, separate local repo at `~/Desktop/Projects/chartr-skills`
(sibling to this checkout, not inside it — nothing in the chartr tree changed).
It carries the seven skills, `CONTRACT.md`, `LICENSE`, and a two-line `README.md`
pointing at the contract. Vendor commit: **`ece028dfd19a21d6b4b990c96d0efe2fa5c83a49`**
on `main` — that full ref, or its short form `ece028d`, is what ticket 05 pins.
The repo is local-only for now (operator's call, asked and confirmed at the
start of this session); it has no GitHub remote yet, which is fine for
identifying a commit but means ticket 05's actual clone-and-vendor step needs
a pushed remote first if it vendors by fetching rather than by local copy.

**Re-authoring, not a word swap.** Each of the four role skills
(`grill`/`prototype`/`research`/`implement`) needed almost no change beyond
stripping `on-ramp:`/`needs-context:` from frontmatter — they were already
free of chartr-specific claims. `wayfinder` and `to-tickets` needed real
surgery:

- Deleted both fenced section-skeleton blocks — `wayfinder`'s map-body
  template and `to-tickets`'s map-template/ticket-template pair — since
  `conventions.md` (per [ticket 04's answer](../../skill-sources/tickets/04-the-conventions-ruleset.md))
  now states both skeletons verbatim (headings, order, frontmatter,
  derived-status table). The method guidance that lived inside those
  placeholders — what a Destination is for, what an implementation map's
  Notes should cover (reading order, vocabulary source, review skills,
  static checks, linter) — survives as prose in the body text, per the
  ticket's explicit instruction that this re-authoring is part of the ticket,
  not a follow-up.
- Removed every chartr/cockpit-dependent sentence: "In a chartr space the
  cockpit does this driving," "what turns the ticket green in the cockpit,"
  "which is what lets chartr render it visually as a star-map," "Under
  chartr the claim is written and cleared for you." Each became a
  environment-neutral statement ("some environments automate this driving,"
  "an automated environment may write and clear this for you") that stays
  true whether or not any particular tool is watching — the failing examples
  the ticket itself named.
- Removed every reference to the deleted `tracker-convention` skill and its
  `glossary.md` (both retired by ticket 04) and to `domain-modeling` (culled,
  out of scope, nothing replaces it) — the `wayfinder` mention of
  domain-modeling was softened to "if one is available" rather than treated
  as a hard dependency, since a source may register any subset of skills.
- Kept the naming allowance: `map.md`, "the frontier," "a blocker," `##
  Answer`/`## Ruled out` are used throughout without being redefined.
- Kept `to-spec`'s own spec-body template — Problem Statement, Solution, User
  Stories, etc. — since ticket 04's answer explicitly disclaims prescribing a
  spec's own headings ("this ruleset does not prescribe its headings or
  recreate a `to-spec` template"). That skeleton isn't one conventions.md
  states, so the "no skeleton" clause doesn't reach it. `to-spec` did lose
  its one literal `.plan/maps/<slug>/spec.md` path restatement, softened to
  "beside the map it belongs to, as `spec.md`" — the fixed root is
  conventions.md's fact to state, not this skill's.
- `CONTRACT.md` states all six things the ticket asked for: the seven skills
  and what each is for, the two-clause test with its naming allowance and
  skeleton ban (quoting the ticket's own failing examples so a future editor
  has the bar in front of them), frontmatter-is-two-fields, no host framing,
  no relative cross-skill links (with the reason — a source may register a
  subset), role-short/method-long, and conventions.md winning on
  disagreement.
- `LICENSE` is MIT, adapted from the attribution notice already carried by
  `github.com/rengwu/skills` (the repo `internal/prompt/prompt.go`'s
  `SourceRepo`/`SourceCommit` currently point at), naming that `wayfinder`,
  `to-spec` and `to-tickets` derive from Matt Pocock's skills and that the
  other four are original to this repo.

**Verification done:** a human read (this session, standing in for the
operator) of every skill against `CONTRACT.md`'s two-clause test — grepped
the finished repo for `chartr`, `cockpit`, `tracker-convention`, `glossary`,
`harness`, `on-ramp`, `needs-context`, fenced code blocks, and literal
frontmatter keys (`blocked_by:`, `claimed_by:`, `type: task`); all clean
except the two intentional mentions inside `CONTRACT.md` itself (quoting the
ticket's own bad examples to teach the rule) and the repo's own name in
`README.md`. No test exists to write here, per the ticket: chartr validates
nothing about a source's skills by design.

**Deliberately left out:** no push to a remote (operator chose local-only);
no GitHub repo created; nothing in the wayfinder-harness tree touched, since
this ticket is prose against a separate repo and gates only ticket 05, which
does the vendoring.
