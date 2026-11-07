---
title: Rex beta reverse-engineering evidence
section: Discussions
updated: 2026-10-29
status: Binary analysis and bounded experiments
---
# Rex beta reverse-engineering evidence

This is the technical basis for [Rex ideas for Chartr](rex-beta-adoption.md).
The request was to decompile the installed Rex beta and deeply investigate ideas
for Chartr. Work included native binary inspection, recovery of Go function
metadata, selected ARM64 disassembly and manual reconstruction, API introspection,
synthetic runtime experiments, native UI inspection, public documentation, and
comparison against Chartr source. This is research, not an accepted implementation
plan or a Chartr product verification run.

## Identity and reproducibility

The [manifest](../evidence/2026-10-29-rex-beta/manifest.json) records binary hashes,
versions, OS, Chartr revision and the initial dirty-tree fingerprint.

| Component | Inspected identity |
| --- | --- |
| Installed application | `/Applications/Rex Beta.app`, 0.1.0, build 1497, preview channel |
| Client source identity embedded in plist | `f8c3eac550a507d5069bce3d947fd30aa7fc17ab` |
| Server source identity embedded in executable | `2db882139236ba19730a5a35e172729e1dfc96ee`, built 2026-10-26 |
| Server version | `0.0.1793047680-g2db882`, API revision 7, supported range 0–7 |
| Architecture | Universal x86_64/ARM64 binaries; ARM64 slice analyzed |
| Host | macOS 27.0, build 26A5368g, Apple Silicon |
| Chartr source | `369298062bfd5ba00ad0c273d580ce408797b19c` |

The client and server have different source revisions. Treating the GUI build
number as the server's source identity would be wrong. The server embedded in the
application and the installed version-1497 server have identical SHA-256 hashes.
The original tree already contained uncommitted Harp analysis and changes to the
wiki home, open questions and generated reader; those were preserved.

`rex` was not on this command environment's PATH. The bundled executable was
invoked by absolute path. A mounted DMG also had the same bundle identifier;
native inspection explicitly selected the copy in `/Applications`.

## What decompilation recovered

The GUI is a compiled Swift/AppKit/SwiftUI application. The server/CLI is a Go
executable with native terminal components. There is no Electron JavaScript
application to unpack. Stripping removed most ordinary application symbols, but
Go's runtime metadata and Swift reflection/debug strings remained useful.

The [inventory](../evidence/2026-10-29-rex-beta/inventory.json) contains:

- 47,161 Go function entries recovered from `__gopclntab`; 6,380 have Rex's
  first-party package prefix. This includes generated wrappers and generic
  instantiations, not 6,380 distinct product features.
- 377 embedded Rex Swift source filenames. A separate scratch extraction found
  374 Rex-related demangled type-like strings; this is not a complete Swift type
  census or recovered source code.
- All 73 advertised server methods, the two advertised block creators, their
  methods/events, and all 87 GUI actions advertised by the running client.
- Terminal creator: 16 methods and 13 events. Onboarding is the other built-in
  creator. Executable-plugin infrastructure exists, but that does not establish
  a shipping collection of editor, browser, diff or dashboard blocks.

Fourteen selected Go functions were disassembled. Their exact addresses,
source-file/line metadata and direct call targets are in the
[reverse-engineering index](../evidence/2026-10-29-rex-beta/reverse-engineering-index.json).
These focus on status storage, process exit, client ownership, terminal idling,
snapshot restoration and output backpressure. The semantic reconstructions below
are partial manual decompilation, not original source or a complete decompiler
listing of the application.

The [extractor](../evidence/2026-10-29-rex-beta/symbols.go.txt) uses Go's
`debug/macho` and `debug/gosym`. An important correction: `__text` begins before
Go's actual text because this is a cgo binary. Using that section's address gave
misaligned function addresses. The final extraction uses `moduledata.text`,
`0x100009640`, and excludes bound-method wrappers when selecting bodies.
Only the corrected index is retained as evidence.

