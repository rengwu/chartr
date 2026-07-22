---
type: task
blocked_by: [02]
claimed_by: s46b8beb325ff
claimed_at: 2026-07-22T03:48:44Z
---

# Revert to the vanilla lifecycle

## Question

With the review feature gone, retire the `proposed` status itself and the last
gate-shaped semantics, so the derived-status model becomes exactly wayfinder's.
Small and isolated on purpose — this is the dangerous half (a half-done parser
misreads every ticket's status, this map's own included), kept apart from ticket
02's bulk so it lands in one clean, green commit.

- **Parser** (`internal/wayfinder/parse.go`). Remove `StatusProposed`,
  `HasProposedAnswer` / `ProposedHeading`, and their `Derive()` branch; revert
  `Frontier()` to vanilla — resolved blockers unblock, with no approval and no
  unclaimed condition; `doc.go` loses "the chartr's one addition." An in-flight
  `## Proposed Answer` is now an **unknown heading**: a ticket carrying it derives
  `open` (or `claimed` if a claim marker survives), never `resolved` — ignored,
  not migrated.
- **The context bundle stops treating `## Proposed Answer` as an answer.** This is
  the settled *ignore, don't tolerate-as-answer* decision (spec → Implementation
  Decisions; planning ticket 01 rejected tolerate-as-answer because it would
  silently bless answers no human approved). In `compose.go`, `AnswerSection`
  drops its `Proposed Answer` fallback — a blocker's answer reads `## Answer`, else
  `## Ruled out`, and **never** an in-flight `## Proposed Answer` — so a dependent
  is never handed an unblessed proposal as though it were the answer. Verify no
  other reader on the blocker-answer path (the context-bundle assembly, the ticket
  pane's inline blocker answers) still falls back to it.
- **Config** (`config/binding.go`). Remove the entire `autopilot` resolution — the
  fields, the warning, `Resolution.Autopilot` — confirmed resolved-and-consumed-
  nowhere, deleted without replacement.
- **Star-map.** Remove the **base** `proposed` star and its palette (`theme.ts`
  proposed hue and the base state) now that the status is gone.
- **Docs & ADRs.** Strike the review vocabulary from `CONTEXT.md` (`proposed`,
  `agent review`, `human review`, `review brief`, `abandon`, `autopilot`) and
  redefine `resolved` ("the session said so", not "always blessed"); update
  `docs/design-system.md`'s star-map grammar; write the **ADR 0004 / 0008 / 0009
  amendments** and strike **ADR 0007**'s lifecycle-selection premise (kind still
  selects the role set).

Done when: `internal/wayfinder` tests assert `Derive()` yields only open /
claimed / resolved / out_of_scope, that a `## Proposed Answer` derives open or
claimed (never resolved), and that `Effort.Frontier()` unblocks a dependent the
instant its blocker resolves; a `compose` test asserts a blocker carrying only
`## Proposed Answer` contributes **no** answer to a dependent's context bundle;
`go vet ./...` / `go test ./...` and the frontend gates are green; and ADRs 0004 /
0008 / 0009 are amended with 0007's premise struck.

## Answer

The derived-status model is exactly wayfinder's again. `proposed` is retired as a
status, the last gate-shaped semantics are gone from the parser, the context
bundle and config, and ADRs 0004 / 0008 / 0009 carry amendments with 0007's
lifecycle premise struck. Two commits: the code revert (green on its own), then
the vocabulary and ADRs.

**Parser** (`internal/wayfinder`). `StatusProposed`, `HasProposedAnswer` /
`ProposedHeading`, their parse in `ParseTicket` and their `Derive()` branch are
deleted; `doc.go` drops "the only chartr-specific addition." An in-flight
`## Proposed Answer` is now an unknown heading — `sectionRange` matches heading
text exactly, so it never collides with `## Answer` — and its ticket derives
`open`, or `claimed` if a claim marker survived. Ignored, not migrated.

**`Frontier()` needed no code change, and that is the honest report.** Its
blocker test was already `dep.Status != StatusResolved` with an `open`-only
outer filter (open implies unclaimed, since a claim derives `claimed`), so the
strictness lived entirely in `proposed` being a status that is neither resolved
nor open. Retiring the status *is* the revert: a dependent now unblocks the
instant its blocker's `## Answer` lands, with nothing to approve in between. The
doc comment now says so rather than implying a hold.

