---
title: Decisions
updated: 2026-11-07
---
# Decisions

## DEC-RELEASE-033-002 — Publish the prepared 0.3.3 release

Accepted November 7: after preparation completed and its results and limits were
reported, the user said “lets publish.” Publish the prepared revision as stable
v0.3.3 using the established tag, package and GitHub release workflow. The
[accepted verification scope](#dec-release-033-001-prepare-0-3-3-with-simple-verification)
continues to apply: do not take over the desktop or add the skipped upgrade test.
AUR publication remains deferred and disabled. This authorization does not turn
unexecuted native scenarios into passing evidence.

## DEC-RELEASE-033-001 — Prepare 0.3.3 with simple verification

Accepted November 7: the user said “skip #1. verify others in a simple manner
(i don't want my desktop to be hijacked), then prepare the release,” followed by
“bump it to 0.3.3.” Item #1 was the proposed v0.3.2-to-candidate upgrade test.

Prepare 0.3.3, omit that upgrade test, and use bounded checks that preserve the
user's desktop. Existing headless UI/model tests, private real-backend sessions,
historical visual evidence and candidate CI/package checks are the chosen method.
This does not establish native banner/click or physical-input behavior as passed,
nor authorize publication. No new naming-policy choice is needed. See
[preparation](../development/release-0.3.3.md) and the
[run](../verification/runs/2026-11-07-release-0.3.3.md).

## DEC-REX-001 — Explore one focused feature before implementation

Accepted October 30 through the
[prioritization answers](../journal/2026-10-30-rex-prioritization-feedback.md#accepted-scope-and-next-stage):
target one focused feature, finished end to end, for the first deliverable.
The immediate next stage is design and feasibility exploration followed by a
concrete plan. The proposal shortlist was unresolved in this answer and is
subsequently settled by DEC-REX-002. These choices authorize
exploration and planning; they do not select a feature or authorize product
implementation.

## DEC-REX-002 — Select terminal task names for exploration

Accepted October 30 in the [submitted shortlist](../journal/2026-10-30-rex-prioritization-feedback.md#selected-target):
explore proposal **#1, stable task names for individual terminals**. It was the
only selected item. The accepted immediate scope remains design and feasibility
exploration for one focused feature, followed by a concrete plan. DEC-REX-003
subsequently settles the naming policy; implementation is described in
[the task-name discussion](../discussions/terminal-task-names.md).

## DEC-REX-003 — Automatic task names with configurable manual-name lifetime

Accepted October 30 through [design questions 5–8](../journal/2026-10-30-rex-prioritization-feedback.md#accepted-naming-policy).
Use available conversation titles automatically with process/shell fallback,
while manual names override automatic names. Show the provider icon beside the
task name. The user selected keeping a manual name until renamed/reset and added
“this could go both ways. make it a setting.” Therefore provide a setting for
keeping the manual name versus resetting it when a new agent conversation is
identified. The plan uses Keep as the default, following the selected choice.

The October 30 [screenshot follow-up](../journal/2026-10-30-rex-prioritization-feedback.md#project-suffix-follow-up)
requests removing the project-name suffix from automatic terminal labels.

Task names in desktop notifications are deferred. The
[implementation plan](../discussions/terminal-task-names.md) supplies the settings
placement, metadata handling, persistence and verification details. This accepts
feature behavior for planning; it does not authorize implementation or claim
that any provider's automatic naming/reset behavior has been tested.

## DEC-REX-004 — Implement terminal task names

Accepted October 30 in the user's direct instruction: “proceed with implementation,
use work-until-grill.” This advances the completed exploration into implementation
of DEC-REX-003, with verification and records. The earlier interview's planning-only
scope is historical; notification naming/testing and the other Rex proposals remain
deferred. See the [implementation run](../verification/runs/2026-10-30-terminal-task-names.md).
No commit, release or installed-app replacement is requested.

## DEC-RELEASE-032-001 — Focused verification is sufficient for 0.3.2

Accepted October 19 in the [release-readiness answer](../journal/2026-10-19-grill-b6498bc2-33e5-46be-9664-08769eb2c378.md):
accept the completed focused verification for 0.3.2 and defer the remaining
desktop matrix. Candidate `4afea26f` passes local tests, macOS/Ubuntu/Arch CI,
all shipping package builds and downloaded-package inspection. The
[preparation record](../development/release-0.3.2.md#accepted-readiness-scope)
retains visual/layout, physical input, accessibility, full plugin journeys and
installed Linux desktop coverage as unverified follow-up.

Notification testing and AUR publication remain explicitly deferred. No
consequential readiness question remains for this candidate. This is a
release-specific scope decision, not a permanent change to the acceptance matrix
or a publication action. Historical outcomes, including the candidate run's
aggregate **Untested**, remain unchanged.

## DEC-ATTENTION-006 — Space context and supported-log response previews

Accepted October 19 in the [notification-context answer](../journal/2026-10-19-grill-fb098581-9bb9-448d-87d2-391b21a21ae7.md).
Include the owning space and agent in the notification title. Use a shortened
first-line response preview when a supported agent log provides a current reply;
otherwise retain “Agent may need your attention.” The user selected supported
logs with a generic fallback, rather than terminal-text guessing or deferring
previews. This changes presentation, not DEC-ATTENTION-005's detection, unread,
focus or notification-click policy. The [feature record](../features/panes.md#notification-content)
defines the implemented limits and selected verification.

## DEC-RELEASE-031-002 — Focused verification is sufficient for 0.3.1

Accepted October 16 in the [release-readiness answer](../journal/2026-10-16-grill-37e45122-ab5c-4350-82ff-7a6c037454a4.md):
use the completed focused verification and defer the remaining matrix for 0.3.1.
The observed macOS menu and Linux confirmation defects are corrected; local
tests, platform CI, package builds and the selected desktop recheck pass.
The [release preparation page](../development/release-0.3.1.md#remaining-release-judgment)
lists the retained gaps, including hidden-state monitoring, physical input/GPU
and accessibility, broader desktop/visual journeys and plugin variants.

Those gaps remain unverified follow-up rather than 0.3.1 readiness blockers.
Historical Failed/Untested outcomes remain intact. This is a release-specific
scope decision, not a permanent change to the acceptance matrix or a publication
action. The AUR and manual-click deferrals in DEC-RELEASE-031-001 still apply.

## DEC-RELEASE-031-001 — Release version and explicit deferrals

Accepted October 16 in the release-preparation conversation: use **0.3.1**,
preserve the user's hand-tuned fade implementation and align its tests/docs,
defer AUR publication while account registration is unavailable, and omit the
requested manual notification-click check. These are scoped deferrals, not passes
or a blanket waiver of the [release acceptance matrix](../verification/acceptance.md).
The later DEC-RELEASE-031-002 separately resolves the remaining 0.3.1 coverage.
Automatic AUR publication is opt-in; the [runbook](../development/releasing.md#aur-publishing)
records the re-enablement path. [Release preparation](../development/release-0.3.1.md)
tracks evidence and remaining scope.

## DEC-ATTENTION-005 — Reuse existing detection with best-effort alerts

Accepted October 15 in [questions 15–17](../journal/2026-10-15-grill-1cbbb041-3fb2-41c0-97c7-434cb1dbeff1.md).
Use Herdr's existing agent detection for attention. Accept occasional false or
missed alerts, including cancellation or unfamiliar screens appearing inactive.
The dot and notification mean the agent may need attention, not confirmed
success or failure. This prioritizes broad upstream-supported coverage and low
Chartr maintenance over new provider-specific outcome adapters. Explicit terminal
signals and a combined detector are not selected for this iteration.

Send at most one desktop notification per terminal until the user visits it.
Exact terminal focus in an active foreground window clears unread attention and
allows a later event to notify again. Existing group aggregation, retention during
resumed work, restored unread dots without catch-up notifications, and app-lifetime
scope remain unchanged. Codex alone is sufficient for the first real-agent
validation; this is not a Codex-only support policy.

This supersedes the strict uncertainty and confirmed-outcome requirements in
DEC-ATTENTION-001/003. DEC-ATTENTION-004 records the earlier implementation choice;
its rich backend contract is no longer a prerequisite for the selected design.
The interview itself made no product changes. Implementation authorized on October 15
replaces the rich patch with terminal-identity preservation only. The
[revised plan](../discussions/agent-attention.md#accepted-simplification-october-15)
separates the new target from the October 14 implementation and verification.

## DEC-ATTENTION-004 — Carry the required backend changes as a tracked patch

Accepted October 13 in [concise question 14](../journal/2026-10-13-grill-1a7ca491-2a9a-41df-bf41-6294ae40628a.md).
Keep a Herdr patch in Chartr, applied to the pinned source with a distinct
reproducible build identity. Chartr owns patch maintenance when upgrading Herdr;
agent-specific interpretation remains in the backend. The continuing
work-until-grill instruction covers implementation and verification. No upstream
submission or separate fork is requested. The [plugin feasibility check](../verification/runs/2026-10-13-agent-attention-plugin.md)
preceded this choice; it found that current hooks cannot recover lost outcome
information.

October 14 implementation checkpoint: the tracked patch and Chartr policy are
implemented locally. [Verification](../verification/runs/2026-10-14-agent-attention.md)
separates passing checks from remaining native/provider gaps. DEC-ATTENTION-004
supersedes the earlier no-implementation constraints quoted in decisions 001–003;
those paragraphs retain their original planning context.

These migration decisions were accepted in the
[first interview](../journal/2026-10-05-grill-ab31b76a-3123-4152-9f0b-1c822da69430.md)
and clarified in the
[second interview](../journal/2026-10-05-grill-fa77900d-80f2-4c30-95c5-cf40e9aa1ee9.md).
The original product ADRs are linked below. Each migration decision below is dated
2026-10-05 and remains accepted unless explicitly superseded.

| ID | Decision | Context and basis |
| --- | --- | --- |
| DEC-JSTACK-001 | Migrate development documentation and process; retain Wayfinder product behavior. | Q1 explicitly limits the scope. Wayfinder's map/ticket format is a separate product contract. |
| DEC-JSTACK-002 | Keep public guides outside `docs/wiki/`; use that wiki for development records. | Q2 explicitly preserves public guides under `docs/`, outside the wiki subtree. This maintains separate user and contributor audiences. |
| DEC-JSTACK-003 | Use current implementation as the migration baseline when older documentation disagrees. | Q3 selects implementation over previous specifications for this migration. Record the inspected revision and dirty state; source inspection does not establish successful execution. This is a migration baseline, not an ongoing rule to redefine regressions as correct. |
| DEC-JSTACK-004 | Preserve a full browsable historical archive in the wiki. | Q4 selects the full archive. Preserve dates, original claims, evidence, and supersession context while adapting links and presentation. |
| DEC-JSTACK-005 | Make needed reader improvements in shared jstack, with lean and simple implementations. | Q5 selects shared extensions and asks what changes are needed. Q8 and Q9 settle the extension scope in DEC-JSTACK-007 and DEC-JSTACK-008. Avoid a chartr-specific reader fork. |
| DEC-JSTACK-006 | Use an advisory development workflow with optional records. | Q7 explicitly selects advisory use. No mandatory interview/report sequence or new enforcement gate is established by this choice. |
| DEC-JSTACK-007 | Add simple inline local images to the shared jstack reader. | Q8 accepts standard Markdown image rendering for local PNG/JPEG/GIF/WebP attachments, reusing validation and hashing with responsive sizing and necessary sanitizer/content-policy changes. Keep files alongside the HTML and verify rendering. |
| DEC-JSTACK-008 | Use GitHub URLs for references outside the wiki. | Q9 rejects the need for a local repository-link extension. Pin historical references to recorded revisions where available. This preserves the lean reader while keeping public guides separate. |
| DEC-JSTACK-009 | Validate documentation and changed-reader behavior for migration completion; fresh chartr execution is not required. | Q10 resolves the qualification in Q6. Check content fidelity, links, preserved history/evidence, wiki build/freshness, and tests plus browser rendering for reader changes. Imported verification recipes are unexecuted until actually run. |

The [question register](../discussions/open-questions.md) preserves the resolved
questions. No consequential migration-scope question remains open after round 2.

## Product architecture decisions

2026-10-10 accepted requirement: scrolling, view switching and terminal input
must remain responsive on battery with Low Power Mode enabled. The user requested
investigation and a lean plan, with no implementation yet. The
[rendering investigation](../journal/2026-10-10-low-power-frame-investigation.md)
records the evidence. After the user rejected a conditional trial plan, the
[concrete design](../journal/2026-10-10-low-power-fix-design.md) selects
supplied-drawable frame delivery as the recommended implementation. This remains
an agent proposal, not user approval to implement or a verified product fix.

2026-10-11: The user authorized implementation with `work-until-grill`. The
[implementation record](../journal/2026-10-11-low-power-implementation.md) carries
that scope forward: supplied-drawable delivery on modern macOS, the pinned
renderer extraction, preserved legacy/transaction paths, and isolated native
verification. This supersedes the earlier restriction on implementing the fix;
verification results remain separate from the accepted design.

Original files and numeric identities are preserved in the [ADR register](adr/index.md).
Their original prose supplies context and rationale; no missing original decision
dates have been invented during import.

| Identity | Decision | Current connection |
| --- | --- | --- |
| ADR-0001 | [Private Herdr](adr/0001-a-private-herdr.md) | [Session lifecycle](../features/session-lifecycle.md) |
| ADR-0002 | [Zed layer](adr/0002-the-zed-layer.md) | [Terminal](../features/terminals.md), [Settings](../features/settings.md) |
| ADR-0003 | [Plugin runtimes](adr/0003-two-plugin-tiers.md) | Hosted tier superseded by ADR-0007; retain rejection rationale |
| ADR-0004 | [Complete terminal stack](adr/0004-the-zed-terminal-stack.md) | [Terminal interaction](../features/terminals.md) |
| ADR-0005 | [Multi-workspace ownership](adr/0005-spaces-follow-zed-multi-workspace.md) | [Spaces](../features/spaces.md), [Panes](../features/panes.md) |
| ADR-0006 | [Native services and web Wayfinder](adr/0006-native-plugin-services-and-wayfinder.md) | [Plugin services](../features/plugin-runtime.md), [Wayfinder](../features/wayfinder.md) |
| ADR-0007 | [Prebuilt native surfaces](adr/0007-prebuilt-native-surfaces.md) | [Plugin packages](../features/plugin-packages.md), [Runtime](../features/plugin-runtime.md) |

## DEC-LIFECYCLE-001 — Isolated native reopen verification

Accepted 2026-10-12. The user requested the last-window reopen fix with
work-until-grill, then explicitly approved a native AppKit test driver when the
regular desktop tool could not start. Approval covers close/reopen, minimize,
disposable terminal input and quit in the prepared isolated test instances;
it does not authorize controlling the installed app or changing Accessibility
permissions. [Submitted answer](../journal/2026-10-12-grill-e4ec3c4a-3dec-4cfc-a0f5-5c9f91924f20.md),
[feature](../features/session-lifecycle.md),
[implementation and executed verification](../verification/runs/2026-10-12-window-reopen.md).

## DEC-ATTENTION-001 — Agent attention behavior

Accepted 2026-10-12 through the
[grill-form answers](../journal/2026-10-12-grill-b4cd88bb-d438-4828-a304-f5b77f4dae14.md).
Unseen completion and input/approval requests receive an accent dot on their
terminal tab and enclosing group. Focus in an active window acknowledges that
terminal; visibility in a split does not. Desktop notifications apply when that
terminal is not focused or Chartr is inactive. Scope is agents only while Chartr
is running, and uncertain state must not produce guessed alerts.

The user requested comparative research before selecting the detection
architecture. The [discussion and research](../discussions/agent-attention.md)
preserves the exact requirements, evidence and proposed reuse of Herdr. This is
planning authorization only; no implementation or architecture proposal has
been approved by these answers.

## DEC-ATTENTION-002 — Reuse Herdr for attention detection

Accepted 2026-10-13 when the follow-up form was collected. The user selected
reuse of Herdr with conservative attention events, then asked for the concrete
work required for dots and notifications. The
[answer](../journal/2026-10-12-grill-ff575bf5-cacf-4ab8-868e-46d9ca93c63e.md)
comes from the round created October 12.

Herdr remains the common detection boundary; Chartr owns one provider-independent
attention/focus/presentation policy. This resolves the architecture direction
left open by the first interview. The [proposed implementation](../discussions/agent-attention.md#proposed-implementation)
maps the remaining work to the backend API, attention model, shared chrome and
existing GPUI notification service. The user has not authorized implementation
or accepted every proposed contract/lifecycle detail.

## DEC-ATTENTION-003 — Retain unseen attention and include failures

Accepted 2026-10-13 through the
[grill-form-concise answers](../journal/2026-10-12-grill-5f286d71-92ef-499d-a8fd-4591f968fade.md).
Keep a terminal's unread dot until focus, even when its agent resumes working.
Restore dots for restored sessions after relaunch, without catch-up desktop
notifications. Include confirmed agent failures in the same dot/notification
policy, labeled as failures; normal exits, user cancellations and uncertain
detection do not qualify.

These choices preserve unseen attention independently of activity and add
dependable failure alerts to DEC-ATTENTION-001's event scope. They resolve the
remaining user-level retention/failure questions. The
[implementation design](../discussions/agent-attention.md#proposed-implementation)
still requires backend contract and signal verification; product implementation
remains outside the authorized discussion scope.

The user subsequently authorized investigation and validation with
`work-until-grill`, retaining the no-implementation constraint. The
[contract investigation](../verification/runs/2026-10-13-agent-attention-contract.md)
records executed gaps and a revised technical proposal. It does not change these
accepted product decisions or establish live-provider coverage.