Complete scratch strings, metadata, API schemas and selected disassembly remain
under `/tmp/chartr-rex-analysis-20261029/` for this session. They are temporary;
the durable report and selected evidence are in this wiki. Proprietary binaries,
full string dumps and original resources were not added to the repository.

## Architecture reconstructed from independent surfaces

```text
Swift native UI
  RexMacOS: windows, palette, sidebar, settings, terminal presentation
  RexShared: workspace state, actions, activity projections, seen ledger
  RexSync: connections, snapshots, membership, block ownership
       |
       | control API + separately ticketed terminal data streams
       v
Go server and CLI
  sessions -> windows (tabs) -> tiled/floating layers -> blocks
  layout/focus/labels/ownership/event sequencing
  Lua actions, keymaps, events, scripts and executable block creators
       |
       v
terminal block
  process and PTY -> libghostty state -> output fanout/snapshots
  status records, process metadata, directory listing, capture and blobs
```

This map is supported by the recovered package names, client type/source names,
library linkage, and live API schemas. It is not proof of every internal call
relationship. Both the GUI and the server contain terminal-related code; there
is no basis here for a simplistic claim that Rex eliminates all duplicate
emulation while Chartr does not.

The bundle also contains Ghostty rendering resources, a Metal library, SwiftNIO
resources, Sparkle, a design-token theme catalog and icon assets. The theme
catalog contains 41 entries. Five material-style JSON resources cover several
interface treatments. These indicate a deliberate design system, not a reason
to reproduce its assets or increase Chartr's theme count.

## Selected manual reconstructions

### Status inheritance and replacement

`programStatusStore.effectiveAppLocked`, ARM64 `0x1006b6cd0–0x1006b6e30`,
checks the record's own app string, then repeatedly trims its ID at the last
slash and looks up ancestors in the record map. The root is considered too.
In readable pseudocode:

```text
if record.app is present:
    return record.app
id = record.id
while id is not empty:
    id = parent_path_or_root(id)
    if records[id] exists and records[id].app is present:
        return records[id].app
return no_app
```

This reconstruction is supported by the slash-byte comparison, map lookups and
return branches, and by a runtime child record inheriting the root's `cargo`
identity. It is independent of a guessed window title.

`programStatusStore.applyLocked`, `0x1006b6310–0x1006b6a10`, contains state
selection, map replacement, ancestor-app resolution, an explicit `0x40` capacity
comparison and eviction logic. Runtime probes established that this build keeps
64 records: adding the 65th removed the oldest untouched record. The broader
protocol's maximum is not this build's configured capacity.

Changing a child from blocked-with-message to working without those fields
removed its previous kind/message. An idle root did not erase a blocked child.
Clearing an intermediate parent removed descendants even when that parent had
no record. Invalid IDs did not overwrite the root; unknown states and decoded
control characters were ignored in the exercised reports.

### Process lifetime and outcome lifetime are different

`Block.childExited`, `0x1006aeb30–0x1006aedd0`, updates exit state, removes
selected status records and publishes the process exit. The runtime fixture
confirmed that working/blocked records disappear while done/error records remain.
A simulated shell-prompt boundary also cleared in-progress records and retained
completed outcomes. Alternate-screen entry did not erase status records.

This is valuable separation: a process can end while a result still needs review.
However, a program-reported outcome is still a claim by that program. It is not
proof that a repository test suite passed, nor a reason to relabel all heuristic
inactivity as success.

