---
type: grilling
---

# The conventions ruleset

## Question

This is the ticket that makes "chartr ships no skills" actually work. Every shipped skill today leaks chartr-specific material — all eleven hit `.plan/`, `.chartr`, `## Answer` or the word chartr. A generic skill from someone else's repo knows none of it and will write maps wherever its own defaults point. The ruleset is what closes that gap: chartr's file-format contract, embedded, materialized to a stable path, and pointed at from both payloads.

Two prior attempts at this exist and both are being deleted — `docs/agents/issue-tracker.md` (a committed artifact in the operator's repo) and the `tracker-convention` skill (shadowable and toggleable, which a contract must not be). Understanding why each failed is the best guide to what this one must be.

Settle:

- **What is in it.** The candidates are the map directory layout, the ticket filename and numbering rule, the frontmatter fields, the section names, the derived-status table, the frontier definition, the claim fields, and the method vocabulary in `glossary.md`. Some of that is *format chartr parses* and some is *method a skill teaches*. Only the first belongs here — a ruleset that also teaches wayfinding has re-created the skill it replaced. Draw that line explicitly, field by field.
- **Whether one document is enough.** The ruleset is read by a session about to write a ticket, and also by one about to chart a map. One file the agent reads whole, or a small directory it zooms into? The payload cost is identical — it is a pointer either way — so this is about what an agent does with it once opened.
- **Materialization.** Where it lands, whether it is rewritten on upgrade, and what happens if the operator edits it. The existing `Materialize` never overwrites an operator's edits because hackability is the point; a *contract* the operator has edited into disagreement with the parser is a different proposition, and the answer may differ from the skill library's.
- **Enforcement versus statement.** The destination says the ruleset "enforces" conventions. Nothing in a markdown file enforces anything — `internal/wayfinder/lint.go` does. What does the ruleset promise, what does the lint actually check, and is a session told about the lint so a malformed ticket is caught by the agent rather than by the operator later?
- **The glossary's fate.** It is inlined into every ticket payload's context bundle today. If its vocabulary moves here, the bundle either sources it from here or stops carrying it because the payload already points at it. This decision is shared with [The two core payloads](02-the-two-core-payloads.md) and one of the two must own it.
- **What `.plan/maps/` being a convention implies.** It is stated in the ruleset, parsed by `internal/mapscan`, and restated in CLAUDE.md. Is the path itself configurable, or fixed forever — and if fixed, does the ruleset say so in a way that stops an agent from being creative about it.

## Done when

The ruleset's contents are enumerated with the format/method line drawn through them, its file shape and materialization behaviour are settled, its relationship to the lint is stated, and the glossary has exactly one home.
