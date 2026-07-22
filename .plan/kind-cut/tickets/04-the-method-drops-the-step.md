---
type: task
blocked_by: [03]
---

# The method drops the step

## Question

Stop asking map-creating sessions to record a kind, strip the declarations that
are now dead config, and record the reversal where the reasoning lives. This is
the ticket that actually pays for the cut: every prior one deleted code, this
one deletes an obligation.

**`docs/wayfinder-adapter.md`.** The whole "On creation, record the map's kind"
section, the "Graduating a planning map to implementation" subsection, and "The
fallback, if this step is skipped" all go. That leaves a document whose stated
purpose — "This file adds exactly one chartr-side step" — has no step left in it.
Decide honestly between deleting the file and reducing it to a short note that
this repo's maps are plain local-markdown with no chartr-side additions, and say
which you chose and why. If it goes, remove the reference from `CLAUDE.md` too;
if it stays, it must not describe a step.

**`CLAUDE.md`.** The "Wayfinder maps in this repo" section exists almost entirely
to mandate the kind declaration. Reduce it to what remains true: this repo plans
with wayfinder maps under `.plan/`, and the chartr watches this space. No
recording step, no ADR 0007 gate, no "stays inert until a human confirms".

**`.chartr/config.toml`.** Remove all seven `[maps."…"]` tables — the six that
predate this effort and `[maps."kind-cut"]`, this map's own. Ticket 03 made them
inert; this makes them gone. Append nothing.

**`.plan/chartr-design/spec.md`.** Amend only what kind touches — the review-era
staleness elsewhere in this spec is pre-existing debt and explicitly out of
scope:
- The "Maps, kind, and discovery" heading (~:30) loses `kind`.
- **Strike stories 13, 14, 15 and 16** (~:34–37) — inert-until-classified, the
  pre-filled guess, committed kind for teammates, and noticing graduation *for
  classification*. Story 16's surviving half (the chartr notices a new map
  directory) is real and should be restated rather than lost.
- Line ~123 drops `classify` from the list of operator actions.
- Line ~132 — the whole map-kind paragraph — is struck and replaced by the rule
  that replaced it: a ticket's `type:` selects its role; a discovered map is
  live.
- Line ~156 drops "map kinds" from what committed workspace config holds.
- Lines ~186 and ~189 drop `classify` and "discovery and classification" from
  the tested surface; keep discovery.

Note each strike so the change is legible rather than silent — the precedent is
the `needs-you-cut` answer's treatment of story 63.

**Supersede ADR 0007 — do not delete it.** Write `docs/adr/0015-*.md` (0014 is
the highest today) recording that kind is removed, and why: its deciding premise
was struck by its own amendment when review was cut, and the ground the amendment
kept — kind selects the role set — was redundant with per-ticket `type:`, which
states the same thing exactly rather than by map-uniform approximation. Name what
was consciously given up: the inert-until-classified gate and the teammate-level
agreement it carried. Add a superseded pointer at the top of 0007; leave its body
and its amendment intact as the record of how the decision moved. Check the other
ADRs for cross-references to 0007 and fix any that would now mislead.