The public [status protocol](https://www.superlogical.com/rex/docs/build/program-status)
provides a portable way for cooperating programs to report state, progress,
blocked reason and task identity. Its reporting and presentation layers are
separate. The experiments above establish only the specific fixture behavior,
not universal provider adoption or correct real-agent cancellation handling.

### Idle terminal resources have an explicit lifecycle

`mux.Terminal.idleCheckLocked`, `0x1006984c0–0x100698630`, rejects stale timer
callbacks, checks current state, compares elapsed inactivity against a configured
interval, then calls `suspendLocked` or rearms the timer. This is a timer-driven
resource policy, not merely an animation preference.

`suspendLocked`, `0x100698630–0x100698c50`, examines Kitty image placements,
flushes pending state, reads title/directory/modes, calls terminal suspension and
drops idle caches. `restoreLocked`, `0x100699200–0x100699480`, has snapshot
restoration and failure handling, including client closure/fresh-state paths,
and rearms activity tracking. `parkTimerFired` checks a timer token and activity
state before parking terminal storage.

Separate scratch `.snap` files appeared during the experiment, ranging from
657 to 1,361 bytes for these tiny fixtures. That is an observation of snapshot
creation, not a measurement of memory savings or restoration latency. Symbols
also identify compression passes, chunk queues, memory accounting and sealed
spool files. No claim is made that they all executed in the tested path.

`pauseTimerFired`, `0x100696d20–0x100696ed0`, walks client progress, closes
clients on its stalled path, calls `resumeOutputIfCaughtUpLocked`, and rearms
when output is still paused. Together with the API's bounded event backlog,
this suggests a deliberate policy for slow consumers. Network loss, fairness
and stall thresholds were not benchmarked.

### Ownership is about presentation arbitration

`Session.nextOwnerLocked`, `0x100584e40–0x100585040`, is part of claim
selection. The advertised API distinguishes taking/requesting ownership,
disowning, disconnects and reconnects. Its schema says the owner controls size
and theme arbitration; ownership grants no additional authorization.

This distinction matters if Chartr ever supports two clients viewing one PTY.
A phone's viewport should not repeatedly resize a desktop editor merely because
both are attached. The policy is inspected, not validated with competing clients.

## API details that matter to Chartr

The live schemas are substantially more useful than a list of verbs. They include
argument/result shapes, examples, validation errors, ordering constraints and
revision information. This supports the vendor's
[self-describing API model](https://www.superlogical.com/rex/docs/automate/server-api).

| Contract observed in this installed version | Why it matters |
| --- | --- |
| Stable session/window/block IDs; names and positions resolve through explicit APIs | Automation can target identity and report ambiguity rather than guessing from visible labels. |
| Independent server operations and GUI client actions | Creating a pane need not switch the user's currently viewed session. |
| `focus=false` when creating work | An agent can add a monitor or tests without interrupting typing. Verified for a split. |
| Structured nested layouts with documented rollback | A workspace recipe can be prepared as a unit. Invalid syntax and a valid layout with an unavailable creator left existing state intact in the exercised checks. |
| Bootstrap snapshot plus a sequence fence | Consumers can reconcile events that arrived around the initial response. This is a schema-observed contract, not a race stress test. |
| Event lag above 8 MiB closes the connection rather than skipping events | Reconnection must obtain a fresh baseline; silent loss is avoided by the documented policy. |
| One-use, expiring, principal-bound data tickets | Terminal data streams have an explicit attachment/handshake boundary. This was inspected, not security-audited. |
| Output chunks carry stable blob ID, byte offsets, total and EOF | Large captures can be reconstructed without silently cutting a log. Verified byte for byte. |
| Process endpoint exposes exact exit code, cwd and limited invocation identity | Good task labeling and job completion do not require exposing all command arguments. |
| Directory listing returns missing/not-directory/denied separately | A path picker can provide a useful failure state instead of an empty-looking directory. Missing and regular-file inclusion were exercised. |
| Explicit take/request ownership and pending size requests | Multi-client viewing requires viewport policy, not just a shared socket. |
| API compatibility range and conversion packages | Version mismatch is anticipated, although cross-version combinations were not tested. |

A few advertised CLI paths are incomplete: the bundled guide explicitly says
`rex resize` and `rex block new` are not implemented in this version and points
to APIs instead. This is a documentation observation, not a newly reproduced bug.
The shell, block and client namespaces also differ; a future Chartr CLI should
avoid making users learn backend vocabulary that conflicts with its UI.

## Native interface inspection

All nine Settings categories were opened without changing preference values:
Server, Interface, Themes, Terminal, Shortcuts, Integrations, Notifications,
Remote Hosts and Developer. These are UI observations, not assertions that every
setting behaves correctly.

| Surface | Observed detail | Interpretation |
| --- | --- | --- |
| Server | Running state/uptime, details, stop action, Tailscale setup, client label and Remote Control toggle | Make backend identity and lifecycle understandable. Remote Control was off and was not enabled. |
| Interface | Above/left/right tabs, up to three pane icons, compact/comfortable panes, active borders, inactive fading, focus-follows-mouse, animation choices | Strong focus/ownership cues are more transferable than decorative materials. |
| Themes | Light/dark cards, duplicate/create, Ghostty/iTerm import, extended palette | Preview a real palette before changing it. Import behavior was not exercised. |
| Terminal | Font/style/fallback preview, ligatures, contrast adjustment, cursor preview, option-key policy, selection choices | Settings demonstrate their visible effect. Native IME/accessibility behavior was not tested. |
| Terminal context | Insert Path, generated-title preference, source-directory inheritance | Improve task context and path entry without becoming an IDE. |
| Shortcuts | Search by command or key, winning binding explanation, GUI overrides, Lua export, argument-only actions disclosed | Discovery and conflict explanation are a larger gain than adding more shortcuts. |
| Integrations | Agent setup and optional session naming | Teach agents the application's own interface; these are workflow integrations, not proof of outcome detection. |
| Notifications | Separate tab/badge/sound/system channels, per-state preferences, idle-process inclusion | Offer notification control without conflating activity with unread acknowledgment. |
| Remote Hosts | Empty list with add-host entry point | Remote access is a designed product area, not merely SSH typed into a local terminal. |
| Developer | Developer-mode toggle | Debug tools are separated from normal settings; the toggle was not enabled. |

The command palette visibly searches commands, sessions and tabs, supports
subpages, and offers shortcut editing. Searching `zoom` returned pane zoom and
font-size commands, consistent with action keywords rather than label-only
substring matching. Its result rows did not appear in the captured accessibility
tree even though they were visible in the screenshot; this is a tooling/AX
observation, not a complete accessibility assessment.

The integrated path picker was opened and cancelled without inserting a path.
It exposes a path field and accessible file/folder rows. A late View-menu
inspection produced stale element IDs and then an unavailable screenshot.
Therefore no successful interaction with the Activity overview, native status
transition, notification click, or menu-bar activity workflow is claimed. The
Activity action, UI types, preference pages and keywords are separately evidenced.

## Runtime experiments

A separate foreground server used only:

```text
Unix socket: /tmp/chartr-rex-analysis-20261029/server.sock
State:       /tmp/chartr-rex-analysis-20261029/state
Config:      /tmp/chartr-rex-analysis-20261029/init.lua (empty)
```

Only synthetic programs were created there. Existing Rex sessions were not
renamed, sent input, closed or restarted. No remote listeners, Tailscale sign-in,
agent installation, app update or preference change was performed. Chartr was not
launched or its backend restarted.

After the experiments, the scratch server was stopped and its process/socket
were confirmed gone. The existing default server retained its original PID and
three sessions. The [cleanup receipt](../evidence/2026-10-29-rex-beta/cleanup.json)
records those checks; it does not infer anything about the contents of user sessions.

The [summary](../evidence/2026-10-29-rex-beta/probe-summary.json) records 25
successful bounded checks across the final main, supplementary and exit probes.
An earlier failed assertion and an interrupted harness attempt remain preserved.

| Area | Observed outcomes | Evidence |
| --- | --- | --- |
| Status records | Working/progress, parent inheritance, idle parent with blocked child, replacement, malformed IDs, control text, unknown state, subtree clear | [Main results](../evidence/2026-10-29-rex-beta/probe-results.json) |
| Lifetimes | Prompt cleanup, alternate-screen preservation, keep-open exit code 7 and in-progress cleanup | [Main results](../evidence/2026-10-29-rex-beta/probe-results.json) |
| Layout | Explicit label, 70/30 split without focus theft, zoom/unzoom, detach without termination, invalid-layout rejection | [Main results](../evidence/2026-10-29-rex-beta/probe-results.json) |
| API | Full chunk reassembly, unavailable-creator rollback, floating layer, explicit missing directory and files | [Supplement](../evidence/2026-10-29-rex-beta/supplement-results.json) |
| Capacity and exit | 64-record cap with oldest eviction; `run --wait` returned exit code 7 | [Supplement](../evidence/2026-10-29-rex-beta/supplement-results.json) |
| Default close policy | Quick failure remained inspectable without `--keep-open`; quick success removed its block | [Exit checks](../evidence/2026-10-29-rex-beta/exit-results.json) |

Two harness corrections are explicit. The first fixture used canonical PTY input,
so a long JSON command exceeded its input-line limit. The rerun disabled canonical
mode. The capture assertion then looked for `content` in a blob reply, whose schema
actually uses `data`; the supplemental check reconstructed all chunks and matched
the complete capture. Neither is evidence of a Rex product failure.

Reproduction sources:
[fixture](../evidence/2026-10-29-rex-beta/fixture.py.txt),
[main probe](../evidence/2026-10-29-rex-beta/probe.py.txt),
[supplement](../evidence/2026-10-29-rex-beta/supplement.py.txt),
[exit probe](../evidence/2026-10-29-rex-beta/exit-probe.py.txt), and the
[preserved first attempt](../evidence/2026-10-29-rex-beta/probe-attempt-1.json).
These scripts document the experiment and require the recorded scratch server;
they are not portable Chartr regression tests. The main script intentionally
preserves its mistaken assertion; use the supplement for that result.

## Remote access, plugins, configuration and update limits

Recovered packages cover Tailscale, Unix peer identity, HTTP/WebSockets,
login-server/PAM boundaries, client actions, Lua execution/timers/events,
executable block discovery/reload, compatibility conversion, agent-skill setup,
terminfo, SSH configuration, Apple privacy and sandbox extensions.
Their presence maps the implemented subsystem surface; it does not establish
correctness or completeness on every operating system.

The public [remote-access guide](https://www.superlogical.com/rex/docs/linux/remote-access)
describes opt-in listeners and Tailscale routes. No remote connection or
cross-device resize/contention scenario was tested. The
[upgrade guide](https://www.superlogical.com/rex/docs/linux/upgrading) distinguishes
package installation from restarting a server and explains that restart effects
depend on server mode. Persistent sessions must not be advertised as surviving
arbitrary daemon termination or reboot on the evidence of ordinary client detach.

The [agent setup guide](https://www.superlogical.com/rex/docs/automate/agent-skill)
explains that startup hooks supply location/context and the installed CLI supplies
the skill's current instructions. This does not mean those hooks report all
agent outcomes. The
[key-mode guide](https://www.superlogical.com/rex/docs/customize/key-modes) and
[Lua reference](https://www.superlogical.com/rex/docs/reference/lua) describe
customizable modal controls and a general scripting runtime. Chartr can adopt
a discoverable command interface without taking on that entire runtime.

The vendor's [privacy statement](https://www.superlogical.com/rex/docs/reference/privacy)
describes local-only default server communication and its outbound-connection
policy. Plist inspection confirms signed Sparkle-feed configuration, but no packet
capture or independent no-telemetry audit was performed. Framework or dependency
presence alone is not evidence of actual network transmission.

## Coverage boundary

This investigation is broad across the installed bundle and deep in selected
mechanisms. It does **not** claim every machine instruction was understood, full
Swift/Go source was recovered, or every advertised path was exercised.
Unverified areas include real-agent matrices, native notifications and sounds,
crash/reboot recovery, update execution, multi-client arbitration, remote/mobile
clients, plugin authoring/reload, sustained throughput, idle power, memory scaling,
IME, graphics-protocol compatibility and full accessibility.

The resulting recommendations depend mostly on directly observed UI/API behavior
and current Chartr source. Speculative architectural conclusions and larger
product bets remain labeled as such.
