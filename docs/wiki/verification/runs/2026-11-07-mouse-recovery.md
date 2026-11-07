---
title: Killed TUI mouse recovery
section: Verification
updated: 2026-11-07
result: Passed
---
# Killed TUI mouse recovery

Scope: `FEAT-terminals`, `FEAT-session-lifecycle`. Investigate the user's live
imparo terminal, recover that shell, and exercise a killed foreground TUI through
the real private backend and direct-attach stream. No installed-app replacement,
personal backend restart, or agent relaunch is part of this run.

## Incident evidence

The installed Chartr process started at 00:25:16 MYT on November 7. Its private
Herdr daemon (PID 704) started October 19 and survived. The imparo shell was PID
55746, pane `w1Q:p7`, terminal `term_65f1d7be9a91e5f`. The backend process-info API
reported that zsh itself owned the foreground process group. Native inspection
showed `zsh: terminated codexng` and mouse-report fragments at its prompt.

The [backend timeline](../../evidence/2026-11-07-mouse-recovery/incident-timeline.log)
places most agent-to-shell transitions, including imparo, around 00:22:24 MYT;
the attach clients disconnected around 00:25:10–13 and reattached at 00:25:16–18.
This establishes that imparo's agent had ended before the frontend restart.
It does not identify the signal sender or establish the cause of memory pressure.
Tabs represented surviving shell sessions, not resurrected agent processes.

The [baseline identities](../../evidence/2026-11-07-mouse-recovery/baseline-identity.json)
record installed frontend/backend and local baseline sidecar hashes. Existing
uncommitted task-name and layout work was preserved.

## Live recovery

The user explicitly authorized inspection of the running personal app and
switching to imparo. The first keyboard-only attempt could not cancel the input
under the stale protocols; Enter submitted the existing numeric mouse fragments,
which produced command-not-found errors. It did not successfully run the reset.
The shell's PTY also retained raw settings (`-isig`, `-opost`).

After verifying the exact shell and PTY (`/dev/ttys064`), reset sequences were
written as output through that slave to disable mouse, focus, and extended
keyboard reporting. Native input then ran `stty sane`. A native drag selected text
and left an empty prompt without new mouse fragments. This is a **Passed manual
recovery on the installed build**, not proof of automatic recovery in the patch.
Other personal tabs were not reset.

## Implementation

The existing tracked Herdr patch now clears mouse modes when its foreground
observer sees a known non-shell process group return to the owning shell. Both
normal and handed-off runtime paths notify direct-attach clients to stop capture.
It reuses the existing process observation, adding no per-frame process scan.
Unknown probes, live TUIs, transitions between jobs, and modes owned by the shell
itself are preserved. The screen, scrollback, bracketed paste, and PTY termios
settings are not reset.

This is bounded recovery using the existing periodic foreground observer. It
does not guarantee recovery for a job that starts and ends entirely between
observations, a shell without distinct foreground job groups, or a stale state
already present before the observer starts. It does not restore killed processes
or diagnose the original memory exhaustion. Extended keyboard/termios recovery
remains manual; the automatic change is scoped to mouse reporting.

## Executed checks

The [baseline regression](../../evidence/2026-11-07-mouse-recovery/baseline-regression.log)
**Failed as expected**: after SIGKILL, `CHARTR_SHELL_READY` arrived but the attach
stream never disabled any-motion reporting. The helper timed out after five
seconds. This reproduces the persistent mode leak independently of the user's
personal sessions.

All [44 selected upstream tests](../../evidence/2026-11-07-mouse-recovery/backend-results.json)
**Passed**, covering mouse encoding, foreground-group policy, terminal identity,
handoff, and the new recovery behavior. Each ran in a separate test process.
The nine [fetch contract tests](../../evidence/2026-11-07-mouse-recovery/fetch-tests.log)
**Passed**; the [patch check](../../evidence/2026-11-07-mouse-recovery/patch-check.log)
verified exact hash/version agreement and clean application to pinned upstream.

All [four real-session tests](../../evidence/2026-11-07-mouse-recovery/live-regression.log)
**Passed** against the newly [built distributable sidecar](../../evidence/2026-11-07-mouse-recovery/sidecar-build.log):

| Scenario | Result |
| --- | --- |
| Killed TUI while attached; mouse capture released and queued reports discarded | Passed |
| Killed TUI while detached; reattachment does not reenable stale capture | Passed |
| Live backend handoff preserves terminal identity and active TUI mouse demand; subsequent kill releases capture | Passed |
| Clean TUI exit, selection, press/drag/release delivery, reattachment and shell scrollback | Passed |
| Direct-attach session creation and backend crash recovery | Passed |

All [36 Chartr backend client tests](../../evidence/2026-11-07-mouse-recovery/client-tests.log)
**Passed**. `cargo fmt --all --check` passed. The aggregate **Passed** result covers
these bounded checks and the installed-build manual recovery, preserving the
baseline failure above. The [normal app build](../../evidence/2026-11-07-mouse-recovery/app-build.log)
passed after test compilation. The [changed build identity](../../evidence/2026-11-07-mouse-recovery/changed-identity.json)
records the revision, dirty source fingerprint, relevant untracked source hash,
test executable hashes, normal app/sidecar hashes, and toolchains. The sidecar is
`0.8.2-chartr.5a2dee700eee.identity.914a955dff8a`, protocol 22.

The fixture cleanup left no test-owned daemon or Python TUI running. The personal
backend remained PID 704. Build artifacts were retained locally; no commit,
release, or installed-app replacement was performed.

The regression fixture launches a disposable Python TUI, enables SGR any-motion
reporting, and is killed with SIGKILL without cleanup. The enclosing shell restores
its own termios. Selected cases cover exit while attached, exit while detached
followed by reattachment, and live daemon handoff followed by exit. Existing
selection/gesture/scrollback and backend crash-recovery tests remain in scope.

Linux and a fresh native frontend running the patched sidecar are outside this
bounded run. No release or full desktop acceptance is claimed.
