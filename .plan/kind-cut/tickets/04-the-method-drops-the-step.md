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
