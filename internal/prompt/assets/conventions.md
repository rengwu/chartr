# Chartr map format

This is the write contract for chartr maps. Use it when asked to chart work into
a map, create a wayfinder map, break work into tickets, or equivalent. It defines
only the files and their format; a skill defines how to plan and work.

## Layout

```text
.plan/maps/<slug>/
  map.md
  spec.md                  optional settled specification
  tickets/
    01-kebab-case-slug.md
  assets/                  optional supporting files
```

- Maps exist only at `.plan/maps/<slug>/`, relative to the repository root.
  `<slug>` is lower-kebab-case. `map.md` is required. A `map.md` anywhere else is
  not discovered. The root is fixed and not configurable.
- Planning and implementation maps use the same format. If both exist,
  `.plan/maps/<slug>-impl/` is the implementation map.
- A written specification must be `.plan/maps/<slug>/spec.md` in the planning
  map. Link it from every map that uses it. Chartr does not parse its contents.
- Asset entries are paths relative to the map's `assets/` directory.

## Ticket files and identity

A ticket file is `tickets/NN-kebab-case-slug.md`.

- `NN` is the ticket's permanent identity. Use two digits below 100 (`01`–`99`)
  and natural width from 100 onward. Readers accept legacy widths such as `1`;
  writers must not create them.
- Never change or reuse a number, including after reordering, deletion, or ruling
  a ticket out. The slug may change.
- Numbers identify tickets in filenames, links, `blocked_by`, `undermined_by`,
  and `<clears-with: NN>`.

## `map.md`

Write an H1 title followed by all five H2 sections in this order:

```markdown
# <title>

## Destination

## Notes

## Decisions so far

## Not yet specified

## Out of scope
```

- `Destination`: non-empty prose defining completion for the map.
- `Notes`: standing context needed by any session, such as constraints,
  warnings, commands, and prerequisite reading.
- `Decisions so far`: exactly one top-level bullet for every resolved ticket on
  this map. Link the ticket and summarize its answer:

  ```markdown
  - [Ticket title](./tickets/02-ticket-slug.md) — concise gist.
  ```

- `Not yet specified`: one top-level, bold-lead bullet per unresolved patch. If
  a ticket is assigned to clear it, end the bullet with `<clears-with: NN>`:

  ```markdown
  - **Short name.** What is unknown and why it matters. <clears-with: 04>
  ```

  Omit the marker when no ticket owns the patch.
- `Out of scope`: boundaries and exactly one link bullet for every ticket on
  this map closed with `## Ruled out`. Use the same link form as `Decisions so
  far`; never list a ruled-out ticket as a decision.

Except for `Destination`, a section may be empty or contain an HTML comment.
Always keep its heading. Index only tickets belonging to this map; same-map
links must start with `./tickets/`.

## Ticket format

An open ticket has this shape and no closing heading:

```markdown
---
type: task
blocked_by: [01, 02]
undermined_by: []
assets: [sketch.png]
---

# <title>

## Question

Enough context to work the ticket without prior conversation.

## Done when

Concrete, verifiable completion criteria.
```

Frontmatter must be a leading `---`-delimited block. Recognized keys:

| Key | Rule |
| --- | --- |
| `type` | Required: `grilling`, `research`, `prototype`, or `task`. |
| `blocked_by` | Optional list of premise tickets. Absent or empty means none. |
| `undermined_by` | Optional list of tickets whose answers may invalidate this ticket's answer. This flags human judgment; it never reopens a ticket. |
| `assets` | Optional list of paths relative to this map's `assets/`. |
| `claimed_by` | Chartr-owned session ID. Agents must not create, edit, or remove it. |
| `claimed_at` | Chartr-owned RFC 3339 claim time. Agents must not create, edit, or remove it. |

`status` is forbidden and ignored: status is derived. Unknown keys are tolerated
and ignored.

The structural H2 headings are exact:

- `## Question` and `## Done when` are always required.
- To resolve a ticket, add `## Answer` with non-empty content stating what was
  decided or built, why, and what was excluded.
- To close it as out of scope, add `## Ruled out` instead, with non-empty content
  explaining the boundary.
- Never include both closing headings. Other headings do not affect status. A
  bare closing heading closes nothing.

## Status and frontier

Status is derived in this order:

| First matching condition | Status |
| --- | --- |
| Non-empty `## Answer` | `resolved` |
| Non-empty `## Ruled out` | `out_of_scope` |
| Non-empty `claimed_by` | `claimed` |
| Otherwise | `open` |

A leftover claim does not change a closed ticket's status.

The **frontier** contains each `open` ticket whose every `blocked_by` ticket
exists and is `resolved`. `claimed` and `out_of_scope` tickets are not open. An
`out_of_scope` or missing blocker does not clear an edge. A resolved blocker
clears its edge as soon as its non-empty `## Answer` is written; there is no
approval step.

## Terms

- **Map:** one `map.md` and its tickets under `.plan/maps/<slug>/`.
- **Ticket:** one question or unit of work, in one file, for one session.
- **Blocker:** a ticket named in another ticket's `blocked_by`.
- **Frontier:** open tickets whose blockers all exist and are resolved.
- **Answer:** the `## Answer` content that resolves a ticket.
- **Ruled out:** closed as out of scope and retained to prevent rediscovery.
