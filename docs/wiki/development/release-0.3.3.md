---
title: Release 0.3.3 preparation
section: Development
updated: 2026-11-07
status: Prepared under accepted verification scope
---
# Release 0.3.3 preparation

The user requested **0.3.3**, explicitly skipped the proposed v0.3.2 upgrade
check, and requested simple verification without desktop takeover. Preparation
uses branch `release/0.3.3`; the workspace and both Cargo lockfiles agree.
Local checks, platform CI and candidate package inspection are complete. The
[scope decision](../decisions/index.md#dec-release-033-001-prepare-0-3-3-with-simple-verification)
and [fixture recheck](../verification/runs/2026-11-07-release-0.3.3-recheck.md) distinguish
executed checks from remaining native coverage. Publication is not part of this
preparation request; v0.3.2 remains the published stable release.

## Scope since 0.3.2

- Automatic conversation titles with agent/process/shell fallback, exact
  project-suffix cleanup, manual rename/reset from terminal menus and the
  command palette, optional rename shortcut and full-label tooltips.
- Persistent manual names keyed by terminal identity, with Keep as the default
  and optional reset on a different known conversation. Pane remapping preserves
  item ownership, labels and session-bound plugin associations.
- Rounded workspace frame, smaller muted space headings and symmetric 4 px
  left/right gaps in Tabs mode. Spaces remains flush with its sidebar.
- Bounded mouse-mode cleanup when the backend observes a foreground job return
  control to its shell, including detached and handed-off runtimes. The exact
  bundled Herdr patch identity is updated.
- Public naming/recovery documentation and corrected current feature descriptions.

## Verification scope

Local checks pass: 451 workspace tests, four explicitly enabled real-session
tests, formatting, the native plugin example, 15 applicable Python tests, ten
JavaScript tests and the normal macOS build. The test runner caps local Cargo
build concurrency at two jobs. See the run for exact identity and logs.

No desktop app, system notification banner or live agent was launched. Rename
and notification behavior were checked through existing automated UI/model tests;
the real PTY suite verified mouse routing and recovery. Existing October 30 frame
captures were reviewed with matching source hashes; this is historical visual
evidence, not a fresh native visual matrix.

- Upgrade from v0.3.2 is explicitly skipped for this preparation.
- Native dialog/IME/plugin focus, actual banner appearance and OS click delivery,
  physical mouse/trackpad interaction, live-provider title freshness and broader
  native desktop acceptance remain unverified.
- The killed-TUI fixture restores its shell's termios. Automatic recovery fixes
  mouse reporting only, not keyboard protocol modes, termios or killed processes.
- Identical titles after known conversation changes may remain suppressed until
  a different title arrives. Manual-name reset requires recognized identity.
- Task names in notifications, other Rex proposals and AUR publication remain
  deferred. Existing best-effort attention semantics are unchanged.

## Candidate and packages

Candidate `1b16fee02cac5ab5e7f135707d473f1f0730666e` is pushed. Its
[source comparison](../evidence/2026-11-07-release-0.3.3/candidate.json) confirms
all 506 source/build files match the local checks, including resolved license
symlink contents. The [initial CI run](https://github.com/rengwu/chartr/actions/runs/42378368818)
and [package workflow](https://github.com/rengwu/chartr/actions/runs/42378372725)
identify the exact candidate.

All three package artifacts were downloaded and matched GitHub's archive digests.
All seven payload checksums pass. Linux x86_64/ARM64 version, source revision,
architecture and exact Herdr pin were verified; tar/Debian binaries match and the
x86_64 Arch binaries match its tarball. The AUR recipe matches the archive checksum.
The macOS DMG has the production bundle ID, version 0.3.3, candidate revision,
ARM64 executables and expected sidecar; ad-hoc signature and checksum checks pass.
It was mounted read-only without opening Finder, then detached without launching
Chartr. See the [run's package evidence](../verification/runs/2026-11-07-release-0.3.3.md#candidate-package-results).

Initial macOS CI failed a raw screen-text readiness assertion in the killed-TUI
fixture after its mouse-mode checks completed. Ubuntu and Arch CI passed. The
[diagnosis](../verification/runs/2026-11-07-release-0.3.3.md#ci-fixture-diagnosis)
preserves the failure and explains the test-only correction; the shipping app and
backend source are unchanged.

The correction is committed as `56ce59cf`. All four local real-session tests and
three additional killed-TUI repetitions pass, followed by the normal app build.
The [recheck](../verification/runs/2026-11-07-release-0.3.3-recheck.md) records
byte-identical normal app/sidecar binaries and unchanged shipping source relative
to the inspected packages. The [CI recheck](https://github.com/rengwu/chartr/actions/runs/42382207347)
passes on that fixture commit: macOS, Ubuntu and Arch jobs are all successful,
including real-terminal sessions and Linux X11/GTK fixture checks. The
[receipt](../evidence/2026-11-07-release-0.3.3/ci-recheck.json) identifies the tested
commit. Subsequent preparation-record changes affect documentation and evidence
only.

The [draft release notes](../evidence/2026-11-07-release-0.3.3/release-notes.txt)
are ready. Preparation is complete within the accepted scope; publication remains
pending. No tag or published release is created by this run.
