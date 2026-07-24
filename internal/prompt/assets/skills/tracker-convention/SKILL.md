---
name: tracker-convention
description: The wayfinder map format chartr reads — the files under .plan/maps/, a ticket's frontmatter and sections, and the status derived from them. Restates the convention; it does not change it.
---

# The wayfinder tracker convention

A wayfinder effort — a **map** — is a directory of markdown under
`.plan/maps/`. There
is no database and no status field: everything a reader needs is in the files,
and a ticket's state is *derived* from what the ticket already says. This skill
restates that format so a session (or any other tool) reads a map the same way
chartr does. It describes the convention; it does not change it. The shared
vocabulary lives alongside this file in [`glossary.md`](glossary.md).

## Layout

```
.plan/maps/<slug>/
  map.md                    the effort: destination, notes, decisions, scope
  tickets/
    01-the-first-question.md
    02-the-next-one.md
  assets/                   optional supporting material a ticket names
```

A ticket file is `NN-slug.md`. The number is the ticket's identity — it is how
every other ticket refers to it — so it never changes once written.

## `map.md`

An H1 title, then the sections a map keeps current:

- `## Destination` — what done looks like for the whole effort.
- `## Notes` — the standing orientation every session on this map needs.
- `## Decisions so far` — one line per resolved ticket: the gist plus a link.
- `## Not yet specified` — the fog: what is still unknown, each patch anchored to
  the ticket that would clear it.
- `## Out of scope` — routes deliberately not travelled, recorded so they are not
  rediscovered.

A map is either a **planning map** (its tickets resolve decisions) or an
**implementation map** (its tickets deliver code against a settled spec). The two
share this format; they differ in what a ticket asks for and which roles work it.

## A ticket

```markdown
---
type: task
blocked_by: [01, 02]
undermined_by: []
claimed_by: s1a2b3c4d5e6f
claimed_at: 2026-07-22T04:08:25Z
assets: [sketch.png]
---

# The ticket's title

## Question

What this ticket asks, in enough context that a session can work it cold.

## Done when

The concrete condition that makes it done — a checklist, not a mood.

## Answer

What was decided or built, why, and what was deliberately left out.
```

- **`type`** — `grilling`, `research`, `prototype` (planning maps) or `task`
  (implementation maps). It selects the kind of work, and on its own the role a
  session is spawned in — the four types map one-to-one onto the four roles, and
  nothing about the map narrows that.
- **`blocked_by`** — the ticket numbers whose answers this one builds on. A
  blocker's `## Answer` is handed to a session as a premise.
- **`undermined_by`** — tickets whose answers have called this one's into
  question. A flag for a human, never an automatic reopening.
- **`claimed_by` / `claimed_at`** — the live session holding the ticket.
  chartr writes these at spawn and removes them when the session dies without
  answering; nothing else touches them.
- **`assets`** — files under the map's `assets/` this ticket refers to.

## Derived status

A ticket's status is read off the file, in this order — never stored:

| The file says | Status |
| --- | --- |
| `## Answer` with prose under it | `resolved` |
| `## Ruled out` with prose under it | `out_of_scope` |
| a `claimed_by` marker and no closing section | `claimed` |
| none of the above | `open` |

Closure is read first, so a claim left behind on a closed ticket is inert litter
rather than a broken invariant. A *bare* closing heading with nothing under it is
not an answer: a session that died just after typing one still reads unfinished.
Any other heading is unknown to the reader and settles nothing.

The **frontier** is what can be worked right now: the open tickets whose blockers
are all closed. A blocker counts as cleared the moment its `## Answer` lands, so
a dependent unblocks at the speed of the work — there is no approval hop in
between.

## The two rules that keep it honest

1. **The map is the memory.** Sessions are not; each one is assembled fresh and
   leaves nothing behind but its answer. Anything the next session must know
   belongs in a ticket's answer or the map's notes.
2. **The agent writes, the tooling watches.** An `## Answer` is written by the
   session that did the work, in its own words. Status is derived from that
   writing, never asserted over it.
