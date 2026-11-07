---
title: Release 0.3.3 preparation
section: Development
updated: 2026-11-07
status: Local checks passed; platform checks pending
---
# Release 0.3.3 preparation

The user requested **0.3.3**, explicitly skipped the proposed v0.3.2 upgrade
check, and requested simple verification without desktop takeover. Preparation
uses branch `release/0.3.3`; the workspace and both Cargo lockfiles agree.
The [scope decision](../decisions/index.md#dec-release-033-001-prepare-0-3-3-with-simple-verification)
and [candidate run](../verification/runs/2026-11-07-release-0.3.3.md) distinguish
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

Platform CI and package workflows will run on the prepared branch. Candidate
package inspection must verify version/revision, exact bundled sidecar, checksums,
platform architecture and consistency across Linux package formats. Packages are
inspected without launching their applications.

The [draft release notes](../evidence/2026-11-07-release-0.3.3/release-notes.txt)
are ready for publication after preparation. No tag or published release is
created by this run.
