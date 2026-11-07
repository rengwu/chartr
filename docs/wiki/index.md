---
title: Chartr development wiki
updated: 2026-11-07
---
# Chartr development wiki

Development knowledge for the current chartr working tree. Use the
[public documentation](https://github.com/rengwu/chartr/blob/main/docs/README.md)
for user and plugin-author guides. Markdown is authoritative; `index.html` is the
locally generated reader.

## Start here

- [Project status, November 3](state/2026-11-03.md): current feature states,
  automated checks, and committed/local changes since v0.3.2. The
  [release comparison](evidence/2026-11-03-project-status-checks/release-delta.json),
  [check results](evidence/2026-11-03-project-status-checks/results.json),
  [source identity](evidence/2026-11-03-project-status-checks/identity.json), and
  [feature evidence](evidence/2026-11-03-project-status-checks/feature-evidence.json)
  retain the inspected snapshot and coverage.
- [Development workflow](development/workflow.md): choose the jstack skills that
  fit the task; records and interviews are advisory.
- [Feature map](features/index.md): 15 coherent behavior records, with the complete
  [97-story crosswalk](features/story-crosswalk.md).
- [Code map](development/code-map.md): implementation owners and boundaries.
- [Decisions](decisions/index.md), [architecture rationale](decisions/adr/index.md),
  and [open questions](discussions/open-questions.md).

## Verification and releases

[0.3.3 preparation](development/release-0.3.3.md) uses the user's requested
simple checks without desktop takeover and explicitly skips old-to-new upgrade
testing. Its [candidate run](verification/runs/2026-11-07-release-0.3.3.md)
records passing local checks and the remaining native coverage limits.

[Next-release assessment, November 7](discussions/open-questions.md#next-release-assessment-november-7)
proposes priorities for upgrade preservation, native naming/input checks,
notifications, final candidate gates and focused polish. These are recommendations,
not accepted scope changes or newly executed product results.

[November 7 mouse recovery](verification/runs/2026-11-07-mouse-recovery.md)
records stale mouse reporting after terminated TUIs, the live imparo recovery,
and bounded backend regression checks.

[0.3.2 preparation](development/release-0.3.2.md) covers the Settings-only view
switcher, pane/tab styling and notification context changes. Its
[candidate checks](verification/runs/2026-10-19-release-0.3.2.md) record passing
local/platform gates and package inspection. The user
[accepted focused verification for 0.3.2](decisions/index.md#dec-release-032-001-focused-verification-is-sufficient-for-0-3-2)
and deferred the remaining desktop matrix. Notification testing and AUR publication
remain deferred. [Chartr v0.3.2](https://github.com/rengwu/chartr/releases/tag/v0.3.2)
is published as the latest stable release. The [publication run](verification/runs/2026-10-21-release-publication.md)
records passing tag builds, package/checksum inspection, public downloads and the
skipped AUR publication.

- [Shared setup](verification/setup.md): exact build identity, instance isolation,
  fixtures, available tests and cleanup.
- [Release acceptance](verification/acceptance.md) and
  [packaging runbook](development/releasing.md).

The [0.3.1 candidate follow-up](verification/runs/2026-10-16-release-candidate.md)
records passing package inspection and selected native attention checks.
The [desktop follow-up](verification/runs/2026-10-16-release-desktop.md) verifies
macOS Quit policies and a Linux terminal/Clock journey, and preserves the Linux
confirmation-clipping failure. The [recheck](verification/runs/2026-10-16-release-desktop-recheck.md)
tracks its correction and the latest automated gates.
[Release preparation](development/release-0.3.1.md) records the accepted readiness
scope. The [submitted answer](journal/2026-10-16-grill-37e45122-ab5c-4350-82ff-7a6c037454a4.md)
accepts focused verification and defers the remaining matrix for 0.3.1;
the [checkpoint](journal/2026-10-16-release-preparation.md) records that outcome.
AUR publication and the manual notification-click check remain deferred.
[Chartr v0.3.1](https://github.com/rengwu/chartr/releases/tag/v0.3.1) was published
on October 16. Its [publication run](verification/runs/2026-10-16-release-publication.md)
records passing tag builds, asset/checksum inspection, public downloads and the
skipped AUR publication.

The [initial October 16 assessment](verification/runs/2026-10-16-release-readiness.md)
remains historical **Failed** evidence. The
[fade expectation correction](verification/runs/2026-10-16-fade-expectations.md)
preserves the user's manual rendering values and passes all 428 local workspace tests.

The migration did not build or exercise Chartr. Later execution is recorded in
dated runs, including the [macOS frame-delivery recheck](verification/runs/2026-10-11-metal-frame-delivery-recheck.md).
Unexecuted recipes remain unvalidated, and historical passes apply only to their
recorded snapshots.

## History and migration

- [Rex ideas for Chartr](discussions/rex-beta-adoption.md): October 29 installed-beta
  investigation ranks 15 proposals, led by task names, workspace search, activity
  triage, a local control API and reusable workspace recipes. The
  [technical appendix](discussions/rex-beta-internals.md) records binary analysis,
  native inspection, isolated experiments and coverage limits. Terminal task
  names have an [accepted policy and implementation plan](discussions/terminal-task-names.md);
  implementation was subsequently authorized and is recorded in the
  [implementation run](verification/runs/2026-10-30-terminal-task-names.md).
  The [first prioritization response](journal/2026-10-29-grill-8f14b5c8-c08d-4baf-84bc-599282c28304.md)
  requests individual proposal choices; the
  [second response](journal/2026-10-30-rex-prioritization-feedback.md) requests
  multiple selections. The [later answer](journal/2026-10-30-rex-prioritization-feedback.md#accepted-scope-and-next-stage)
  accepts one focused feature and exploration before implementation. The
  [final shortlist](journal/2026-10-30-rex-prioritization-feedback.md#selected-target)
  selects terminal task names. [Design answers](journal/2026-10-30-rex-prioritization-feedback.md#accepted-naming-policy)
  accept automatic titles, configurable manual-name lifetime and provider icons;
  task names in notifications are deferred.
- [Harp ideas for Chartr](discussions/harp-adoption.md): October 21 source comparison
  recommends task names, space attention summaries, optional set-aside state and
  usage status. These are proposals; no implementation or adoption is claimed.
- [Agent attention and notifications](discussions/agent-attention.md): the October 15
  [accepted simplification](discussions/agent-attention.md#accepted-simplification-october-15)
  is implemented using existing Herdr detection and one notification until the
  exact terminal is visited. The [Codex recheck](verification/runs/2026-10-15-lean-attention.md)
  records passing automated tests, real completion/approval/cancellation states,
  dots/focus, persistence, live handoff and minimized/closed-window monitoring.
  Native click navigation, direct banner inspection and broader platform/layout
  coverage remain unverified. [Setup attempts](verification/runs/2026-10-15-lean-attention-attempts.md)
  and the [October 14 strict implementation](verification/runs/2026-10-14-agent-attention.md)
  remain historical evidence.
  The October 19 [notification-context decision](decisions/index.md#dec-attention-006-space-context-and-supported-log-response-previews)
  adds space/agent titles and supported-log reply previews with generic fallback;
  native appearance for this change remains unverified.
- [Last-window reopen implementation](verification/runs/2026-10-12-window-reopen.md):
  window restoration and shutdown ownership corrected; automated checks and
  native reopen/session/quit checks pass. The unchanged-build failure is preserved.

- [Sidebar CI fixture repair](verification/runs/2026-10-12-sidebar-ci.md):
  stale empty-space expectation corrected while preserving populated-space
  coverage; local checks and the full macOS, Ubuntu and Arch/AUR CI matrix passed.
- [Supplied-drawable implementation](journal/2026-10-11-low-power-implementation.md):
  implemented Low Power Mode fix, with three active captures and selected native
  and app rendering checks passed; aggregate lifecycle failure, latency percentile
  and compatibility limits recorded.
- [Concrete Low Power Mode fix design](journal/2026-10-10-low-power-fix-design.md):
  supplied-drawable frame delivery removes acquisition from the observed blocking
  path; exact code boundaries, lifetime rules and compatibility scope are defined.
- [Low Power Mode evidence and resolution plan](journal/2026-10-10-low-power-frame-investigation.md):
  standalone Metal timing reproduces drawable backpressure; records a bounded
  pacing trial, fallback and acceptance criteria. No product fix applied.
- [Running build lag investigation](journal/2026-10-10-running-build-lag.md):
  battery/AC reproduction localizes the visible stall to main-thread Metal
  drawable waits; frame scheduling needs a fix. Backend overhead is recorded
  separately.
- [Historical archive](archive/index.md): complete research, original rewrite
  specification, inactive Companion protocol, screenshots and receipts.
- [Agreed migration scope](discussions/jstack-migration.md).
- [Implementation and validation journal](journal/2026-10-05-jstack-migration-implementation.md).
- [Round 1 interpretation](journal/2026-10-05-jstack-migration-round-1.md) and
  [round 2 interpretation](journal/2026-10-05-jstack-migration-round-2.md).
- [Round 1 answers](journal/2026-10-05-grill-ab31b76a-3123-4152-9f0b-1c822da69430.md) and
  [round 2 answers](journal/2026-10-05-grill-fa77900d-80f2-4c30-95c5-cf40e9aa1ee9.md).
