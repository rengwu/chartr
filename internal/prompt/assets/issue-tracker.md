<!-- chartr-tracker-adapter: v1 — chartr wrote this file; it is safe to edit, and chartr will ask before replacing it. -->

# Issue tracker: chartr (local markdown)

This repo's wayfinder tracker is **chartr**: plain markdown under `.plan/maps/`,
committed to git, watched live. **No remote tracker, no `.scratch/`.** Every
wayfinder skill reads and writes maps by this file — the format below is the
contract; don't invent shapes or fall back to any `.scratch/` default.

## Layout

```
.plan/maps/<slug>/          # planning map + its spec.md (siblings)
  map.md                    # H1 title; ## Destination, Notes, Decisions so far,
  tickets/NN-slug.md        #   Not yet specified, Out of scope
  assets/
.plan/maps/<slug>-impl/     # implementation map, same shape
```

Implementation maps go under `<slug>-impl/`. `NN` is a ticket's permanent
identity — never reused or renumbered. (`.plan/handoffs/`, `.plan/research/` are
not maps.)

## A ticket — `tickets/NN-slug.md`

```markdown
---
type: task            # task | grilling | research | prototype
blocked_by: [01, 02]  # ticket numbers whose ## Answer this builds on
claimed_by: <session> # set while worked; chartr writes/clears it (by hand only offline)
---
# Title
## Question   — what it asks, workable cold
## Done when  — the concrete condition
## Answer     — what was decided/built (writing it resolves the ticket)
```

## Status is derived, never stored — no `status:` field

- `## Answer` with prose → **resolved** · `## Ruled out` with prose → **out_of_scope**
- `claimed_by` and no closing section → **claimed** · else → **open**

**Frontier** = open tickets whose `blocked_by` are all resolved — the work that can
start now. Computed, never written. A blocker clears the instant its `## Answer`
lands.

## Rules

1. Write only under `.plan/maps/` — never `.scratch/`, remote, or `docs/`; maps
   elsewhere are invisible to chartr.
2. Never store status; the agent writes prose, the tooling derives.
3. The map is the memory — anything the next session needs lives in an `## Answer`
   or the map's Notes.

## Before committing

Every `blocked_by` names a real ticket; each number used once; no stated progress
counts (they're derived). chartr checks these live when it's driving.
