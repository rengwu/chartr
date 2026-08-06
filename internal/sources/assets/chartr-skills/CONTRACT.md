# Contract

This repo ships seven `SKILL.md` directories and nothing else that a reader
needs to run them. It is a **default skill source**: a directory of markdown
meant to be registered — alongside, or in place of, anyone else's — by an
environment that resolves skills by name and hands their bodies to an agent.
This document is what keeps that possible. On any disagreement between a
skill's prose and this contract, **the contract wins**.

## The seven skills

- **`grill`** — a role: interrogate an open question on a planning map until
  one decision survives its own weaknesses, and record it as the ticket's
  answer.
- **`prototype`** — a role: build the throwaway slice that settles one doubt
  on a planning map, and report the finding rather than the code.
- **`research`** — a role: investigate an open question on a planning map and
  land grounded, cited knowledge the map can build on.
- **`implement`** — a role: deliver working code against a settled spec on an
  implementation map, building to the ticket's Done-when.
- **`wayfinder`** — a method: chart a big, foggy effort as a shared map of
  tickets and work it one ticket at a time until the way to the destination is
  clear.
- **`to-spec`** — a method: synthesize the current conversation into a spec,
  no interview.
- **`to-tickets`** — a method: break a plan or spec into tracer-bullet
  implementation tickets, written as a wayfinder map.

The four roles are what a ticket-scoped session is bound to and reads as its
whole job; `wayfinder`, `to-spec` and `to-tickets` are invoked by name to
chart, spec, and ticket work before any ticket session exists.

## The acceptance test

Every sentence in every skill must pass both clauses:

1. **No sentence whose truth depends on this environment running.** A skill
   read by an agent that has never heard of the environment that sourced it
   must still be entirely true. "In a chartr space the cockpit does this
   driving" fails this — it names a product the reader may not have. "The
   frontier is derived from those edges, never written down" passes — it is
   true with or without any particular tool watching.
2. **No rule this environment's own file-format convention states.** Where
   files live, a ticket's frontmatter, the section headings and their exact
   order, and how status is derived belong to that convention alone — a skill
   here does not re-teach them, because two owners of one rule is how the two
   drift.

**Naming is allowed; specifying is not.** A skill may write `map.md`, "the
frontier", "a blocker", and the closing-section token — a skill cannot discuss
its own outputs without naming them. What it may not do is be the place a rule
about them is *stated*.

**No skill carries a section skeleton.** A fenced block reproducing the map's
five headings, or a ticket's frontmatter and its `## Question` / `## Done
when` headings, is the format restated in template form — the same failure as
stating it in prose, just harder to spot on review. Where a skeleton's
placeholder used to carry method guidance, that guidance is re-authored as
prose in the skill, not dropped with the template.

## Frontmatter

Exactly two fields: `name` and `description`. Nothing else — no on-ramp flags,
no context requirements, no environment-specific routing keys. A skill that
needs to say more about when it applies says so in its body.

## No host framing

Nothing here names a specific coding agent, CLI, or product surface. No slash
commands, no hooks, no loaders, no "invoke this with `/foo`". A skill is
prose an agent reads and follows; how the surrounding tool invokes it is the
tool's business, not this repo's.

## No relative links between skill directories

A source may be registered with only some of these skills present — a subset,
reordered, or standing beside a different repo's skills entirely. A relative
link from one skill directory into another breaks the moment that assumption
holds. Name another skill in prose (`the research skill`) if you must refer to
one; never link to its file.

## Role skills short, method skills long

`grill`, `prototype`, `research` and `implement` are read as part of a bigger
assembled document, every time a ticket session spawns — they earn their
place by staying short. `wayfinder`, `to-spec` and `to-tickets` are read on
their own, once, to do one significant piece of work — they can afford to be
long, and the method they teach needs the room.

## `conventions.md` wins

Where an environment provides a file-format convention — commonly named
`conventions.md` or similar — and it disagrees with anything written here,
the convention wins. This repo teaches method; it never owns format.

## License

MIT. See [`LICENSE`](LICENSE) for the license text and attribution.
