---
type: task
blocked_by: [02, 03, 05]
---

# The two payloads

## Question

Compose the two documents chartr hands an agent. **The tracer bullet completes
here**: one `grilling` ticket session spawns, and the body between its core and its
conventions pointer came out of `<configDir>/sources/chartr-skills/grill/SKILL.md`,
resolved through the registry. Demonstrate it rather than assert it — spawn it on
this repo's own next map and read the composed payload in the cockpit's preview.

Five parts, four shared. Order is core, role, conventions, preferences, then the
context region — **instructions, then data**. The sources block is *context*, so in
both payloads it renders below the `# Context` rule `renderMarkdown` already draws,
which is why preferences remain the last instruction bytes in both.

**The free payload carries no live facts about the space** — the same bytes in an
empty tree and a tree mid-effort. No map list, no frontier, no branch for a space
without `.plan/`. The agent is already in the tree and can list a directory, while a
list composed at spawn is wrong the moment another session resolves a ticket and a
free session outlives that snapshot. What varies is the sources block and the
preferences bytes, and that is all.

**Hold chartr's own voice to the ignore test:** every sentence chartr writes must
still be true if the agent does nothing about it. That admits a location and an
inventory and rejects the next addition, which will almost always arrive phrased as
a *should*. It forces consequence-framing, and that is the technique rather than a
loophole — chartr may not say "read the conventions"; it may say *"a file under
`.plan/maps/` is read only where it follows the format stated at ⟨path⟩"*. Both
point at the same file; only the second survives being ignored, and it carries the
entire reason to open it. `preferences.md` is exempt: it is the operator's voice,
unwrapped, unlabelled and unranked, and it is permitted to contradict the
conventions.

The sources block renders identically in both: enabled sources in file order,
default row last, each with its **local checkout path and its skill names,
comma-joined, no descriptions**. Names fall out of the walk for free; descriptions
would cost one file read per skill at every compose. **Never print a git source's
URL** — it is a fetchable address in a payload handed to an agent running with
permissions skipped, on an effort whose standing decision is that nothing fetches
unattended. Disabled sources do not appear. Close with the shadowing sentence,
stated as a fact and naming the qualified form.

The ticket payload keeps today's shape with two edits: the core's sentence listing
what was assembled **drops its glossary clause** — a core that still promises it is
lying to every session — and **the role body is concatenated, not pointed at**,
which is what makes the claim trailer's skill record mean *this text ran*. No origin
line above the body. **Delete `glossary.md`** and the bundle's glossary part, along
with `TrackerSkill`, `GlossaryFile` and `Skill.Support`; `Bundle` itself is
unchanged, because it never held the glossary. The `ctxPart` list becomes `sources`,
`map`, `ticket`, blockers for a ticket payload and `sources` alone for a free one.

The preview keeps provenance, re-pointed: `Segment.Layer` becomes `Segment.Origin`,
a free string — `chartr` for the embedded cores and the conventions pointer, *the
source's registered name* for a resolved skill body, `operator` for preferences,
`context` for assembled parts. This is the badge that answers the one silent failure
source order can cause. The frontend's `layerVariant` lookup keeps its `?? 'outline'`
fallback so an open string set degrades rather than breaks. **The free payload gets
a preview too**, space-independent, four parts, no ticket or role selector; it hangs
off the settings surface in ticket 10, not the space card. Composition warnings
survive with one named case: a source whose status is `unavailable`.

**`Part` now always holds exactly one `Segment`** — it did under whole-skill
shadowing too, and the multi-segment machinery was built for a per-field merge that
never shipped. Collapsing them is a real simplification that touches the wire format
and the frontend; the spec flagged it for this ticket to decide. Take it if the diff
stays small, skip it and say so if it does not.

## Done when

Both payloads have golden files and a `grilling` session composes a payload
containing the seeded `grill` body, asserted through the preview endpoint. **The
free payload's golden is the only mechanical guard on the ignore test** — chartr's
own voice there is four sentences and nothing else will notice a fifth, so a diff on
that file is the review. The preview badges name the source a role body came from.
A real grilling session spawned on this repo's next map reads correctly in the
cockpit. `go vet`, `go test`, and the frontend `check`/`build`/`vitest` pass.
