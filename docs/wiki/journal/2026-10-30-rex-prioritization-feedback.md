---
title: Rex prioritization form feedback
section: Journal
updated: 2026-10-30
status: Naming policy accepted; implementation plan prepared
---
# Rex prioritization form feedback

The user requested an interview to select proposals from the
[Rex analysis](../discussions/rex-beta-adoption.md). The
[first response](2026-10-29-grill-8f14b5c8-c08d-4baf-84bc-599282c28304.md)
requested individual choices instead of grouped directions.

The second submitted form, session `40c4405b-73cf-4ff9-93de-d2c13615fc56`,
recorded this feedback on question 1:

> i want to select multiple

Its final preview state selected proposal **#1, stable task names for individual
terminals**. Questions 2–4 (first-deliverable scope, next stage and workflow
example) were untouched. The [original form and submitted response](../evidence/2026-10-30-rex-prioritization/individual-selection-feedback.json)
preserve that distinction.

Interpretation: redisplay all 15 proposals as independent checkboxes, carrying
#1 forward as an editable selection. This feedback requests a broader selection
mechanism; it does not settle the shortlist, deliverable scope or implementation
authorization. The remaining questions stay open.

The installed jstack package changed from 0.2.1 to 0.3.0 after collection and
before its old `finish` command could run. The old session is retained, and its
source and submission are archived above. The corrected round uses the current
jstack grill-form assembler and Fungi delivery workflow. No Chartr product
behavior was changed or verified by this interview.

## Accepted scope and next stage

Fungi session `c7ade4c949fff1d45786afdc4d47d041` returned a valid submitted
`grill-form/v1` result. The [saved response](../evidence/2026-10-30-rex-prioritization/scope-and-next-stage.json)
selects **one focused feature, finished end to end** as the first-deliverable
scope and **explore design and feasibility, then bring back a concrete plan**
as the next stage. These choices are accepted in
[DEC-REX-001](../decisions/index.md#dec-rex-001-explore-one-focused-feature-before-implementation).

The proposal question returned `status: untouched` with empty `states`, notes
and comments. It does not establish a submitted shortlist, even though the form
carried forward #1 as its initial editable preview state. The optional workflow
example was also untouched. Reopen only the shortlist question with independent
checkboxes and no initial selections; do not repeat the accepted scope questions.

A separate disposable control test selected two checkboxes and returned
`states.main.selected: [1, 2]` through the same installed jstack/Fungi preview
mechanism. This establishes that the tested control can retain multiple values;
it does not establish why the user's prior response contained no preview state.
The test answers are synthetic and are not user preferences.

## Selected target

Fungi session `b37909d043da533dc224f6aa62d7a7c9` returned a valid submitted
`grill-form/v1` result with `states.main.selectedProposals: [1]` and the matching
title **Stable task names for individual terminals**. It contains no notes or
other selections. The [saved shortlist](../evidence/2026-10-30-rex-prioritization/selected-shortlist.json)
settles the target for exploration, recorded in
[DEC-REX-002](../decisions/index.md#dec-rex-002-select-terminal-task-names-for-exploration).
Other Rex proposals were not selected in this round; that does not permanently
reject them.

The [source investigation and proposed plan](../discussions/terminal-task-names.md)
maps the existing title, dialog, identity and persistence boundaries. Manual names
fit the current host architecture. Automatic conversation titles and notification
names extend behavior beyond the basic override. A focused follow-up explores
those choices, name lifetime and provider presentation. No implementation is
authorized or claimed by this selection.

## Accepted naming policy

Fungi session `d6d3d0a7e0eadae104564a875c49ad4a` returned a valid submitted
`grill-form/v2` result. All four questions have one selected choice and
`status: feedback`. The [saved design answers](../evidence/2026-10-30-rex-prioritization/naming-design-choices.json)
record:

| Question | Submitted answer | Interpretation |
| --- | --- | --- |
| 5 — Automatic names | Use available conversation titles automatically, with process/shell fallback. | Include automatic titles in this feature; manual names remain the override, as stated in the question. This supersedes the earlier manual-only recommendation. |
| 6 — Manual-name lifetime | Keep until renamed or reset; note: “this could go both ways. make it a setting.” | Provide both keep and reset-on-new-conversation behaviors as a setting. The plan defaults to Keep, following the selected choice; the note must not be discarded as acceptance of a fixed keep-only rule. |
| 7 — Provider display | Provider icon + task name. | Keep provider text in detail rather than repeating it in every tab label. |
| 8 — Notifications | Terminal labels first; notification naming later. | Task names in desktop notifications are explicitly deferred. |

[DEC-REX-003](../decisions/index.md#dec-rex-003-automatic-task-names-with-configurable-manual-name-lifetime)
records the accepted policy. The [implementation plan](../discussions/terminal-task-names.md)
now includes automatic title fallback and the lifetime setting. Global Terminal
settings placement, a keep default, known-identity change detection and fallback
rules are concrete implementation proposals derived from these answers, not
additional quoted user requirements. Existing metadata does not identify every
conversation change, so automatic reset cannot be promised for every agent.

The scoped interview is complete. No further preference question is needed to
deliver the plan. Implementation remains outside the selected exploration phase;
no product code, runtime behavior or test result changed during this interview.

## Implementation authorization

After the design/feasibility handoff, the user said “proceed with implementation,
use work-until-grill.” This explicitly advances the selected task-name feature
into implementation. [DEC-REX-004](../decisions/index.md#dec-rex-004-implement-terminal-task-names)
records the authorization; the [implementation run](../verification/runs/2026-10-30-terminal-task-names.md)
contains actual checks and limitations. Earlier planning-only answers above are
preserved as the preceding phase.

## Implementation outcome

Terminal task names are implemented in the working tree with the accepted
precedence, manual lifetime preference, context-menu/palette editing, and durable
terminal-ID ownership. The normal desktop build, 450 workspace tests, three
isolated live-backend checks, plugin-example check, formatting, Python and
JavaScript checks passed. Native desktop inventory failed repeatedly, including
the user's “try again” request and a reset of the automation session. The
[run](../verification/runs/2026-10-30-terminal-task-names.md) therefore retains an
aggregate Untested verdict for the selected native rename/focus/reopen scenarios.
No product decision required another grill. No commit, release or installed-app
replacement was performed.

## Project suffix follow-up

The user supplied a sidebar screenshot showing `Say hello | imparo` under the
`imparo` space and requested omitting the project-name suffix. Automatic terminal
titles now strip the trailing ` | <working-directory name>` when it matches that
known directory. The displayed example becomes `Say hello`. Manual names and
other pipe-separated task text remain literal. Focused verification is appended
to the [implementation run](../verification/runs/2026-10-30-terminal-task-names.md#project-suffix-follow-up).
