---
title: Terminal task-name implementation
section: Verification
updated: 2026-10-30
result: Untested
---
# Terminal task-name implementation

Implementation of [DEC-REX-003/004](../../decisions/index.md#dec-rex-004-implement-terminal-task-names),
covering [FEAT-terminals](../../features/terminals.md),
[FEAT-settings](../../features/settings.md) and individual terminal titles in
[panes](../../features/panes.md). The user explicitly requested implementation
using work-until-grill after accepting the design.

## Scope and build

The host resolves manual name → available conversation title → process/shell.
Manual names and last known conversation identity use a versioned SQLite
`terminal_names` value through the existing ordered snapshot writer. Default Keep
and optional conversation reset use the normal Terminal settings store. Context
menus, an unbound action and the palette reuse the native rename modal.

Names are keyed by persistent terminal ID. Runtime handoff reconciliation also
preserves item ownership when pane numbers move or swap; reader-exit callbacks
match the actual attachment. Provider icons/activity and notification title
formatting remain separate. No provider adapters, transcript scans or backend
protocol changes were added.

Base revision: `369298062bfd5ba00ad0c273d580ce408797b19c`, with pre-existing wiki work
and the implementation changes. macOS 27.0 (26A5368g), arm64, Rust 1.96.0. The normal
`cargo build -p chartr --locked` completed successfully after test compilation.
[Build identity](../../evidence/2026-10-30-terminal-task-names/build-identity.json)
contains changed-source hashes, tracked patch fingerprints, and the normal app,
unit-test, live-test and pinned Herdr binary hashes. Source hashes remained unchanged
through the final build. [Git status](../../evidence/2026-10-30-terminal-task-names/git-status.txt)
records the dirty tree, including the pre-existing documentation work. No commit/release or
installed application replacement is included.

## Results

| Scenario | Expected and observed | Result | Evidence |
| --- | --- | --- | --- |
| Naming rules and identity | Manual/automatic/fallback precedence; validation; both lifetime policies; missing/mismatched/normalized identity; stale title suppression; restart and new-terminal isolation passed. | Passed | [Workspace tests](../../evidence/2026-10-30-terminal-task-names/workspace-tests.log) |
| SQLite durability | Empty/older storage, attachment-independent names, attention isolation, failed transaction retry, ordered flush and malformed/future-record preservation passed. | Passed | [Workspace tests](../../evidence/2026-10-30-terminal-task-names/workspace-tests.log) |
| Settings and command entry points | Both preferences save/reload; keymap bind/clear; palette selection and GPUI context-menu rename/reset dispatch to the exact item passed. | Passed | [Workspace tests](../../evidence/2026-10-30-terminal-task-names/workspace-tests.log) |
| Runtime ownership | Synthetic pane-number swaps preserve terminal items, labels and cross-space ownership; remapped reader exit reaches the correct attachment. | Passed | [Workspace tests](../../evidence/2026-10-30-terminal-task-names/workspace-tests.log) |
| Workspace suite | 450 passed, zero failed, seven ignored, including doc tests. Ignored/native cases are not counted as passed. | Passed | [Log](../../evidence/2026-10-30-terminal-task-names/workspace-tests.log) |
| Isolated real backend | All three ignored live-session tests passed: direct attachment, crash/recovery without resurrecting dead sessions, and shell/TUI mouse routing. | Passed | [Log](../../evidence/2026-10-30-terminal-task-names/live-backend.log) |
| Formatting and plugin example | Workspace formatting check and locked standalone plugin-example check passed. | Passed | [Formatting](../../evidence/2026-10-30-terminal-task-names/fmt.log), [plugin check](../../evidence/2026-10-30-terminal-task-names/plugin-check.log) |
| Normal desktop build | `cargo build -p chartr --locked` passed, after test compilation, with the normal non-test desktop feature graph. | Passed | [Build log](../../evidence/2026-10-30-terminal-task-names/build.log), [identity](../../evidence/2026-10-30-terminal-task-names/build-identity.json) |
| Python project checks | 24 cases ran: 15 passed and 9 packaging/platform cases skipped on macOS. | Passed | [Log](../../evidence/2026-10-30-terminal-task-names/python.log) |
| JavaScript plugin/layout checks | 10 tests passed. | Passed | [Log](../../evidence/2026-10-30-terminal-task-names/node.log) |
| Native rename/reset/cancel, inner pane menus, focus beside embedded surfaces, live preference and frontend reopen | Could not obtain native inventory, including the user's requested retry and a fresh automation session. No product interaction was attempted. | Untested | [Driver attempts](../../evidence/2026-10-30-terminal-task-names/desktop-driver.txt) |

## Execution notes

Commands: `cargo test --workspace --locked --no-fail-fast`, `cargo fmt --all --check`,
`cargo check --manifest-path examples/plugins/hello/Cargo.toml --locked`,
`python3 -B -m unittest discover -s scripts/tests -v`, and
`node --test crates/chartr/tests/plugin_bridge.cjs plugins/wayfinder/tests/layout.test.mjs`.
The live backend checks ran the exact already-built workspace test artifact:
`target/debug/deps/live_session-3e4e2f6b3cfb3ed6 --ignored --nocapture --test-threads=1`.
A separate `cargo test -p chartr --test live_session ...` invocation began recompiling
a different feature graph; it was interrupted and replaced with this same test
binary's ignored cases. The completed direct run passed all three checks.
The existing fixture creates short private `/tmp/cr-*` namespaces and disposable
shells, with daemon shutdown and temporary-directory cleanup in its Drop path.

Initial compilation found missing imports and a test selector requiring a static
string. These were corrected before the passing run; the
[diagnostics](../../evidence/2026-10-30-terminal-task-names/compilation-attempts.txt)
preserve those attempts. The compiler also reports an existing future-Rust warning
for dependency `block 0.1.6`; the current build/test checks accept it.

## Limits

Aggregate **Untested** because selected native interaction scenarios remain
unexecuted. Automated GPUI tests are not proof of macOS compositor/IME behavior.
Agent title/identity coverage remains best effort; live agents and Linux desktop
coverage are not established by synthetic identity fixtures. Notification naming
and notification testing remain explicitly deferred.

The unavailable desktop driver does not require a product-policy decision. Once
it is available, use the [isolated naming recipe](../../features/terminals.md#naming-recipe).
Never use the installed personal profile as a test fixture. No native test
instance or personal-session cleanup was needed for these failed inventory calls.
Wiki metadata, links and generated snapshot checks passed; those checks validate
document integrity and do not change the native verification verdict.

## Project suffix follow-up

The user's screenshot showed `Say hello | imparo` beneath the `imparo` space.
The requested display is `Say hello`. Automatic title normalization now removes
an exact trailing ` | <working-directory name>`; other task text and manually
entered names are preserved. This is a later source snapshot than the original
450-test run above.

| Check | Observed result | Result | Evidence |
| --- | --- | --- | --- |
| Focused naming regression | All 11 naming tests passed, including the screenshot example before/after observation, manual-name preservation, embedded pipes, unmatched suffixes and startup-title fallback. | Passed | [Tests](../../evidence/2026-10-30-terminal-task-names/suffix-tests.log) |
| Normal desktop build | Locked normal build completed successfully after test compilation. | Passed | [Build](../../evidence/2026-10-30-terminal-task-names/suffix-build.log), [identity](../../evidence/2026-10-30-terminal-task-names/suffix-identity.json) |
| Formatting and document structure | Formatting, diff whitespace, report metadata, wiki build and snapshot freshness checks passed. | Passed | Commands recorded below |

Commands: `cargo test --workspace --locked terminal_names::tests`,
`cargo build -p chartr --locked`, `cargo fmt --all --check`, `git diff --check`,
the installed jstack report checker, and `node docs/wiki/_build.mjs` with its
`--check` pass. No new native UI interaction was claimed for this focused display
correction. The original native coverage limits remain in place.
