---
name: to-tickets
description: Break a plan, spec, or the current conversation into tracer-bullet implementation tickets, written as a wayfinder map — one file per ticket under `<slug>-impl/`, so progress can be tracked against it.
---

# To Tickets

Break a plan, spec, or conversation into a set of **tickets** — tracer-bullet vertical slices, each declaring the tickets that **block** it.

The output is a **wayfinder implementation map**: a real wayfinder map, in your environment's own file-format convention, not a flat checklist. One `map.md` plus one file per ticket, under a `<slug>-impl/` directory sibling to the plan it implements. This is what lets an environment render the tickets and track progress against them — the same tool that reads a planning map reads this one, because it *is* a planning map, one whose Notes carry execution rather than decisions.

**This skill assumes your environment's own file-format convention for wayfinder maps** — where files live, what frontmatter carries structure, how status is derived, and what to verify before committing. Everything below defers to it — this skill only says how an *implementation* map differs from a planning one. The format is never invented here.

## How an implementation map differs from a planning map

A planning map (plain `wayfinder`) charts **decisions** and resolves them by grilling. An implementation map charts **work** and resolves it by shipping code. The format is identical; three things differ, and the guidance below bakes them in:

- **Every ticket is a task** — wayfinder's one ticket type that *does* rather than decides. No research/prototype/grilling tickets; the deciding already happened on the planning map.
- **The Notes carry execution.** wayfinder is "plan, don't do" by default and lets an effort override that in its Notes; an implementation map always does.
- **There is no fog.** Every question is settled in the spec, so **Not yet specified** stays empty; a genuinely new question goes back to the planning map rather than opening fog here.

Everything else — file layout, blocking edges, derived status, claims, verification — is exactly what your environment's convention states.

## Process

### 1. Gather context

Work from whatever is already in the conversation context. If the user passes a reference (a spec path, or any other document) as an argument, fetch it and read it in full — it is the source of truth the map executes.

### 2. Explore the codebase (optional)

If you have not already explored the codebase, do so to understand the current state of the code. If a `CONTEXT.md` glossary exists at the repo root, ticket titles and descriptions should use its vocabulary; respect ADRs in `docs/adr/` in the area you're touching. If neither exists, proceed silently.

Look for opportunities to prefactor the code to make the implementation easier. "Make the change easy, then make the easy change."

### 3. Draft vertical slices

Break the work into **tracer bullet** tickets.

<vertical-slice-rules>

- Each slice cuts a narrow but COMPLETE path through every layer the change touches (e.g. schema, API, UI, tests) — vertical, NOT a horizontal slice of one layer
- A completed slice is demoable or verifiable on its own
- Each slice is sized to fit in a single fresh context window
- Any prefactoring should be done first

</vertical-slice-rules>

Give each ticket its **blocking edges** — the other tickets that must complete before it can start. A ticket with no blockers can start immediately.

**Wide refactors are the exception to vertical slicing.** A **wide refactor** is one mechanical change — rename a column, retype a shared symbol — whose **blast radius** fans across the whole codebase, so a single edit breaks thousands of call sites at once and no vertical slice can land green. Don't force it into a tracer bullet; sequence it as **expand–contract**. First expand: add the new form beside the old so nothing breaks. Then migrate the call sites over in batches sized by blast radius (per package, per directory), each batch its own ticket blocked by the expand, keeping the build green batch to batch because the old form still exists. Finally contract: delete the old form once no caller remains, in a ticket blocked by every migrate batch. When even the batches can't stay green alone, keep the sequence but let them share an integration branch that all block a final integrate-and-verify ticket — green is promised only there.

### 4. Quiz the user

Present the proposed breakdown as a numbered list. For each ticket, show:

- **Title**: short descriptive name
- **Blocked by**: which other tickets (if any) must complete first — refer to them by title, not number, at this stage
- **What it delivers**: the end-to-end behaviour this ticket makes work

Ask the user:

- Does the granularity feel right? (too coarse / too fine)
- Are the blocking edges correct — does each ticket only depend on tickets that genuinely gate it?
- Should any tickets be merged or split further?

Iterate until the user approves the breakdown. Only once approved do you assign numbers and write files — numbering before approval invites churn in the blocking edges.

### 5. Write the implementation map

Write to a `<slug>-impl/` directory, a **sibling** of the spec's own directory, reusing the spec's own slug — if the tickets came from a plan named `<slug>`, the implementation map is `<slug>-impl` so the plan and its implementation sit side by side. Create the directory if needed. Tell the user the path.

Do **not** modify or delete the source spec or planning map — the implementation map references them.

Number each ticket in dependency order, blockers first, following whatever identity rule your environment's convention states for ticket files.

Write the map body and each ticket following your environment's structural convention, with this method content filled in:

- **The map's Destination**: the spec implemented end to end, from the user's perspective — what "done" looks like when every ticket has shipped. Link the spec. One short paragraph.
- **The map's Notes**, carrying execution: state plainly that every ticket is a task delivering working code, not a decision, and that all decisions were made on the planning map and synthesized into the spec — the single source of truth here. Do not re-litigate a decision; if implementation exposes one as wrong, mark the *planning* ticket undermined and raise it, rather than quietly deviating. Beyond that, capture whatever a fresh session on this map needs and nothing narrower would surface on its own: the reading order (spec, then this map, then the ticket, then any approved asset it names); the vocabulary source, if a `CONTEXT.md` exists; the review or verify skills to use before a commit; the per-ticket static checks and test commands; anything about the dev environment a fresh session needs; and a map linter's invocation, if your environment wires one.
- **Decisions so far** stays empty until the first ticket ships.
- **Not yet specified** stays empty — every decision is settled in the spec; this map only executes it.
- **Out of scope** is inherited from the spec's own Out of Scope, one line per item ruled out and why; these never graduate into tickets on this map.
- **Each ticket's Question**: what this ticket makes work, end to end, from the user's perspective — the tracer bullet, not a layer-by-layer implementation list. Name the seams and symbols the spec already fixed, since they are decided; avoid brittle line-level file paths that go stale. Where the spec's Testing Decisions call for tests-first, say which test leads and that it lands red before the implementation.
- **Each ticket's Done when**: the observable condition that proves the slice shipped — the tests that go green and the behaviour a human can verify.

No acceptance-criteria checkboxes: a ticket's status is *derived* your environment's way — an answer resolves it — so a checklist would be a second copy of "Done when" that goes stale in one home.

### 6. Verify before committing

Verify what you wrote against your environment's file-format convention — edges name tickets that exist and form no cycle, numbers are used once, no ticket carries a closing section yet, the map states no progress count. If your environment wires a map linter, run it. Then commit the map and tickets together.

## Working the map

Work it in fresh sessions exactly as wayfinder works any map — one frontier ticket per session, claimed before work, verified against your environment's convention before commit. Some environments automate that driving; by hand, follow the `wayfinder` skill's "work through the map". What is specific to shipping code: run the project's static checks and tests, review the diff, and drive the real behaviour where "Done when" is only real at runtime — then **resolve by shipping**, appending the answer with what shipped plus a gist + link to the map's Decisions so far. Writing that answer *is* what resolves the ticket, so progress is read from the map, never written down as a count.
