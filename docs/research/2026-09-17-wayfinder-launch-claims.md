# Wayfinder launch failure and claim recovery — 17 September 2026

Ticket 07 in Envision's implementation map was claimed by `w4:p1` at
00:31:28 MYT on 17 September. The terminal disappeared immediately afterward.

## Evidence

- Herdr's log in `~/.config/chartr/chartr/herdr/herdr-server.log` records the
  shell spawning as PID 48020 at `2026-09-16T16:31:28.727488Z`, followed by a
  bus error at `16:31:29.230521Z`.
- macOS's `~/Library/Logs/DiagnosticReports/zsh-2026-09-17-003129.ips` identifies
  `/bin/zsh`, PID 48020, as the crashed process. Its main-thread stack is
  `hend → loop → zsh_main`; termination is SIGBUS / EXC_BAD_ACCESS.
- The selected registration uses adapter `codex`, default prompt delivery, and
  no extra arguments. That delivery embeds the complete Wayfinder prompt as a
  quoted command argument, then sends the command through the interactive PTY.
- An isolated `/bin/zsh -f -i` PTY reproduced SIGBUS with a large synthetic
  multiline command. Sending the same command via bracketed paste or sourcing
  a script completed successfully and preserved the exact payload. These
  experiments used a shell builtin, temporary files, and synthetic text; they
  did not launch Codex or run the user's ticket.

The evidence points to interactive zsh history processing as the launch failure,
rather than a recorded Codex startup error. The claim remained because delivery
was accepted before the shell crashed. Existing rollback handles delivery errors,
not the later lifetime of the launched process.

## Changes

- Host launch delivery now stages multiline commands or commands over 4 KiB in
  a private temporary script. Only a short source command reaches the line editor.
  Sourcing preserves the user's shell and the provider's quoting and arguments.
  Typed prompt bytes after the command are passed through unchanged.
- Temporary scripts use owner-only permissions and unlink themselves when read.
  Session ownership retains unconsumed scripts across reattachment and cleans
  them up when the session closes or delivery fails.
- Claimed tickets expose **Open session** and **Release claim…**. Release uses
  a confirmation modal bound to the displayed ticket and session ID. It works
  when the terminal is gone and rejects a changed claim instead of clearing a
  newer session's work. The bridge is tested with provider services unavailable;
  the normal catalog still requires Agent and Skill sources to be enabled to
  open Wayfinder (disabling either also disables the dependent pane).
- Release only removes claim metadata; it does not terminate a session or edit
  the ticket's answer. The next snapshot restores the appropriate frontier state.

The existing claim on the user's ticket was not edited during investigation.
After restarting the updated development build, it can be released through
the ticket's detail pane before retrying the launch.

## Verification

- 17 Wayfinder Rust tests passed, including ended-session recovery, stale-claim
  rejection, claim-before-input ordering, and failed-delivery rollback.
- Three session tests passed, covering exact large-prompt delivery, script
  permissions/removal, typed input, attachment replacement, and abandoned launches.
- Five JavaScript layout tests passed.
- The browser smoke suite passed in Chromium and WebKit, including release
  cancellation, conflict reporting, successful release, and restored launch controls.
- `cargo check -p chartr --locked` and `cargo build -p chartr --locked` passed.
  The updated development executable is available at `target/debug/chartr`.
