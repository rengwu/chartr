---
title: Rex ideas for Chartr
section: Discussions
updated: 2026-10-30
status: Task-name policy accepted; implementation planned
---
# Rex ideas for Chartr

The best lesson from Rex is to make every running piece of work **identifiable,
findable, inspectable and operable**. Chartr already has persistent terminals,
spaces, groups, agent launching, unread attention and a richer project-tool
system. Its opportunity is to connect those capabilities into a coherent daily
workflow rather than reproduce Rex's terminal stack or visual styling.

My recommended starting point is **terminal task names + a workspace switcher +
an activity overview**. The next substantial investment should be **a small,
agent-usable local control API**, followed by **reusable workspace recipes**.
Those additions would improve Chartr's existing product while taking advantage
of its Agent, Skills, Saved Prompts, Markdown Prompt and Wayfinder capabilities.

These are recommendations, not accepted decisions. The user requested analysis
and suggestions, not product implementation. Existing attention, private-backend
and view-mode decisions remain in force. The earlier
[Harp proposals](harp-adoption.md) remain unselected; Rex provides additional
evidence for several of them.

The October 30 [prioritization interview](../journal/2026-10-30-rex-prioritization-feedback.md#accepted-scope-and-next-stage)
accepts a focused first deliverable and exploration of design and feasibility
before implementation. The [submitted shortlist](../journal/2026-10-30-rex-prioritization-feedback.md#selected-target)
selects only **#1, stable terminal task names**. The
[design answers](../journal/2026-10-30-rex-prioritization-feedback.md#accepted-naming-policy)
accept automatic conversation titles with manual override, configurable manual-name
lifetime, and provider icon plus task name; notification naming is deferred.
Its [implementation plan](terminal-task-names.md) records the next work.
The user wants ideas shown individually and permits multiple selections when
several apply; the other proposals remain unselected.

## Research basis

The [technical investigation](rex-beta-internals.md) records the exact scope,
limits, decompilation method, native inspection and experiments. It examined Rex
0.1.0 build 1497, recovered 6,380 Rex-specific Go function entries, mapped 377
embedded Swift source filenames, disassembled 14 selected functions, enumerated
73 server methods and 87 GUI actions, inspected all nine Settings categories,
and recorded 25 successful bounded synthetic checks with two harness corrections
preserved. This is not a claim that every function or feature was exercised.

Chartr was compared at
[`369298062bfd5ba00ad0c273d580ce408797b19c`](https://github.com/rengwu/chartr/tree/369298062bfd5ba00ad0c273d580ce408797b19c).
Source and current feature records were inspected; Chartr was not built or
launched. Existing uncommitted Harp wiki work was preserved.

Evidence shorthand below:

- **UI**: visible native controls/screens, without exercising every setting.
- **API**: contract advertised by this installed server/client.
- **Runtime**: an isolated synthetic experiment, never an unrelated user session.
- **Binary**: recovered metadata or selected disassembly; not original source.
- **Proposal**: an adaptation for Chartr, not a claim that Rex already does it.

## Ranked recommendations

Scope is relative: **S** is a contained UI/model change; **M** crosses several
existing modules; **L** introduces a substantial capability or backend contract.
These are not time estimates. Priorities combine everyday value, fit and cost.

| Rank | Improvement | Value | Scope | Recommended timing |
| --- | --- | --- | --- | --- |
| 1 | Stable task names for individual terminals | Distinguish several agents immediately | S–M | First slice |
| 2 | Searchable workspace switcher | Reach any space, group or terminal quickly | M | First slice |
| 3 | Compact activity overview and next-unread navigation | Triage many concurrent tasks | M | First slice |
| 4 | Local control API, CLI and versioned agent guide | Let agents operate Chartr reliably | L, small initial API | Next foundation |
| 5 | Reusable workspace recipes | Recreate useful project workflows in one action | M–L | After control foundation |
| 6 | Pane zoom and keyboard resizing | Read one busy pane without losing the arrangement | S–M | Independent quick improvement |
| 7 | Set aside running work | Reduce clutter without terminating processes | M | After naming/navigation |
| 8 | Named jobs with truthful completion and retained failures | Make tests/builds easier to monitor and revisit | M–L | With recipes/control API |
| 9 | Explicit terminal capture and export | Share evidence and let agents inspect designated output | M | With control API |
| 10 | Contextual path picker and directory-aware launches | Reduce path typing and wrong-directory mistakes | M | Independent improvement |
| 11 | Searchable settings and effective-shortcut explanations | Make existing capability discoverable | M | Incremental |
| 12 | Resource budgets for hidden/background terminals | Keep large workspaces responsive and quiet | Measurement first; potentially L | Profile before changing architecture |
| 13 | Session/backend diagnostics and clear update state | Make recovery safer and failures easier to explain | M | Incremental |
| 14 | Optional explicit program-status input | Show progress, blocked reasons and child work | L/backend-dependent | Investigate upstream first |
| 15 | Discoverable plugin commands | Make project tools callable from palette and automation | M–L | After small host API succeeds |

## 1. Name the task, keep the process identity

**Rex evidence:** UI title preferences and optional agent session naming; API
labels for sessions, windows and blocks; Runtime explicit block rename.

Chartr's current
[`Session::title`](https://github.com/rengwu/chartr/blob/369298062bfd5ba00ad0c273d580ce408797b19c/crates/chartr-herdr/src/control.rs#L148)
prefers the running agent/process over the saved label. The
[rename flow](https://github.com/rengwu/chartr/blob/369298062bfd5ba00ad0c273d580ce408797b19c/crates/chartr/src/app/rename.rs)
targets spaces and groups. Several unrelated tasks therefore collapse to the
same visible provider name even though conversation-title data already exists.

Add an explicit name on terminal identity, with this precedence: **manual task
name → usable conversation title → current process/shell fallback**. Keep the
provider icon and reveal cwd/process in secondary detail. Include “Reset to
automatic.” Automatic updates must never overwrite a manual name.

Example: three `Codex` rows become `Fix OAuth retry`, `Review migration` and
`Add billing tests`, with the same provider glyph. This creates useful input for
search, notifications, recipes and activity summaries without a model call.
Agent-supplied names should be optional and must use the same override policy.

Acceptance: rename while an agent runs; restart/detach/handoff; verify identity
and name survive and changing process titles does not erase the override.

## 2. Turn the palette into a workspace switcher

**Rex evidence:** UI palette searches commands, sessions and tabs; API target
resolution, action keywords and session selection.

Chartr's
[palette](https://github.com/rengwu/chartr/blob/369298062bfd5ba00ad0c273d580ce408797b19c/crates/chartr/src/app/command_palette.rs)
filters a fixed command list by lowercased label substring. It does not search
live space/terminal identities. Improve this existing surface before adding
another navigation mode.

Offer two clear result types: actions and live destinations. Match terminal name,
space, group, provider and cwd; show enough parent context to distinguish names.
Opening a result must focus the exact pane and selected inner tab, including a
collapsed space. Preserve manual sidebar order; rank search results separately.

Start with predictable token/prefix matching and recency tie-breaking. A complex
semantic-search service is unnecessary. Keep current Tabs/Spaces behavior and
its Settings-only switcher decision intact. Never auto-select an ambiguous target
for an agent simply because it is the first fuzzy match.

Acceptance: duplicate task names in two projects, inactive inner tabs, collapsed
spaces, a target closing during search, and focus over embedded plugin surfaces.

## 3. Add an activity overview, not another chat interface

**Rex evidence:** UI per-status presentation preferences and Activity action;
Binary activity projections/seen ledger; Runtime nested status behavior. The
Activity overview itself was not successfully exercised through native automation.

Chartr already has an
[attention model](https://github.com/rengwu/chartr/blob/369298062bfd5ba00ad0c273d580ce408797b19c/crates/chartr/src/attention.rs)
and exact-terminal navigation. Expose that data in a compact popover or filter:
**Needs review**, **Blocked**, and **Working**, with task name, space and observed
state age. Add space-level unread counts and “Next unread terminal.”

Unread and blocked are different facts. Completion, cancellation or a brief
inactive transition may produce unread attention under the accepted best-effort
policy. Call that count “unread” or “to review,” not “awaiting approval.” Do not
show an inferred idle state as a green success check. Clicking one result should
acknowledge only its exact terminal under the existing foreground-focus rule.

Keep unread through resumed work. Preserve the one-banner-until-visit rule.
Add notification preferences gradually, beginning with a global switch and
optional per-space muting; Rex's full state-by-channel matrix need not be copied.
This extends the accepted lean design and does not revive the disabled Chats UI.

## 4. Give agents a small, reliable Chartr control interface

**Rex evidence:** API introspection, stable IDs, client/server distinction and
focus options; Runtime capture, layout operations and exact exit status. Its
[API documentation](https://www.superlogical.com/rex/docs/automate/server-api)
explains why capabilities can be discovered against the connected version.

Chartr's executable currently accepts a folder path; the private Herdr wrapper
has a backend API, but that is not an API for Chartr's spaces, groups, plugins or
focus. A script manipulating Herdr directly could create sessions that Chartr's
workspace model does not own correctly.

Start with a deliberately small local interface: inspect workspace, resolve
identity, create a terminal with explicit cwd/command, focus a target, capture a
chosen terminal, and wait for a launched job. Return JSON, stable IDs, clear
errors and the actual selected target. Creation should support preserving the
user's focus. Destructive operations must have an explicit target.

Route mutations through Chartr's existing workspace ownership and action code.
Do not create a second layout manager or expose the private Herdr socket as the
public contract. Define what works when the GUI is closed; return a clear
unavailable result for UI-owned operations until headless ownership is designed.

Expose invoking terminal/space context to launched agents and ship a guide from
the installed version. Distinguish “beside this agent” from “the pane the user is
currently viewing.” Keep access local and permissioned; plugin permissions and
project scope must still apply. Identity-bearing responses and duplicate-safe
launch handling matter more than a large method catalog.

This is the largest broadly useful foundation in the list: it also improves
verification tooling, reproducible setup and future plugin automation.

## 5. Build workspace recipes around Chartr's existing tools

**Rex evidence:** API nested layouts; Runtime focus-preserving splits and rollback.
**Chartr proposal:** a recipe system using its richer surfaces and services.

Let a project save a small recipe such as “Implement,” “Review” or “Debug.” A
recipe describes pane arrangement, role names, working directories, approved
commands and optional plugin surfaces. It is different from session restoration:
restoration reconnects to existing identities; a recipe creates fresh work.

For example, an Implement recipe could open an Agent terminal, a narrow test
pane, and Wayfinder or Markdown Prompt beside them. Review could open a diff
command and an agent prepared with a selected saved prompt. Reference saved
prompt/template IDs and configured agent providers through existing services,
rather than duplicating provider configuration inside every recipe.

Preview missing prerequisites, commands and plugin permissions before starting.
Never execute a repository recipe merely because its folder was opened. Validate
all available prerequisites first; if launch partially fails, retain an accurate
receipt of created IDs and clean up only resources that attempt created.
Use existing Wayfinder claim rules; a recipe must not bypass them.

This is a stronger Chartr differentiator than a general Lua runtime: it combines
its existing project tools into a repeatable workflow.

## 6. Zoom a pane without dismantling the workspace

**Rex evidence:** UI zoom controls; API zoom and percentage resize; Runtime
zoom/unzoom preserving the underlying split and block identities.

Chartr already supports drag/drop, directional movement/focus, divider resizing
and same-axis rebalancing on insertion/removal. Recommending those as entirely
new would be inaccurate. The missing product slice is temporary **pane zoom**
and convenient keyboard resize controls.

Zoom should change presentation while preserving split geometry, selected inner
tabs, plugin ownership and terminal identity. Restore the exact arrangement when
unzooming. An unread child hidden by zoom must remain unread. Define what happens
when the zoomed pane closes or a new pane is added while zoomed.

Add repeatable directional resize actions with modest increments. Consider a
visible temporary resize mode later; keep its active state obvious and Escape
reliable so ordinary terminal arrows are never unexpectedly intercepted.
The public [key-mode guide](https://www.superlogical.com/rex/docs/customize/key-modes)
illustrates this pattern, but a complete scripting/key-mode system is not needed
for the first version.

## 7. Let users set work aside without closing it

**Rex evidence:** Runtime/API detaching a block preserves its process. This is
mechanical support, not evidence of a complete user-facing snooze workflow.

Add a reversible “Set aside” state or tucked section for long-lived terminals.
It should reduce clutter, remain searchable, and preserve the terminal's
backend identity. Chartr already detaches on normal app exit; this proposal is
per-item organization inside a still-running workspace.

Store user intent separately from activity and unread. Recommended initial rule:
set-aside hides the row from the main list but keeps unread visible in the
overview; muting notifications is a separate explicit control. Do not infer that
a program has stopped or suspend its actual process. Avoid automatic wake-up
based on unreliable prompt detection.

Scope plugin support explicitly. A terminal can reconnect after dropping its
local attachment, while an arbitrary embedded plugin may need a live view or a
restorable state. Start with terminals if necessary. The earlier
[Harp discussion](harp-adoption.md#set-sessions-aside) already identifies the
notification and resume decisions that this feature needs.

## 8. Treat tests and builds as named jobs with known exits

**Rex evidence:** Runtime `run --wait` returns the child exit code, explicit
keep-open preserves output, and a quick failing command remains inspectable
under the default close policy.

Add a small “Run task” workflow for commands Chartr itself launches: named job,
explicit cwd, running state, elapsed time, exact exit code, retained output and
rerun. A failed build should not disappear with its terminal. A useful first
version can be an ordinary named terminal with a result badge and rerun action.

Chartr's
[session endpoint](https://github.com/rengwu/chartr/blob/369298062bfd5ba00ad0c273d580ce408797b19c/crates/chartr/src/session.rs)
currently observes the local attach client ending. That is not automatically the
exit of the server-owned command. Obtain the actual launched job's result through
a supported backend capability or an explicit launch wrapper with stable job
identity. Do not parse a TUI's final text to manufacture success.

Keep deterministic command outcomes separate from best-effort agent activity.
A stopped/cancelled agent must not be reported as a successful task merely
because it is inactive. Recipes and automation can then rely on trustworthy
build results without changing every provider detector.

## 9. Add explicit capture, copy-output and export

**Rex evidence:** Runtime plain/HTML capture and complete chunk reassembly; API
supports VT, scrollback, alternate-screen modes and bounded blob reads.

Offer “Copy terminal output” and “Export output,” plus a narrowly scoped capture
operation for agents. These would help share build failures, review agent work
and preserve a terminal's evidence before closing it. Begin with plain text and
selection-aware copy; add styled HTML only when useful.

Define whether the operation returns the current screen, scrollback, or a known
job's output. An alternate-screen TUI is a current visual state, not a complete
conversation transcript. Return truncation/freshness metadata for bounded
captures; never silently imply that the first chunk is the whole result.

This does not change the accepted notification-preview policy. That policy uses
supported provider logs and a generic fallback; explicit user/agent capture is
a different operation. A captured screen must not become an implicit source of
outcome classifications or privileged instructions.

## 10. Add a small contextual path browser

**Rex evidence:** UI path picker; API cwd-aware directory listing with typed errors;
Runtime file names with spaces and missing-directory responses.

Provide a keyboard picker that inserts a selected file/folder path into the
current input, and a separate “New terminal here” action. Default to the actual
terminal cwd when reliably known, otherwise show the owning space folder as the
fallback. That distinction prevents commands from opening in a surprising
project root after the user has navigated elsewhere.

Use appropriate shell escaping and insert text without automatically pressing
Enter. Keep folder navigation responsive with bounded listings and filtering;
show permission/missing-directory errors rather than treating them as empty
results. A small recent-path list can be project-scoped.

Chartr's project ownership and plugin file-access rules remain separate from
terminal cwd. A user changing directory in a terminal must not silently expand
a plugin's permissions. If remote terminals are ever added, local and remote
paths must carry host context instead of being treated as interchangeable.

## 11. Make settings and shortcuts explain themselves

**Rex evidence:** UI settings search, visual previews, shortcut search by chord,
winning-binding explanation and action keywords/argument schemas.

Chartr already has themes, light/dark choices, reduced motion and editable
hotkeys with conflict checks. Improve discovery rather than expanding the
settings count. Add search across host and enabled-plugin settings with direct
navigation to the matching control. Keep plugin contributions lazy.

Show effective shortcuts in the palette, including user overrides, rather than
independently maintained default text. Let a user search a chord and see the
action/context that wins. Offer a direct path from an action to its keybinding.
Existing semantic actions and keymap metadata are the starting point, not a
reason to replace GPUI's binding system.

For appearance, prioritize a compact terminal preview showing normal, selected,
muted and ANSI-colored text. Contrast, legible inactive panes and clear focus
provide more daily benefit than copying glass materials or adding 41 themes.
No automatic recoloring should erase semantic colors in logs without an explicit
choice.

## 12. Budget resources for large, mostly inactive workspaces

**Rex evidence:** Binary idle timers, parking, snapshot suspension/restoration,
compression, bounded queues and slow-client handling; scratch snapshot files.
There is no comparative performance benchmark in this investigation.

Chartr keeps Herdr-backed terminals through a local attach PTY and the Zed
terminal model. Profile that complete chain under realistic workloads: 1, 10 and
50 terminals; idle, noisy background output and a foreground TUI; AC and battery;
Low Power Mode; hidden/minimized windows. Record input latency, wakeups, CPU,
resident memory, allocations and reattach cost.

Only then choose targeted work: avoid redundant polls/repaints, stop unnecessary
hidden-view work, bound captures/queues, and investigate releasing cold frontend
state while Herdr continues owning the PTY. Any cold-state policy must retain
scrollback as promised, unread identity, input continuity and fast restoration.

Chartr already has a specific supplied-drawable Low Power Mode fix. Do not undo
it or change terminal engines based on symbol names. “Rex contains a suspension
mechanism” is a useful engineering lead; “Rex is faster” is unproven.

## 13. Explain backend health, recovery and update state

**Rex evidence:** UI server uptime/details and diagnostics; Binary lifecycle and
compatibility machinery; API running version and idle-only stop capability.

Build on Chartr's existing Problems menu and retry/crash-loop behavior. A concise
session details panel could show frontend build, bundled/running backend version,
connection state, exact terminal identity, cwd, attachment status and why recovery
is available. Add an explicit sanitized diagnostic export, excluding terminal
contents and sensitive arguments unless separately selected.

Distinguish “the app updated,” “the backend is still running an older version,”
and “a restart/handoff is required.” Do not put an ambiguous Restart button next
to live work. Chartr already pins a patched Herdr and preserves terminal identity
through handoff; surface that actual contract rather than substituting Rex's
version policy.

This is particularly valuable given the recorded history of a shared-profile
test disrupting sessions. Development diagnostics should also make the active
profile/namespace and package identity unmistakable. A diagnostic operation
must not silently restart a backend just to make version numbers match.

## 14. Investigate explicit status as an optional backend capability

**Rex evidence:** Runtime multi-record status behavior and Binary storage/cleanup.
The public [OSC 7501 specification](https://www.superlogical.com/rex/docs/build/program-status)
is a candidate interoperable input, not a reason to build Rex-specific adapters.

Progress percentages, blocked reasons and child-task labels could improve
Chartr's activity overview considerably. They should arrive as structured reports
from cooperating programs and remain distinct from heuristic observations.
Unknown data must remain unknown. Displayed task messages are untrusted plain
text, not instructions or proof that work succeeded.

Investigate support in Herdr/upstream first. Chartr's accepted
[attention simplification](../decisions/index.md#dec-attention-005-reuse-existing-detection-with-best-effort-alerts)
explicitly avoids a custom provider/outcome stack. Do not recreate that rejected
complexity or assume private escape sequences pass intact through the existing
Herdr-attach/Zed chain. Establish where parsing lives and which supported API
carries the result before promising UI behavior.

An optional first implementation could enrich details/progress while leaving
notification triggers unchanged. Broader state semantics, child-task counts and
new alerts require a separate accepted design and real-provider verification.
The first three recommendations do not depend on this investigation.

## 15. Expose plugin capabilities through commands

**Rex evidence:** API block creators advertise flavors, methods, events and actions;
Binary executable-plugin and reload infrastructure. Only terminal/onboarding
creators were advertised in the inspected default instance.

Chartr already has a useful advantage: scoped plugin surfaces, typed catalog
services, permissions and project tools. Extend those with optional command
metadata—stable ID, label, arguments, availability and permission requirements—
so a plugin can expose a few meaningful operations in the palette and later CLI.

Examples might be opening a particular Wayfinder map, preparing a Markdown Prompt
from a saved template, or inspecting a plugin's current status. Preserve the
same validation/claim checks as the UI and invalidate commands immediately when
a plugin is disabled. A plugin command should not become a route around project
file permissions or provider availability.

Start with one existing bundled plugin and a small command contract. A universal
remote block protocol, executable plugin host or a rewrite of the current native
ABI is unnecessary until concrete needs justify it.

## What to keep, and what to defer

| Rex area | Chartr judgment |
| --- | --- |
| Persistent server-owned sessions | Already a Chartr strength. Improve discoverability/recovery, not duplicate it. |
| Tab/pane movement and automatic balancing | Much already exists. Add zoom and keyboard control where the product surface is missing. |
| Native visual refinement | Borrow hierarchy, previews and focus cues; preserve Chartr's established styling and shared UI components. |
| Remote/Tailscale/multi-client sessions | A real larger opportunity for workstation-to-laptop/phone continuity, but a separate product direction. Requires auth, host identity, ownership, versioning and platform work. Do not expose Chartr's private Herdr namespace as a shortcut. |
| Floating terminal layers | Runtime support was confirmed in Rex, but Chartr's grouped splits plus zoom should solve the common cases first. Floating layers add occlusion, focus and embedded-view complexity. |
| Full Lua configuration/runtime | Powerful, but a typed command surface and declarative recipes provide a smaller initial commitment. Existing plugin capabilities should be reused. |
| New rich-chat interface | Not supported by this evidence as a priority; Chartr deliberately disabled Chats. Activity triage is a different problem. |
| Universal status correctness | Not established for Rex or Chartr. Keep confidence/provenance and deterministic exits separate from heuristics. |
| Replacing Zed/Herdr with Ghostty/Rex | No evidence that this is necessary for the recommended features; substantial compatibility and distribution cost. |
| Copying implementation or theme assets | Not part of these proposals. Reimplement selected behavior using Chartr's existing architecture. |

## Suggested sequence and decision points

**First delivery:** terminal names, search across live workspace destinations,
space unread counts, next-unread navigation and a compact activity list. This
largely reuses existing state and reinforces the earlier Harp analysis. Pane
zoom can be implemented alongside it as an independent slice.

**Second delivery:** a small local control interface, precise terminal capture and
owned-job completion. Use it internally for repeatable verification before
expanding the public surface. Keep identity, focus preservation and meaningful
errors in the first version.

**Third delivery:** workspace recipes and one plugin command contribution.
These are the features most likely to make Chartr feel more capable than a
terminal multiplexer with better decoration.

Profile resource use throughout; optimize demonstrated bottlenecks. Investigate
explicit status upstream separately. Remote access remains a distinct strategic
choice rather than a hidden dependency of local improvements.

Open product decisions, only if these proposals are selected: set-aside/muting
semantics; initial control-API permissions and closed-GUI behavior; recipe scope
and command review; backend source of exact job results; and whether remote
work is part of Chartr's intended audience. No decision was accepted by this
analysis, and no current feature expectations were rewritten.