**The context bundle ignores a proposal.** `compose.AnswerSection` is
`firstSection(body, "Answer", "Ruled out")` — the `Proposed Answer` fallback is
gone, so a blocker carrying only a proposal contributes *no* answer and Compose
renders its explicit "not resolved" note instead. I swept the whole
blocker-answer path for other fallbacks: `DetailPane.svelte`'s `ANSWER_SECTIONS`
(the ticket pane's inline blocker answers) is now `['Answer', 'Ruled out']`, and
`markdown.ts`'s generic `sectionOf` takes its names from callers — the only two
callers are those. `markdown.test.ts`'s fallback case was re-pointed at
`['Ruled out', 'Answer']`, so it still tests first-match-wins without asserting
the retired heading.

**Config.** The whole autopilot resolution is deleted without replacement: the
workspace and user `autopilot` fields, the committed-flag warning, and
`Resolution.Autopilot`. `parseUser` returns just the bindings again, and
`TestCommittedAutopilotIgnoredWithWarning` went with the warning it asserted. An
`autopilot` key in either layer is now an unknown key: ignored, unwarned.

**Star-map.** The base `proposed` star leaves `theme.ts` — the `VisualState`
member, its `STAR` and `LABEL` entries, and its `visualState()` case — leaving
five states. No renderer was touched (ADR 0010); `starmap.test.ts`'s
all-base-states and lifecycle fixtures were updated to five, not deleted.

**Docs & ADRs.** `CONTEXT.md` loses `proposed`, `agent review`, `human review`,
`review brief`, `abandon` and `autopilot` as terms; `resolved` is redefined to
"the session said so"; `Frontier` reverts to wayfinder's; `Role` loses `review`;
`Kind` now selects the role set, not a lifecycle; and the tagline no longer
promises a gate. `docs/design-system.md`'s star-map exemption is five hues.
**ADR 0004** amended (derived state survives; the `## Proposed Answer` extension,
promotion-at-gate and the stricter frontier withdrawn; the containment forfeited
knowingly, replaced by social visibility). **ADR 0008** amended (write set shrinks
to claim + release; the approval race and the hub's revert levers lapse).
**ADR 0009** amended (mechanism untouched; the autopilot bullets and the
heterogeneity consequence lapse). **ADR 0007**'s lifecycle-selection premise
struck, the decision standing on role-set selection.

**Two things beyond the ticket's letter, both the same rule it already applies.**
The *injected glossary* (`internal/prompt/assets/glossary.md`) still defined
`## Proposed Answer` and still said a blocker "counts as cleared only when its
answer is written *and* a human has approved it" — the stricter-frontier
definition, handed to every session at spawn. Ticket 02 retargeted `implement.md`
and `core.md` for exactly this reason; leaving the glossary would have taught
each new session the lifecycle this ticket just deleted. Both entries are
corrected. `ideate.md` lost "proposed" from its list of things nothing there
derives. Separately, stale comments naming the removed mechanism (`model.go`'s
wire docs, `halt.go`'s respawn guard, `terminal`'s quiet-verdict docs,
`spaces.go`'s user-config doc) were corrected in passing — comments only.

**Tested.** `internal/wayfinder`: `Derive()` yields only the four vanilla
statuses across every closing/claim combination (including bare headings);
`## Proposed Answer` derives `open`, and `claimed` when a claim survives, never
`resolved`; and resolving a blocker puts its dependent on the frontier on the
next read. `internal/server`: the derived-status test at the wire now expects
`open` for a proposal-carrying ticket, and a new payload test asserts a blocker
carrying only `## Proposed Answer` contributes no answer — the prose does not
appear in the part or the composed markdown, and the part reads "not resolved."
`go vet ./...` / `go test ./...` green; frontend `check` / `build` / `vitest`
green (53 tests); no amber in the built CSS (every hue in it is either the
~107 monochrome ramp or `--destructive`).

**Verified against the real binary**, on a second instance pointed at this repo
(port 8811, its own data dir, so the operator's running cockpit was untouched):
with this ticket claimed and unanswered, ticket 04's payload preview rendered
`blocker #03` as `_(no answer yet — this blocker is not resolved)_`, and the
injected glossary in that same payload carried the corrected frontier wording.
After this Answer was committed, the same preview inlined it with no gate in the
path — the unblock is immediate.

**Deliberately not done.** No migration of in-flight `## Proposed Answer`
sections anywhere (the settled ignore-don't-tolerate decision); no touching of
the old maps' text under `.plan/`, which stays a historical record; no new
deterministic check to replace the forfeited containment — that is the cut's
whole point, and the spec rejected it. `prompts/review.md` still sits in this
repo's *untracked* local data dir from a pre-cut run; it is not in git and is not
this ticket's to sweep.