Done when: `git grep -in "map kind\|classify\|unclassified" -- docs/ CLAUDE.md .chartr/ .plan/chartr-design/spec.md`
returns only the deliberate historical references (ADR 0007's own body, ADR 0015,
and resolved tickets' answers); `.chartr/config.toml` has no `[maps.*]` table;
`go vet ./...` / `go test ./...` and the frontend gates are green; and a freshly
registered space in a running cockpit renders every map live with no
classification step anywhere in the UI.

## Answer

**The obligation is gone.** No session charting a map in this repo is asked to
record anything, anywhere. `docs/wayfinder-adapter.md` is deleted, `CLAUDE.md`'s
section is four sentences of what stayed true, all seven `[maps."…"]` tables are
gone, and ADR 0015 records the reversal with 0007 superseded rather than
rewritten.

**`docs/wayfinder-adapter.md` is deleted, not reduced.** The file's own first
paragraph states its purpose — *"This file adds exactly one chartr-side step, on
one event"* — and there is no step. What decided it against a short "this repo
adds nothing" note: such a note is a pointer that must be followed to learn it
points nowhere, and, worse, it keeps an *adapter* concept alive as a hook for
the next chartr-side step to hang on, which is the opposite of what this cut
decided. A session following the vendored `tracker-convention` skill with no
adapter present gets exactly the right behaviour — plain local-markdown — by
doing nothing. The reference is out of `CLAUDE.md`, which keeps one sentence
saying the maps are plain local-markdown with nothing on top, so "does this repo
add anything?" still has an answer where a session already reads. Nothing else
in the repo points at the file (the three `.plan/` hits are resolved maps'
prose — history, left alone), and the vendored skills never referenced it.

**`.chartr/config.toml` is deleted, not emptied — a judgement call the ticket
did not spell out.** Map kind was this repo's *only* committed workspace config:
no `[roles.*]`, no anything else. Removing the seven tables leaves a zero-byte
tracked file whose presence implies content it does not have. Ticket 03 pinned
"a space with no `.chartr/config.toml` at all" as fully supported at both the
resolution level and the process boundary, so the file goes. Verified live
rather than assumed: this repo registered into a fresh cockpit now resolves with
**zero warnings**, and the settings surface reports the layer honestly —
`{"name":"workspace-config","path":".../.chartr/config.toml","exists":false}` —
which is exactly the absent-layer behaviour ADR 0014 specifies. `.chartr/skills/`
is untouched and the directory stands.

**The spec's strikes hold the numbering.** Stories 13, 14 and 15 are struck in
place, each with its own note saying what it asked for and why that is gone; 16
is restated around its surviving half (noticing graduation) with a parenthetical
naming what was struck, following the `needs-you-cut` map's treatment of story
63. Renumbering sixty stories to erase three would break every reference to them
for no gain. The heading dropped `kind`, line 132's map-kind paragraph is
replaced by the rule that replaced it, and `classify` is out of the operator
actions, the tested operator surface, and the tested behaviours — where
*discovery* stays but "inertness until classified" and "the dangling rename" go,
the latter because with no slug-keyed config entries there is nothing left to
dangle.

**Only kind, on line 189.** That line said "the lifecycle end to end **per map
kind** (planning tickets resolve directly; implementation tickets walk
implementing → proposed → agent review → human review → resolved)". The
done-when grep forced a touch here, so it lost `per map kind` and *nothing else*
— the parenthetical still describes agent review and the human review hub, which
is the pre-existing review staleness the map put explicitly out of scope. It
reads a little oddly as a result, and that is the honest state: this cut does not
own it.

**Cross-references fixed by amendment, not by edit.** This repo's ADRs correct
themselves at the bottom and leave their bodies whole — 0007 has an addendum and
an amendment, 0009 has two. So 0009 and 0014 each gained a short amendment
naming what lapses: 0009's "the committed config file gains a second tenant
beside map-kind" (now the other way round — bindings and skills are the tenants,
and a space may have no config file at all), and 0014's "map kind stays
classify-only" (no subject left; its real content, *this surface edits role
bindings and nothing else*, is unchanged). 0014's amendment also settles what
ticket 03 flagged: the `SetUserBinding`-is-harder-than-`DeclareMapKind`
comparison is restated in its own terms now that the function is deleted.
**0009's rejected "committed autopilot, confirmed on clone (mirroring ADR 0007's
declared-but-confirmed map-kind)" is left verbatim** — a Considered-options entry
is frozen deliberation, a record of what was compared at the time, and editing it
would be rewriting the reasoning rather than correcting a claim. A reader who
follows it now lands on 0007's superseded banner, which is the right outcome.

**`CONTEXT.md` was swept too, beyond the ticket's named files.** Its **Kind**
glossary entry defined map kind in the present tense as a live property that
decides which roles a session may spawn as — and this glossary is injected into
*every* context bundle, so it was teaching the deleted concept to every session
spawned after this cut. That is kind debt, not the review-era debt the map put
out of scope, so it is fixed: the entry is deleted, **Role** now states that a
role follows from the ticket's own `type:` with all four offered at the gate, and
**Workspace config**'s tenants are corrected to bindings and the committed skills
layer. Flagged rather than silent because the ticket enumerated its files
precisely and this was not among them.

**The done-when grep returns more than its three named sources, and each extra
is ticket-mandated.** Beyond ADR 0007's body and ADR 0015: the spec's strike
notes (the ticket asks that each strike be *noted*, which cannot be done without
naming what was struck) and the 0009 / 0014 amendments (the ticket asks that
misleading cross-references be *fixed*, and an amendment must name what it
corrects). Also ADR 0014's body lines 9 and 24, deliberately left standing under
their amendment per the paragraph above. No live instruction, config, or UI
string survives anywhere.

**The eyes-on cockpit pass is not met — the browser extension is not connected**,
the same blocker ticket 02 hit. What was verified instead, against a real running
binary built from this working tree: a **freshly registered space** — a temp git
repo with a map, two tickets (`task` and `grilling`), and no `.chartr/` directory
at all — arrives in the control-socket snapshot with both tickets on the
frontier, no `kind` or `kindGuess` key anywhere in the payload, and no warnings;
this repo registers the same way with all seven maps live; the route table has no
classify endpoint; and the built `dist/` JS and CSS contain **zero** occurrences
of `classif`, `unclassified`, `kindGuess` or `Map kinds`, as does `web/src`. The
one thing left unconfirmed is that the rendered pixels match, which ticket 02's
verification of the same surface already covers.

Verified: `go vet ./...` and `go test ./...` green, `svelte-check` 0 errors,
`vitest` 85 passing, `npm run build` clean, no amber in the built CSS. This
ticket changed no Go or frontend code, so the gates confirm no collateral damage
rather than a change.
