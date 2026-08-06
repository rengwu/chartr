# The chartr file-format convention

This document is chartr's **write contract**: the exact on-disk format chartr
reads a wayfinder map in. It states the format and nothing else — it does not say
how to interview, how to decide, how to decompose work into tickets, how to size
them, or how to behave in a session. Those belong to whatever skill is doing the
work.

## When this applies

When work is asked to be **charted into a map**, **created as a wayfinder map**,
**broken into tickets**, or described in equivalent wording, the artifact produced
is a chartr-compatible wayfinder artifact in the format below. Everything about
*how* to arrive at its content comes from elsewhere; everything about *where the
bytes go and what shape they take* comes from here.

## Layout

A **map** is one directory of markdown. Maps live under `.plan/maps/` in the
repository root, and nowhere else:

```
.plan/maps/<slug>/
  map.md                     required — the map itself
  spec.md                    optional — the settled specification, if there is one
  tickets/
    01-the-first-question.md
    02-the-next-one.md
  assets/                    optional — supporting material a ticket names
```

- `<slug>` is lower-kebab-case and names the effort.
- **`.plan/maps/` is fixed and not configurable.** A `map.md` anywhere else —
  `.plan/<slug>/`, a nested directory, elsewhere in the repository — is invisible:
  it is not a map, and no reader will find it.
- A **planning map**, whose tickets resolve open questions, and an
  **implementation map**, whose tickets deliver code against a settled spec, use
  the same format. Where one effort has both, the implementation map is the
  sibling directory `.plan/maps/<slug>-impl/`.
- `spec.md` is a reserved slot: if an effort has a written specification, it lives
  at that path inside the planning map's directory, and is linked from the maps
  that read it.

## Ticket filenames and identity

A ticket file is `tickets/NN-kebab-case-slug.md`.

- `NN` is the ticket's **number** — its permanent identity. It is how every other
  ticket, every index line and every dependency edge refers to it.
- Numbers below 100 are **zero-padded to two digits** (`01`, `09`, `42`); above
  99 they take their natural width (`100`, `101`).
- **A number is never reused and never renumbered**, not when a ticket is ruled
  out, not when tickets are reordered, not when one is deleted. Existing readers
  tolerate legacy widths (`1-foo.md`); every new file is written in the canonical
  form.
- The slug is descriptive and may be edited; the number may not.

## `map.md`

An H1 title, then **five sections, in this order**, all present:

```markdown
# <The map's title>

## Destination

## Notes

## Decisions so far

## Not yet specified

## Out of scope
```

- **`## Destination`** — what done looks like for the whole effort, in prose.
- **`## Notes`** — the standing orientation every session working this map needs:
  constraints, warnings, how to run things, what to read first.
- **`## Decisions so far`** — the index of resolved tickets: **one bullet per
  resolved ticket**, the gist plus a relative link to the ticket file.

  ```markdown
  - [The ticket's title](tickets/02-the-ticket-slug.md) — the gist of what it
    settled, in a sentence or two.
  ```

  Links are relative to `map.md`, so `tickets/NN-slug.md` for a ticket on this
  map and `../<other-slug>/...` for one on another map.
- **`## Not yet specified`** — the fog: what is still unknown. Each patch is a
  **bold-lead bullet** — a bolded short name, then the prose. A patch that a
  particular ticket would clear carries a **clearing edge** as the bullet's last
  sentence, in italics, naming that ticket by relative link:

  ```markdown
  - **The short name of the unknown.** What is unknown and why it matters.
    *Anchored to [The ticket's title](tickets/04-the-ticket-slug.md).*
  ```

  A patch with no clearing edge is fog nobody has scheduled work against yet.
- **`## Out of scope`** — routes deliberately not travelled, recorded so they are
  not rediscovered. A ticket closed as **ruled out** is indexed here, by the same
  relative-link form `Decisions so far` uses — not there.

An empty section keeps its heading; a section may hold an HTML comment saying it
is deliberately empty.

## A ticket

```markdown
---
type: task
blocked_by: [01, 02]
undermined_by: []
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

### Frontmatter

YAML between `---` fences, at the very top of the file. The recognized keys:

| key | meaning |
| --- | --- |
| `type` | **required.** One of `grilling`, `research`, `prototype`, `task`. |
| `blocked_by` | ticket numbers this one depends on, as a list: `[01, 02]`. Empty or absent means unblocked. |
| `undermined_by` | ticket numbers whose answers call this one's into question. A flag for a human; never an automatic reopening. |
| `assets` | filenames under the map's `assets/` this ticket refers to. |
| `claimed_by` | **chartr-owned.** The id of the live session holding this ticket. |
| `claimed_at` | **chartr-owned.** RFC 3339 timestamp of that claim. |

- **`status` is forbidden.** A ticket's status is derived from its body (below).
  Writing it into frontmatter creates a stale second copy of a derived fact, and
  it is ignored where it appears.
- `claimed_by` / `claimed_at` are written and removed by chartr. Nothing else
  writes them.
- **Unknown keys are tolerated and ignored** — a key a reader does not recognize
  is never an error and never invalidates the ticket.

### Structural headings

The headings a reader looks for, exactly:

- `## Question` — what the ticket asks. Always present.
- `## Done when` — the condition that closes it. Always present.
- `## Answer` — written when the work is done: what was decided or built, why,
  and what was deliberately not done.
- `## Ruled out` — written instead of an answer when the ticket is closed as out
  of scope.

Any other heading is unknown to a reader and settles nothing.

## Derived status

A ticket's status is read off the file in this order, and never stored:

| The file says | Status |
| --- | --- |
| `## Answer` with prose under it | **resolved** |
| else `## Ruled out` with prose under it | **out of scope** |
| else a non-empty `claimed_by` | **claimed** |
| none of the above | **open** |

Closure is read first, so a claim left behind on a closed ticket is inert litter
rather than a broken state. A **bare** closing heading with nothing under it is
not an answer: a session that died just after typing one still reads as
unfinished.

## The frontier

The **frontier** is what can be worked right now: every **open** ticket all of
whose `blocked_by` tickets are **closed** (resolved or out of scope). A blocker
counts as cleared the moment its `## Answer` lands — there is no approval step in
between — so a dependent unblocks at the speed of the work.

A ticket that names a blocker which does not exist is not on the frontier.

## The words this format uses

- **Map** — one effort: a `map.md` and its tickets, in one directory under
  `.plan/maps/`.
- **Ticket** — one question or one unit of work, one file, sized to a single
  session.
- **Blocker** — a ticket another one depends on, named in its `blocked_by`.
- **Frontier** — the open tickets whose blockers are all closed.
- **Answer** — a resolved ticket's `## Answer`: its conclusion, written by
  whoever did the work. On disk, an answer *is* the resolution.
- **Ruled out** — a ticket closed as out of scope, recorded rather than deleted
  so the route is not rediscovered.
