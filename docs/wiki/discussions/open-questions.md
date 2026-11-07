---
title: Open questions
updated: 2026-11-07
status: Release 0.3.3 published under accepted verification scope
---
# Open questions

There are no remaining consequential scope questions from the migration interview.
Reopen this register if implementation uncovers a material ambiguity.

## Next-release assessment — November 7

**Subsequent decision:** the user skipped the upgrade check and requested simple
verification without desktop takeover, followed by preparation of **0.3.3**.
[DEC-RELEASE-033-001](../decisions/index.md#dec-release-033-001-prepare-0-3-3-with-simple-verification)
records this scope; [preparation](../development/release-0.3.3.md) tracks execution.
After preparation completed, the user [authorized publication](../decisions/index.md#dec-release-033-002-publish-the-prepared-0-3-3-release).
The [publication run](../verification/runs/2026-11-07-release-publication.md)
records the completed release; the accepted coverage limits remain unchanged.
The recommendations below remain the historical assessment, not the final scope.

The user requested an opinion on remaining issues and polish. The priorities below
are **recommendations, not accepted release decisions or new verification results**.
They are based on source review and the existing dated runs at HEAD `36929806`
with the local naming, frame-inset and mouse-recovery changes. No product tests
were executed for this assessment.

Recommended scope: finish terminal task names, the workspace styling changes and
bounded mouse recovery. Keep the other [Rex proposals](rex-beta-adoption.md),
notification task names and AUR publication outside this release's scope. The
accepted naming policy does not need another design interview.

| Priority | Remaining work | Basis and completion target |
| --- | --- | --- |
| Release-critical verification | Upgrade from the actual v0.3.2 backend with live sessions | The client requests live handoff when the backend version differs. The new mouse fixture hands the new sidecar to another copy of itself; that does not establish old-to-new compatibility. In an isolated profile, open the candidate over a v0.3.2 workspace and verify running jobs, terminal identity, layout, attention and usable input survive. Then rename and relaunch to check new naming persistence. |
| Release-critical verification | Finish native task-name behavior | The [naming run](../verification/runs/2026-10-30-terminal-task-names.md) leaves native dialogs, focus and reopen untested. Exercise all rename/reset entry points, cancel/Escape, long and Unicode names, split/group moves, target closure during editing, embedded-surface focus and frontend restart. Use real agent metadata to check title updates, missing identity and both lifetime policies. |
| Release-critical verification and scope clarity | Verify mouse recovery through the native candidate | The [mouse run](../verification/runs/2026-11-07-mouse-recovery.md) passes backend regressions; its installed-app recovery was manual, and a fresh native frontend using the patch was not exercised. Check selection, wheel/trackpad and subsequent TUI input after a killed job. The fixture runs `stty sane`; automatic recovery does not repair extended keyboard modes or termios. Describe this as mouse recovery. If the native shell remains unusable in the selected scenario, establish a usable recovery path before advertising it as resolved. |
| Outstanding next-release check | Inspect real notifications and click routing | Testing was [deferred until the next release](agent-attention.md#local-bundle-attempt-and-user-deferral). Use an explicitly isolated registered bundle to inspect space/agent titles, reply preview/fallback, one-notice behavior and exact-terminal navigation, including an origin that has closed. Task names in notifications remain deferred. |
| Release gate | Validate one final candidate across supported platforms | Commit the intended source, new naming module and records when release preparation is authorized. Run the existing [acceptance](../verification/acceptance.md) and [packaging](../development/releasing.md) gates on that candidate, including explicitly invoked real-sidecar tests and exact sidecar checks in the packages. Earlier committed-main CI excludes local changes; the November 3 full suite predates the mouse patch. Prior release-specific desktop exceptions do not automatically apply. |
| Focused polish | Inspect the frame, headings and long labels together | The [frame run](../verification/runs/2026-10-30-tabs-frame-inset.md) establishes selected macOS geometry. Check narrow windows, light/dark themes, enlarged UI text, grouped panes, close controls, tooltips and native/web plugin surfaces for clipping, contrast and focus. This is a proposed inspection, not a claim that these defects exist. |
| Documentation cleanup | Correct current descriptions and release claims | The [November 3 assessment](../state/2026-11-03.md) identifies stale default-font and Close All Sessions wording. The terminal feature page also retains an identity-only backend-patch description in its dated attention section. Clarify its historical scope now that mouse recovery extends the patch; preserve old evidence. Document best-effort title/reset coverage and bounded recovery accurately. |

Two limitations deserve explicit release wording: consecutive conversations with
the same title can fall back indefinitely under the current stale-title policy,
and reset-on-new-conversation depends on reported identity. Exercise the real
provider path before deciding whether either needs a code change; keep manual
names and the Keep default as accepted.

The original termination/memory-pressure cause remains undiagnosed in the mouse
incident. A representative multi-session memory/CPU soak would be useful follow-up,
but this assessment does not establish a Chartr memory regression or claim that
mouse recovery fixes resource exhaustion.

The next useful step is an isolated candidate run centered on upgrade, naming and
native input, with notification checks included in release preparation. Any
remaining acceptance exception needs a decision for this release; none is granted
by this assessment.

## Rex adoption proposals

The October 29 [installed-beta investigation](rex-beta-adoption.md) recommends
task names, workspace search and activity triage first, followed by a local
control interface and reusable workspace recipes. Its
[technical evidence](rex-beta-internals.md) distinguishes recovered binary
information, advertised contracts, native observations and isolated experiments.
The research itself adopted no feature. The later interview selected terminal
task names for exploration, then explicitly authorized implementation in
[DEC-REX-004](../decisions/index.md#dec-rex-004-implement-terminal-task-names).

The [first interview response](../journal/2026-10-29-grill-8f14b5c8-c08d-4baf-84bc-599282c28304.md)
requests all 15 proposals as individual choices rather than grouped directions.
The [second response](../journal/2026-10-30-rex-prioritization-feedback.md)
requests multiple selections. The [later submitted answer](../journal/2026-10-30-rex-prioritization-feedback.md#accepted-scope-and-next-stage)
accepts one focused feature for the first deliverable and design/feasibility
exploration followed by a concrete plan. Its proposal question contains no
submitted selection state. A [subsequent shortlist](../journal/2026-10-30-rex-prioritization-feedback.md#selected-target)
selects **#1, terminal task names**, resolving the target. The
[design answers](../journal/2026-10-30-rex-prioritization-feedback.md#accepted-naming-policy)
settle automatic conversation titles with process/shell fallback, provider icon
plus task name, a setting for manual-name lifetime, and notification naming
deferred. The [implementation plan](terminal-task-names.md) uses Keep as the
setting default and existing native identity for optional reset. No further
preference question is needed for this plan. Runtime title freshness and
conversation-identity coverage remain verification limits, not settled test
results. The later direct instruction authorized implementation; the
[run record](../verification/runs/2026-10-30-terminal-task-names.md) tracks executed
checks and the unavailable native automation driver.

If selected, later slices need decisions about set-aside/muting semantics,
control-API authority and closed-GUI behavior, recipe scope/command review,
exact job-result provenance, and remote-work scope. Explicit program-status
support should be investigated in Herdr/upstream before changing the accepted
best-effort attention contract. Existing unread retention, private-backend,
disabled Chats and Settings-only view-mode decisions remain in force.

## Harp adoption proposals

The October 21 [source comparison](harp-adoption.md) proposes task names,
space attention summaries, set-aside state, usage status and diagnostic traces.
No adoption or implementation decision has been accepted. The recommended first
slice is task names plus space unread counts and exact-terminal navigation.
Set-aside notification/resume semantics and the source of live usage data need
resolution if those later proposals are selected. Existing attention detection,
unread retention and private-backend decisions remain in force.

## Release 0.3.2 acceptance scope

[Preparation](../development/release-0.3.2.md) has passing local automated checks,
macOS/Ubuntu/Arch CI, all shipping package builds and downloaded-package inspection.
Resolved October 19 by the [submitted answer](../journal/2026-10-19-grill-b6498bc2-33e5-46be-9664-08769eb2c378.md):
accept focused verification for 0.3.2 and defer the remaining desktop matrix.
[DEC-RELEASE-032-001](../decisions/index.md#dec-release-032-001-focused-verification-is-sufficient-for-0-3-2)
records this release-specific exception. Notification testing and AUR publication
remain explicitly deferred. The broader native desktop matrix remains unverified
follow-up, not a 0.3.2 readiness blocker. No readiness question remains. The user
requested publication on October 21; the [publication run](../verification/runs/2026-10-21-release-publication.md)
records the completed release without changing these deferrals.

## Notification preview source

Resolved October 19 by the [submitted answer](../journal/2026-10-19-grill-fb098581-9bb9-448d-87d2-391b21a21ae7.md):
use supported logs and a generic fallback, with no terminal-text guessing.
[DEC-ATTENTION-006](../decisions/index.md#dec-attention-006-space-context-and-supported-log-response-previews)
records the choice. No additional product decision is pending for this change.
On October 19 the user deferred notification testing until the next release after
reporting session disruption from the supplied shared-profile local bundle launch.
The [attempt and deferral](agent-attention.md#local-bundle-attempt-and-user-deferral)
remain inconclusive for native notification behavior, not a passing verification.

## Release 0.3.1 deferrals

The user set aside the requested manual notification-click check and deferred
AUR publication on October 16 because AUR account registration is unavailable.
These are explicit deferrals, not passing verification or evidence of an AUR
package.

The [submitted release-readiness answer](../journal/2026-10-16-grill-37e45122-ab5c-4350-82ff-7a6c037454a4.md)
also accepts the focused verification and defers the remaining matrix for 0.3.1.
[DEC-RELEASE-031-002](../decisions/index.md#dec-release-031-002-focused-verification-is-sufficient-for-0-3-1)
resolves the release-scope question. [Release preparation](../development/release-0.3.1.md#remaining-release-judgment)
retains the unverified coverage as follow-up. These gaps no longer block 0.3.1
readiness; no remaining readiness decision requires another interview.

## Performance verification questions

Implementation was authorized on October 11. These remain measurement questions,
not requests for implementation approval. The [current recheck](../verification/runs/2026-10-11-metal-frame-delivery-recheck.md)
records the correction after an activation failure and the completed real-window
verification: the selected rendering scenarios pass, with
zero acquisition samples across three active battery/Low Power Mode captures.
Its aggregate result is Failed because the exercised same-process last-window
reopen failed. Earlier failed results remain linked from it.

| ID | Question | Current evidence and next check |
| --- | --- | --- |
| Q-PERF-001 | Is a small Low Power Mode request-rate limit sufficient to restore input responsiveness? | Resolved for design selection: it remains an uncertain mitigation and is not the selected fix. Supplied drawables are now implemented and actual-app traces confirm removal of acquisition waiting. Native event-to-presentation percentiles remain unmeasured. |
| Q-PERF-002 | Does battery power alone reproduce the stall with Low Power Mode off? | The confirmed comparison changed both power source and policy. Add the missing power condition to controlled verification. |

Sources: [Low Power Mode investigation](../journal/2026-10-10-low-power-frame-investigation.md)
and [concrete fix design](../journal/2026-10-10-low-power-fix-design.md).

## Lifecycle follow-up outside the rendering patch

The October 11 recheck found that opening the app after its last window closes
does not recreate a window while the process remains alive. The unchanged app
entry point does not register `on_reopen`; a fresh process launch works. App Hide
was not established either: Cmd+H had no visible effect and the application menu
was empty. These observations are retained as separate follow-up work, not fixed
or counted as successful lifecycle behavior by the performance change.

The user requested the reopen fix on October 12. The
[implementation run](../verification/runs/2026-10-12-window-reopen.md) records a
registered reopen handler, shared startup/restoration path and current-window
quit handling. The user approved an isolated native AppKit test route after desktop tool
startup failed. Automated checks and six native reopen cycles passed, including
terminal identity/input, saved bounds/layout, existing/minimized-window behavior
and both quit policies. The reopen issue is resolved for this tested build;
App Hide remains a separate unverified follow-up. The
[October 16 desktop run](../verification/runs/2026-10-16-release-desktop.md) adds
the missing application menu and shortcuts, and verifies both Cmd+Q policies;
actual hidden-state monitoring is still not claimed.

## Agent attention design

The [interview and comparative research](agent-attention.md) settles the visible
behavior. On October 13 the user accepted reuse of Herdr and requested a concrete
work breakdown. The [implementation design](agent-attention.md#proposed-implementation)
was a proposal during the discussion. After the readiness exchange, the October
13 work-until-grill instruction authorizes implementation. Preparation exposed
the backend distribution choice Q-ATTENTION-004 before product edits.
The concise follow-up resolves retention and confirmed-failure scope. The
[contract investigation](../verification/runs/2026-10-13-agent-attention-contract.md)
then exercised the pinned backend and integration assets. It establishes concrete
contract gaps and revises the implementation design; no additional product choice
was required. The [October 14 recheck](../verification/runs/2026-10-14-agent-attention.md) records
the implementation and selected passing checks. Remaining gaps concern native
banners/clicks, layout/lifecycle variants, Linux and provider coverage; no new
product preference is needed to record these limits.

| ID | Question | Current evidence and next step |
| --- | --- | --- |
| Q-ATTENTION-001 | Can Chartr use one approach across agents without maintaining its own detector for each CLI? | Resolved for direction on October 13: reuse Herdr with conservative events, DEC-ATTENTION-002. This shares detection maintenance; it does not establish reliable automatic detection for every arbitrary CLI. |
| Q-ATTENTION-002 | Does the current backend expose sufficient event identity and evidence for conservative alerts? | Resolved for investigation: no. Executed probes show presentation/replay ambiguity, fallback and cancellation becoming done, loss of failure/cancellation distinctions, and no failure state. The revised design requires typed integration observations plus a qualified global snapshot/attention stream; adding output fields alone is insufficient. Real producer/version eligibility remains an implementation verification gate. |
| Q-ATTENTION-003 | How should attention survive state changes and reconnects? | Policy resolved in DEC-ATTENTION-003. The investigation proposes a separate versioned `app_state` attention value, atomic snapshot/stream startup, explicit gap recovery, and persistence of terminal identity/attention across live handoff. The original handoff changed terminal ID. The patched backend now preserves identity in the executed real-Pi handoff; native dot restoration and attention persistence tests pass. See the October 14 recheck for exact coverage. |
| Q-ATTENTION-004 | How should Chartr carry the required unreleased Herdr changes? | Resolved in DEC-ATTENTION-004: the user selected a tracked Herdr patch in Chartr in the concise question 14. The [executed plugin check](../verification/runs/2026-10-13-agent-attention-plugin.md) established the limits of current hooks before that choice. Local backend changes/builds are authorized; upstream submissions and a separate fork are not requested. |

## Lean attention reconsideration

Resolved for design on October 15 through
[questions 15–17](../journal/2026-10-15-grill-1cbbb041-3fb2-41c0-97c7-434cb1dbeff1.md):
use existing Herdr detection with best-effort alerts, notify once until the exact
terminal is visited, and validate Codex first. The
[accepted simplification](agent-attention.md#accepted-simplification-october-15)
supersedes the earlier strict-outcome requirements. The
[October 14 comparison](agent-attention.md#lean-reconsideration-october-14-proposal)
remains historical; explicit signals are not selected for this iteration.

The simplification is implemented. The [October 15 recheck](../verification/runs/2026-10-15-lean-attention.md)
records a one-second namespace-wide status poll, preserved foreground-focus and
unread retention, removal of the strict protocol/adapters, and real Codex checks.
Completion and approval requests produced unread attention; cancellation also
produced attention, within the accepted best-effort tradeoff. The only backend
patch preserves terminal identity across handoff.

No further product preference is unresolved. Native notification response routing,
direct inspection of banners, split/group focus, app Hide, graceful Quit and Linux
remain verification gaps. macOS accepted notifications and logged presentation;
the desktop tool could not expose the actual alert for a click. These are factual
coverage limits, not reasons to reopen the accepted design or invent a grill.
The October 14 verification applies only to its superseded implementation.

## Resolved questions

| ID | Question | Resolution on 2026-10-05 |
| --- | --- | --- |
| Q-JSTACK-008 | Should historical screenshots render inline in the reader? | Yes: simple local Markdown image support in shared jstack. DEC-JSTACK-007. |
| Q-JSTACK-009 | Should references outside the wiki use repository URLs or retain local checkout links? | GitHub URLs; no reader extension for repository links. DEC-JSTACK-008. |
| Q-JSTACK-010 | Does migration completion require fresh chartr execution? | No: documentation and changed-reader validation only. Q10 resolves the earlier selected option and qualification. DEC-JSTACK-009. |

Sources: [first interview](../journal/2026-10-05-grill-ab31b76a-3123-4152-9f0b-1c822da69430.md),
[second interview](../journal/2026-10-05-grill-fa77900d-80f2-4c30-95c5-cf40e9aa1ee9.md),
and [accepted decisions](../decisions/index.md).
