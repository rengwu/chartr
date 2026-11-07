---
title: Tabs frame left-edge alignment
section: Verification
updated: 2026-10-30
result: Passed
---
# Tabs frame left-edge alignment

The user's Tabs-mode screenshot showed the pane frame touching the left window
edge while the right edge retained an outer gap. This corrects
[FEAT-panes](../../features/panes.md): Tabs now has matching 4 px left and right
insets. The left inset follows the existing tab-strip animation; Spaces keeps
the frame flush with the sidebar boundary. This refines the October 22 frame
appearance contract for Tabs mode.

## Build and isolation

HEAD `369298062bfd5ba00ad0c273d580ce408797b19c` plus the existing dirty task-name
implementation and this two-line layout change. The
[identity](../../evidence/2026-10-30-tabs-frame-inset/identity.json) records source
and patch hashes, the ad hoc signed binary, pinned Herdr, OS, toolchain, process,
bundle and explicit XDG roots. The normal desktop build ran after the tests.

Native checks used a uniquely identified bundle under `target/` and the private
profile `/tmp/chartr-verify.1sh4fhoz`. The process, private socket and SQLite state
were checked before UI interaction. Three disposable zsh tabs belonged to the
fixture project; `INSET_CHECK_READY` printed through the real terminal. The user
closed the initial test window; the same isolated process was verified and
reopened through Finder before continuing. Computer Use through Node REPL worked
for this run; earlier failed driver attempts remain historical evidence.

## Results

| Check | Observed result | Evidence |
| --- | --- | --- |
| Tabs frame | Three outer tabs rendered correctly, including selecting the first tab. Left and right outer gaps each measure 8 physical pixels in the 2x capture, equivalent to 4 logical pixels. | [Tabs screenshot](../../evidence/2026-10-30-tabs-frame-inset/tabs.png), [pixel measurements](../../evidence/2026-10-30-tabs-frame-inset/geometry.json), [accessibility](../../evidence/2026-10-30-tabs-frame-inset/tabs-accessibility.txt) |
| Spaces frame | Frame begins at the existing 280 px sidebar boundary; right gap remains 4 px. The test terminal rendered the readiness marker. | [Spaces screenshot](../../evidence/2026-10-30-tabs-frame-inset/spaces.png), [pixel measurements](../../evidence/2026-10-30-tabs-frame-inset/geometry.json) |
| Existing transition tests | `cargo test --workspace --locked mode::tests`: 10 passed, zero failed. Includes mode restoration, interrupted transitions, reversals and reduced motion. | [Test log](../../evidence/2026-10-30-tabs-frame-inset/tests.log) |
| Normal build | `cargo build -p chartr --locked` passed. Existing dependency warning for `block 0.1.6` remains. | [Build log](../../evidence/2026-10-30-tabs-frame-inset/build.log) |
| Formatting and whitespace | `cargo fmt --all --check` and `git diff --check` passed. | Executed against this working tree. |

Native coverage is limited to these macOS frame states and terminal readiness.
Animation behavior is covered by the existing transition tests and source review,
not a captured native animation trace. This run does not reclassify the earlier
task-name feature's unexecuted native scenarios or establish Linux coverage.

## Cleanup

Quit exited the test app with the private profile's terminate-on-exit preference.
The private backend's `pane.list` returned no panes. Only that recorded backend
PID was stopped; the temporary profile and bundle were removed after checking
that no run-owned processes remained. [Cleanup evidence](../../evidence/2026-10-30-tabs-frame-inset/cleanup.json)
records the backend response. The installed app and personal sessions were not
used as fixtures.
