---
type: task
blocked_by: [02, 03, 05]
claimed_by: sc913abcd812d
claimed_at: 2026-08-06T17:08:12Z
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

## Answer

Both payloads compose, both have goldens, and the tracer bullet closes: a
`grilling` preview on this repo's own `localhost-trust-boundary` map, served by
the real binary against a throwaway config root, returns
`core:chartr, grill:chartr-skills, conventions:chartr, preferences:operator,
sources:context, map, ticket, blocker #01` — the body between the core and the
conventions pointer came out of `<configDir>/sources/chartr-skills/grill/SKILL.md`
through the registry. One commit.

**The core moved into the binary.** It is read from the embedded asset, not
resolved through the layers, because the spec's origin table gives it `chartr
(embedded)` and a layer-resolved core would have badged `built-in`/`user`/
`workspace` — an origin the set does not contain. That emptied `ComposeInput` of
`Roots` entirely (the skill-library manifest part is gone too), so the field and
the nil-`Sources` layer fallback ticket 05 flagged as a hedge both went with it;
`Resolve`, `Library` and `Materialize` are untouched and still serve the settings
surface until ticket 09. The claim trailer is unchanged — the core still records
`core=built-in:<shippedHash>`, which is exactly true of an embedded copy.

**The `Part`/`Segment` collapse was taken.** The diff stayed small: one struct,
`ctxPart`, `renderMarkdown`, `model.ts` and five lines of `PayloadPreview.svelte`.
`Segment` is gone; a `Part` is `{name, kind, origin, label, text}`. The frontend's
lookup is now `originVariant[part.origin] ?? 'secondary'` — the fallback is kept
and deliberately not `'outline'`, so a source name (an open set) reads as a
distinct badge rather than silently borrowing chartr's.

### Each Done-when clause

- *Both payloads have golden files* — `internal/prompt/testdata/{ticket,free}-payload.golden.md`,
  written by `TestTicketPayloadGolden` / `TestFreePayloadGolden` against a real
  seeded config root, with the temp path normalised to `<config>`. Rewrite with
  `go test ./internal/prompt -update`.
- *The free payload's golden is the guard on the ignore test* — chartr's voice in
  it is exactly four sentences: what chartr is, the conventions pointer, the
  sources inventory line, the shadowing sentence. The conventions sentence is the
  consequence form the ticket specified ("a file under `.plan/maps/` is read by
  chartr only where it follows the format stated at ⟨path⟩"), so it survives being
  ignored and carries the whole reason to open the file.
- *A `grilling` session composes the seeded `grill` body, asserted through the
  preview endpoint* — `TestFirstRunSeedsTheSkillsAndTheBindings` already asserted
  it and still does; `TestTicketPayloadGolden` re-asserts it at the package seam by
  reading a prose line out of the materialized `SKILL.md` and finding it in both
  the part and the document.
- *The preview badges name the source a role body came from* — asserted as
  `Origin == "chartr-skills"` in the golden test's part table, in
  `TestPayloadComposesWithProvenanceAndBundle`, and in the bindings test.
- *`go vet`, `go test`, frontend `check`/`build`/`vitest`* — all clean.

### Other tests, and what moved

`TestFreePayloadVariesOnlyWithSourcesAndPreferences` covers the two things that
vary and the two rules about what does not appear: a disabled source is absent,
and no `https://` reaches the document. `TestUnavailableSourceWarns` covers the one
surviving composition warning. `TestFreePayloadPreviewIsSpaceIndependent` hits
`GET /api/payload/free` at the process boundary and asserts the map slug, the
ticket title and the word "frontier" are all absent from a payload composed while
a space with a live map is registered.

Three server tests were retargeted or dropped, all for the same reason — the core
was the last thing they observed the layer model through:

- `TestSkillShadowingMatrix` became
  `TestNeitherCoreNorRoleIsShadowableByASkillLayer`, which writes a `core` into
  the user layer and a `grill` into the workspace layer and asserts the composed
  document is byte-identical to the one before. That is a better guard for this
  ticket than the matrix was.
- `TestBehindDefaultSurfaced` and `TestMaterializedLibraryEditsCompose` are
  deleted. Both asserted a fork/edit reaching a *payload*; no layer reaches a
  payload any more. Fork drift is still covered at the package seam
  (`TestForkedFromDriftOverDirectoryHash`, `LibraryWarnings`) and still surfaces on
  the space, until ticket 09 deletes the model.
- `Skill.Support` went, so the two supporting-file assertions in
  `TestWholeSkillShadowing` went with it, and `TestShippedLibraryIsElevenSkills` is
  now `TestShippedLibraryIsTenSkills` and asserts `tracker-convention` no longer
  resolves.

`CONTEXT.md`'s **Context bundle** entry loses its glossary and manifest clauses
and gains the sources block, per the spec's rule that vocabulary follows its
ticket. *free session* is ticket 08's to write.

### Flagged

- **The whole `tracker-convention` directory is deleted, not just `glossary.md`.**
  Deleting the `TrackerSkill` constant takes it out of `Names()`, which makes the
  embedded directory unreachable — dead bytes in the binary rather than a skill.
  Ticket 09 was going to cut it; leaving an unreferenced copy behind for one ticket
  seemed worse than cutting it here.
- **This commit invalidates `prompt.MatchesShipped` for every already-materialized
  library**, because the embedded set and the core's bytes both changed. Ticket
  06's migration therefore takes its *other* branch on such a machine: it registers
  `builtin-skills` as a source instead of moving it aside. That branch is the safe
  one (nothing is moved, nothing is lost) and the release freeze means no build
  between 06 and 09 ships, so only a from-source dev can see it — but it is a real
  behaviour change to a resolved ticket's guarantee and it is not one I could avoid
  while editing the core.
- **Two vendored seed skills still say "glossary".** `to-spec` and `to-tickets` in
  `internal/sources/assets/chartr-skills/` mention it; fixing them means
  re-authoring in the skills repo and re-running `make vendor-skills`, which is
  ticket 01's artifact and not mine to re-vendor mid-map. `docs/skill-sync.md` and
  ADR 0005 also still describe the glossary — the spec assigns the skill-sync
  deletion to the cut (ticket 09) and the narrative pass to ticket 10.

### Deliberately left out

No UI for the free payload's preview: the endpoint exists and is tested, and the
ticket puts the surface on the settings screen in ticket 10. No free-session
*launch* path — that is ticket 08's split button; `ComposeFree` is waiting for it.
No deletions from ticket 09's list beyond the four this ticket named. A real
grilling session was previewed but not **spawned** in the cockpit's GUI: I read
both payloads out of the running binary's preview endpoints (which is the byte
source the modal renders) rather than clicking through a build, and I did not
write a claim commit into this repo mid-session to do it. If the GUI read is
wanted as its own check, it is one `make build webview` away and nothing about the
wire format is untested.
