# Chartr workspace rewrite specification

## Problem Statement

Chartr's current interface is visually inconsistent and structurally fragile.
Controls and tab chrome are misaligned, reusable Zed components and semantic theme
tokens are not applied consistently, and some item types—including plugin tabs—do
not expose expected controls such as close buttons.

The underlying ownership model is also incorrect. Ad-hoc sessions and opened
plugin views can appear in every space even though an opened tab must belong to
exactly one space and one pane. The existing flat active-item model cannot support
Zed-style nested panes, cross-pane tab drag and drop, directional focus, resizing,
or reliable restoration.

Core application behavior is incomplete: closing shortcuts are missing; settings
do not exist; web plugins are advertised but not hosted; workspace state is not
fully persisted; and Herdr transport failures can surface as an unhandled broken
pipe instead of the proven recovery behavior from Chartr-rs.

## Solution

Build Chartr around a focused implementation of Zed's multi-workspace model. The
application window owns multiple independent spaces. Each space owns an ordered
outer tab collection, and each outer tab owns one recursive Zed-style pane
workspace. A one-item workspace is presented as a standalone tab; a workspace
with multiple items or panes is presented as one grouped tab whose panes
exclusively own their ordered items. A catalog may advertise plugin factories
globally, but an opened plugin instance belongs to exactly one outer tab, pane,
and space.

Provide complete Zed-style pane behavior: nested splits, divider resizing,
directional focus, tab reordering and movement between panes, edge-drop splitting,
joining, zooming/maximizing, contextual commands, a command palette, and complete
layout restoration. Terminals are non-cloneable; plugins may explicitly declare
clone support. Cross-space movement is not supported.

Offer tabbed and sidebar projections over the same model. Both show every
standalone item and every pane group as one outer entry. Standalone terminals
use Herdr's live agent or foreground-process inference before falling back to
the persistent Herdr tab label; pane groups use the neutral `Grouped Tabs`
title. Tabbed mode places that collection beside the active space name;
sidebar mode places it beneath each visible space. Selecting a group renders its
local draggable pane tab bars, while selecting a standalone item renders no
redundant inner bar. Only the active pane exposes compact split/zoom controls.
Presentation never changes item ownership.

In the all-spaces sidebar, space headings directly sort their complete cards.
Sorting uses measured variable-height midpoints, remains active during horizontal
overdrag, resolves the final slot from release Y, and autoscrolls at the vertical
edges. A short interruptible FLIP transition settles displaced cards unless the
user enables Reduce Motion.

Use Zed's existing GPUI, UI, and theme crates and their components, semantic
colors, spacing, typography, focus, accessibility, menu, modal, notification,
and drag-and-drop conventions. Chartr owns product composition, not replacement
UI primitives. Go Chartr supplies the visual reference through normal semantic
`Chartr Light` and `Chartr Dark` themes; `Chartr Dark` is the fixed default.

Present Settings inside the main window as Chartr-rs does, while implementing a
focused Zed-shaped typed settings store, page catalog, field renderer registry,
semantic action/keymap system, atomic persistence, live updates, and plugin page
contributions. Settings are user-global in this version.

Restore Chartr-rs's small, explicit Herdr lifecycle: fresh control connections,
per-session stream failure states, one clean backend restart, a crash-loop guard,
and a non-destructive Retry action. Avoid a generic supervisor or backend
administration surface.

Persist application-owned layout and item metadata in a versioned SQLite store,
while user-editable settings, keymaps, and themes remain files. Keep all data
isolated under the `chartr-zeddy` namespace and do not import existing Chartr
configuration automatically.

## User Stories

1. As a Chartr user, I want every open tab to belong to one space, so that switching spaces never duplicates sessions or plugin views.
2. As a Chartr user, I want every tab to belong to one pane, so that its position and focus are unambiguous.
3. As a Chartr user, I want one permanent Ad-hoc space, so that I can open a terminal without first selecting a project folder.
4. As a Chartr user, I want Ad-hoc terminals to start in my home directory by default, so that folderless sessions have a predictable working directory.
5. As a Chartr user, I want to configure the Ad-hoc working directory, so that folderless sessions suit my workflow.
6. As a Chartr user, I want at most one space per canonical folder, so that aliases and symlinks do not create duplicate projects.
7. As a Chartr user, I want to rename a space's displayed label without changing its folder identity, so that my workspace list is understandable.
8. As a Chartr user, I want missing folders retained as unavailable spaces, so that transient mounts or moved folders do not destroy layout state.
9. As a Chartr user, I want to locate a missing space folder, so that I can reconnect its saved workspace state.
10. As a Chartr user, I want removing a space to leave its folder untouched, so that workspace cleanup cannot delete project data.
11. As a Chartr user, I want new sessions to open as standalone outer tabs in the targeted space, so that they do not silently join an unrelated pane group.
12. As a Chartr user, I want an inactive space's add control to activate that space before creating its session, so that sessions never enter the wrong owner.
13. As a Chartr user, I want nested horizontal and vertical splits, so that I can arrange several terminals and tools at once.
14. As a Chartr user, I want to resize split dividers, so that each pane receives useful screen space.
15. As a Chartr user, I want directional pane focus, so that I can navigate a split layout from the keyboard.
16. As a Chartr user, I want to reorder tabs within a pane, so that related work stays together.
17. As a Chartr user, I want to drag a tab between panes, so that I can reorganize the current space without recreating its item.
18. As a Chartr user, I want to drop a tab on a pane edge to create a split, so that advanced layouts are direct and discoverable.
19. As a Chartr user, I want joining a pane to move its items into an adjacent pane, so that changing layout never kills work.
20. As a Chartr user, I want a split pane removed when its last item leaves, following Zed's default pane lifecycle, so that empty implementation structure does not accumulate in the UI.
21. As a Chartr user, I want an emptied outer tab removed while the space remains usable through its New action, so that phantom groups do not accumulate.
22. As a Chartr user, I want to zoom or maximize a pane, so that I can temporarily concentrate on one item.
23. As a Chartr user, I want terminals never to be cloned or mirrored, so that one session is never represented by multiple terminal tabs.
24. As a plugin author, I want to declare whether my item supports cloning, so that split cloning is safe and intentional.
25. As a Chartr user, I want pane layouts and split ratios restored after switching spaces, so that every space behaves like an independent editor window.
26. As a Chartr user, I want pane layouts and active items restored after relaunch, so that restarting Chartr does not destroy organization.
27. As a Chartr user, I want tabbed mode to show only the active space, so that its compact chrome remains focused.
28. As a Chartr user, I want every non-empty workspace pane in either presentation mode to retain its own draggable tab bar, so that tab ownership and movement remain visible like Zed.
29. As a Chartr user, I want sidebar mode to show either all spaces or only the active space, so that I can choose overview or focus.
30. As a Chartr user, I want All Spaces to be the initial sidebar mode, so that a fresh installation exposes the whole cockpit.
31. As a Chartr user, I want standalone tabs and any number of pane groups mixed in one space, with each group collapsed to one outer entry titled `Grouped Tabs`, so that unrelated sessions remain independent without implying one child represents the group.
32. As a Chartr user, I want only the active non-empty pane to expose compact Zed-style split and zoom controls while all pane tab bars remain visible, so that advanced operations remain available without hiding the pane structure.
33. As a Chartr user, I want selecting an item in an inactive space to activate its space, pane, and item together, so that selection is one coherent action.
34. As a Chartr user, I want the sidebar width and presentation modes persisted, so that the application retains my preferred chrome.
35. As a Chartr user, I want each top-level pane group to be closable, so that I can end everything beneath that group deliberately without closing its sibling tabs or groups.
36. As a Chartr user, I want confirmation before an operation kills multiple sessions, so that bulk actions are not accidentally destructive.
37. As a Chartr user, I want closing one terminal tab to terminate its Herdr session immediately, so that abandoned processes do not accumulate.
38. As a Chartr user, I want `Cmd+W` on macOS and `Ctrl+W` on Linux to close the active tab, so that closing follows familiar application behavior.
39. As a Chartr user, I want a closed session's explicitly bound plugin items to close too, so that dependent tools never outlive their subject.
40. As a Chartr user, I want closing a plugin tab to destroy that view instance, so that it no longer consumes active UI state.
41. As a Chartr user, I want normal application exit to detach sessions, so that quitting the UI does not terminate intentional long-running work.
42. As a Chartr user, I want a setting that can terminate sessions on application exit, so that I may choose stricter cleanup.
43. As a Chartr user, I want removing a space to terminate everything it owns after confirmation, so that the ownership boundary has clear lifecycle semantics.
44. As a Chartr user, I want closing Settings with `Cmd/Ctrl+W` to return to my previous item, so that a hidden terminal is never killed accidentally.
45. As a Chartr user, I want plugin contributions to be globally discoverable but opened instances to remain space-owned, so that catalogs do not duplicate live tabs.
46. As a Chartr user, I want separate plugin instances in different spaces when supported, so that each project can have independent tools.
47. As a plugin author, I want to declare singleton or multi-instance behavior, so that Chartr enforces my contribution's valid lifecycle.
48. As a Chartr user, I want a per-space singleton plugin to focus its existing pane when reopened, so that it is not silently moved or duplicated.
49. As a plugin author, I want a stable owning-space context, so that my view never retargets when another space becomes active.
50. As a plugin author, I want to bind explicitly to one session when required, so that session-sensitive behavior is deterministic.
51. As a Chartr user, I want restorable plugin items to return after relaunch, so that supported tools participate in workspace persistence.
52. As a Chartr user, I want unrestorable plugin items omitted with a summary, so that missing plugins do not create permanent broken tabs.
53. As a plugin author, I want to contribute a lazy settings page, so that configuration appears only when my plugin provides it.
54. As a Chartr user, I want native plugins labeled as fully trusted code, so that their security model is honest.
55. As a Chartr user, I want web plugin permissions visible before enablement, so that I understand their authority.
56. As a web plugin author, I want declared read/write access within my owning project's folder, so that useful project tools are possible in safe mode.
57. As a web plugin author, I want plugin-specific data storage, so that my plugin can persist data without broad filesystem access.
58. As a Chartr user, I want folderless-space web plugins restricted to plugin data in safe mode, so that `$HOME` is not implicitly exposed.
59. As a Chartr user, I want to grant unrestricted filesystem access to one web plugin through unsafe mode, so that capable plugins remain possible without weakening every plugin.
60. As a Chartr user, I want no global unsafe switch, so that one grant cannot silently authorize unrelated plugins.
61. As a web plugin author, I want declared host actions for network and process access, so that powerful behavior is mediated and visible.
62. As a session-bound web plugin, I want declared access to metadata and terminal input for only my bound session, so that session integrations remain scoped.
63. As a Chartr user, I want permission revocation to close live plugin instances and revoke their broker, so that reduced authority takes effect immediately.
64. As a Chartr user, I want native and web plugin panes both to work, so that no advertised plugin tier ends in a placeholder.
65. As a Chartr user, I want disabling a plugin to remove its contributions and prevent future loading, so that enablement has real effect.
66. As a Chartr user, I want Settings presented inside the main window while retaining the spaces sidebar, so that configuration feels native to Chartr-rs.
67. As a Chartr user, I want General, Appearance, Terminal, Hotkeys, and Plugins settings pages, so that the implemented product can be configured coherently.
68. As a Chartr user, I want Settings to show only implemented controls, so that no option is decorative or misleading.
69. As a Chartr user, I want settings changes applied immediately where safe, so that configuration provides direct feedback.
70. As a Chartr user, I want settings updates written atomically, so that a crash cannot corrupt preferences.
71. As a Chartr user, I want keyboard shortcuts editable in the ordinary Hotkeys page, so that customization does not require manual file editing.
72. As a Chartr user, I want contextual shortcut conflict detection, so that terminal input and workspace actions resolve predictably.
73. As a Chartr user, I want a command palette exposing workspace and pane actions, so that advanced operations are discoverable.
74. As a Chartr user, I want `Chartr Dark` as the fixed initial theme, so that the application starts with the intended identity.
75. As a Chartr user, I want `Chartr Light`, fixed theme, and light/dark/system theme-pair options, so that I can change appearance later.
76. As a Chartr user, I want user theme files loaded and refreshed, so that Chartr remains compatible with the intended Zed-style theme model.
77. As a Chartr user, I want IBM Plex Sans and IBM Plex Mono as configurable defaults, so that the Go Chartr visual reference is preserved without locking my typography.
78. As a Chartr user, I want semantic theme colors and Zed UI components everywhere, so that alternate themes remain coherent.
79. As a keyboard user, I want every drag operation to have an action-based alternative, so that pane management is not pointer-only.
80. As an accessibility user, I want reliable focus order, focus restoration, labels, contrast, and reduced-motion behavior, so that the application is operable without visual guesswork.
81. As a Chartr user, I want an affected terminal to show a clear state when its Herdr stream breaks, so that a transport failure is understandable.
82. As a Chartr user, I want reattachment offered only when Herdr confirms the same session exists, so that retry cannot silently create or target the wrong session.
83. As a Chartr user, I want Chartr to restart its private Herdr once after unexpected death, so that a transient backend crash recovers automatically.
84. As a Chartr user, I want repeated backend death to become a stable crash-loop state with Retry, so that Chartr does not restart forever.
85. As a Chartr user, I want surviving space layouts and space-bound plugins retained after backend loss, so that one backend crash does not erase unrelated workspace state.
86. As a Chartr user, I want Herdr's live session list to override stale local terminal records, so that the UI reflects processes that actually exist.
87. As a Chartr user, I want orphaned live Herdr sessions adopted into their owning space, so that detached work is not lost from the UI.
88. As a Chartr user, I want a fresh installation to open the empty Ad-hoc space without spawning a terminal, so that startup has no unnecessary process side effect.
89. As a Chartr user, I want window geometry, pane ratios, expansion state, selection, and chrome restored, so that the entire cockpit returns after relaunch.
90. As an existing Chartr user, I want Chartr-zeddy data isolated from older installations, so that the rewrite cannot corrupt or conflict with existing settings.
91. As a Chartr user, I want to drag-sort every sidebar space, including Free sessions and recovered folders, so that the cockpit order matches my workflow and survives relaunch.
92. As an accessibility user, I want Reduce Motion to disable space-sort settling without disabling direct manipulation, so that reordering remains usable with less animation.
93. As a Chartr user, I want wheel and trackpad gestures to move through Herdr's host scrollback, so that output remains reviewable after it leaves the live viewport.

## Implementation Decisions

- The user-facing application is Chartr. Configuration, state, plugins, and the
  private Herdr runtime use an isolated `chartr-zeddy` namespace for now.
- The application window follows Zed's `MultiWorkspace` responsibility and owns
  ordered space entities plus one active space.
- A space is the lifecycle and persistence boundary analogous to a Zed
  `Workspace`. It owns an ordered, activation-tracked collection of outer
  workspace tabs plus its folder identity and restoration state. Each outer tab
  owns one existing recursive pane workspace; it is standalone when it has one
  item and one pane, and grouped when it has multiple items or panes.
- A pane exclusively owns its ordered items, active item, activation history,
  focus state, and drag state. Chrome never owns or reconstructs item state.
- A pane group is a recursive axis tree with horizontal/vertical members and
  persisted flex ratios. Workspace-level event handling coordinates mutations.
- Items expose lifecycle, serialization, focus, close, and optional clone
  behavior. A terminal session item is non-cloneable and closes destructively.
- A standalone terminal title is recomputed from Herdr on the two-second backend
  refresh: display agent, internal agent, non-shell foreground process, then
  persistent tab label/number. Exiting a process restores the fallback rather
  than leaving a stale locally remembered title.
- Terminal wheel deltas are accumulated in row units. The first upward gesture
  loads ANSI-styled `pane.read` host history on a background thread and moves a
  separate historical VT viewport; live repaint frames remain isolated from
  history so they cannot manufacture duplicate or missing rows. New live output
  marks a bottomed history snapshot for refresh, and resizing invalidates it.
- An opened item entity may appear in only one outer workspace tab, pane, and
  space. Moving an item removes it from its source before insertion; an emptied
  outer tab disappears. Cross-space moves are absent.
- Pane mutations use typed actions and pane events. Product chrome does not reach
  into pane internals to mutate vectors directly.
- Dragged tabs carry their source outer tab, pane, source index, and item
  identity, and use the same tab component for their drag preview. Standalone
  outer entries may be dragged directly into any pane of the selected group.
  Drops on pane tabs use Zed's
  source-aware before/after insertion rule; the trailing tab-strip target
  appends; pane-body center drops move into the target pane; and pane-body edge
  drops split it. Modifier cloning is available only to plugin items that
  declare it, with non-cloneable items falling back to an ordinary move.
- Pane split hit-testing exists only over pane content, never over its tab bar.
  Its edge band is 20% of the shorter pane dimension, corners resolve to the
  nearest edge, and the remainder is the center target. The transient highlight
  fills the content for center drops and the relevant half for edge drops. Tab
  and trailing-strip targets clear split intent; `Escape` cancels the drag and
  clears any transient target.
- A web-plugin child view reports pointer focus through the private host bridge
  so its owning item and pane become active just like native GPUI content.
  Native child webviews are hidden only for the duration of a GPUI drag so the
  dragged tab and pane drop highlight remain visible above their pixels.
- Joining a pane moves items and collapses the axis. Moving or closing the last
  item collapses a non-root pane; an outer workspace tab disappears once no
  items remain anywhere beneath it. As in Zed, invoking split-and-move on a
  pane with only one item instead inserts an empty pane on the opposite side
  and keeps the item focused, so the requested split is visible rather than
  being immediately collapsed by the ordinary empty-source rule.
- A visual outer group is not an item. Its close control is a bulk lifecycle
  action over only that outer tab's descendant items; panes expose their own
  Close All action, and closing the containing space remains the larger boundary.
- Single destructive item closes do not confirm. Any action that would terminate
  multiple live sessions confirms with an exact count.
- The active item after removal follows Zed's activation-history behavior with a
  positional fallback.
- The permanent Ad-hoc space has no folder, cannot be renamed or removed, and
  defaults new sessions to the user's home directory or a configured replacement.
- Folder spaces are deduplicated by canonical path. Display names are metadata and
  do not participate in identity.
- Tabbed and sidebar modes are alternate renderings of the same outer-tab/pane/
  item state. Changing chrome never creates, moves, or closes an item.
- Each standalone item and pane group projects to one entry in both chromes.
  Tabbed mode keeps these entries on the space-name row. Closing a group entry
  closes every item in only that group through the normal bulk confirmation.
- Sidebar mode persists an All Spaces or Active Space submode. Selecting an item
  from another space activates its space, pane, and item as one operation.
- The sidebar is resizable with bounded width. Tabbed mode is active-space-only.
  All-Spaces card sorting is a window-owned, space-specific interaction: the
  complete card carries only on Y, live order changes at measured card
  midpoints, tracked-scroll edge autoscroll follows Zed's curve, and release
  outside the sidebar resolves the closest legal Y slot. Displaced cards use an
  interruptible fixed 150 ms quintic FLIP unless Reduce Motion is enabled.
- User-visible actions are semantic GPUI actions with contextual keybindings.
  Platform defaults follow Zed except that terminal focus does not override the
  requested `Cmd/Ctrl+W` close behavior.
- Settings is a main-window workspace that preserves the spaces sidebar and
  restores the prior focus on close; it is not an item in a pane.
- Settings serialization uses sparse optional content; runtime consumers use
  resolved typed settings with complete defaults.
- One centralized settings store merges defaults and user-global configuration,
  observes changes, performs atomic writes, and refreshes affected windows.
- Settings page data is declarative. Field renderers own reusable controls, while
  domain code owns resolved settings behavior.
- Hotkeys are presented in Settings but backed by a contextual keymap model. The
  command palette, menus, buttons, and shortcuts dispatch the same actions.
- The settings catalog contains General, Appearance, Terminal, Hotkeys, and
  Plugins. Controls are omitted until their behavior exists.
- Zed's existing UI components and semantic styles are audited before any local
  reusable component is introduced. Chartr may compose product-specific views.
- `Chartr Light` and `Chartr Dark` are standard semantic theme families. Chartr
  Dark is the fixed default; users may select fixed or light/dark/system themes.
- Theme colors are resolved at render time. Feature views do not cache palettes or
  embed Go Chartr color literals.
- IBM Plex Sans and Mono are bundled defaults and remain user settings. Application
  controls use Zed's icon components directly; no alternate icon framework exists.
- Default density is the only exposed density initially, though semantic dynamic
  spacing remains compatible with future density settings.
- The plugin catalog stores descriptors and factories, never live item instances.
- Plugin multiplicity defaults to one instance per space. Plugins explicitly opt
  into multiple instances, clone-on-split, serialization, session binding, and
  settings page contributions.
- Plugin items receive a stable owning space. Session-specific items bind to one
  explicit session and close when that session ends.
- Plugin restoration is capability-driven. Failed item restoration produces one
  non-blocking summary and collapses invalid empty branches where appropriate.
- Native plugin libraries remain loaded for process safety. Disabling removes
  contributions, closes live instances after confirmation, and prevents loading on
  future launches; the mapped library stays inert until exit.
- Web plugins are hosted as real pane items in isolated webviews rather than a
  placeholder message.
- Safe web filesystem access is brokered, manifest-declared, canonicalized, and
  constrained beneath the owning folder, including protection against symlink
  escapes. Folderless safe instances receive plugin data storage only.
- Unsafe filesystem access is a persistent, explicit per-plugin grant. No global
  unsafe switch exists.
- Network, process, navigation, external-link, and session operations remain typed
  host actions with visible manifest declarations. Session control reaches only the
  explicitly bound session.
- Permission revocation closes active instances and revokes their broker. Native
  trust and web permissions are visible in Plugins settings.
- The plugin manifest and native ABI may be bumped to encode the new capabilities.
  Bundled examples move with the contract; incompatible plugins fail clearly.
- Herdr control requests use a fresh Unix connection and exact handshake. There is
  no long-lived reconnecting control client.
- A stream error removes the terminal command channel and renders an actionable
  notice inside that item. Reattach is offered only after confirming the stable
  session identity through the control plane.
- A small window-owned health state machine checks the private daemon, performs one
  clean replacement, and detects a second failure within 60 seconds as a crash
  loop. It exposes Retry and no backend administration UI.
- Backend loss removes terminal items and their session-bound plugins, collapses
  newly empty splits and outer tabs, and retains spaces plus space-bound plugins.
- Herdr is authoritative for live session existence. Orphaned sessions enter the
  owning space as standalone outer tabs; stale saved terminal items are dropped.
- Versioned SQLite persistence stores ordered space identities, ordered outer workspace
  tabs, pane trees, item records, active state, split ratios, window bounds,
  sidebar width/submode, chrome mode, expansion state, and migrations. A legacy
  single pane tree migrates to one outer workspace tab.
- User-editable settings, keymaps, and themes remain files. All persistent and
  runtime paths are namespaced to Chartr-zeddy; no automatic legacy import occurs.
- The supported platforms are macOS and Linux. Windows remains deferred until the
  Herdr transport is abstracted beyond Unix-domain sockets.

## Testing Decisions

- Tests observe behavior through the highest stable seam: the window/workspace
  action surface for UI behavior, serialized reload for persistence, the plugin
  host contract for contributions and permissions, and a real private Herdr
  process for transport behavior. Lower-level unit tests supplement rather than
  replace those seams.
- Ownership tests prove that an item entity is present in exactly one outer tab,
  pane, and space after add, outer-to-pane movement, cross-pane movement,
  split-edge drop, join, close, restore, and failed restore operations.
- Pane-group tests cover recursive split construction, flex resizing, directional
  adjacency/focus, edge-drop placement, join/collapse, empty-root invariants,
  zoom/maximize state, and serialization round trips. Property tests exercise long
  transformation sequences and assert tree and ownership invariants.
- Lifecycle tests assert that a single session close kills only that session;
  pane join kills none; session-bound plugins cascade; bulk operations confirm;
  space removal kills all owned sessions; and normal application exit detaches.
- Chrome tests assert that switching Tabbed, Sidebar/All Spaces, and Sidebar/Active
  Space changes only presentation; both chromes show the same standalone and
  grouped outer entries. Selecting and creating items from inactive groups must
  activate the correct space, outer tab, and pane without duplication. Sorter
  tests cover variable-height midpoint order, final release Y, interruptible
  FLIP, Reduce Motion, durable relaunch order, and registry-write rollback;
  pointer acceptance covers horizontal overdrag and edge autoscroll.
- Action tests use semantic commands and contexts, including close, Settings close,
  split, join, focus, move, zoom, palette dispatch, and keybinding conflicts.
- Settings tests cover default resolution, sparse user content, atomic updates,
  parse failure behavior, live observation, hotkey conflict reporting, theme
  selection, plugin page discovery, and restart-bound disclosures.
- Persistence tests launch from saved state and observe restored spaces, ordered
  outer tabs, multiple recursive layouts, active state, window/chrome geometry,
  unavailable folders, missing sessions, orphan sessions, missing plugins, and
  legacy single-layout migration.
- Native plugin tests cover trust labeling, per-space singleton behavior,
  multi-instance opt-in, clone capability, close, disable, settings contribution,
  serialization, ABI mismatch, and restoration failure.
- Web plugin tests cover real view hosting, safe project read/write, canonical and
  symlink containment, folderless storage, unsafe per-plugin access, declared
  network/process/session actions, permission display, and immediate revocation.
- Herdr unit tests cover protocol framing and lifecycle transitions. Required live
  tests launch the vendored private backend and cover handshake, shell painting,
  close/kill, detach/adopt, broken stream, confirmed reattach, one backend restart,
  and crash-loop Retry. Transport completion requires these live tests to pass.
- Visual acceptance captures Chartr Dark and Light at common window sizes for
  tabbed mode, both sidebar submodes, nested panes, drag targets, empty panes,
  confirmations, errors, settings, command palette, and plugin permissions.
- Visual review checks alignment, clipping, typography, semantic colors, hover,
  active and focus states, pane ownership grouping, and absence of placeholders.
- Accessibility tests cover keyboard-only equivalents, focus order/restoration,
  accessible labels, contrast, and reduced-motion behavior, using Zed's components
  and interaction behavior as the gold standard.
- The full workspace suite must pass. A mock-only success, ignored required live
  test, or knowingly broken requested flow does not satisfy the specification.

## Out of Scope

- Windows support and non-Unix Herdr transports.
- Cross-space tab movement or duplication.
- Terminal cloning or mirrored views.
- Preview tabs and pinned tabs.
- Automatic import or shared configuration with Go Chartr or Chartr-rs.
- Multiple operating-system windows; spaces provide independent workspace
  ownership within the Chartr window.
- A global unsafe mode for web plugins.
- A generic process-supervisor framework or backend administration UI.
- Phosphor compatibility or user-selectable application-control icon sets.
- Exposing non-default UI density before it has dedicated visual acceptance.
- A command, keybinding, or Hotkeys row for space sorting.
- A reusable generic sortable framework or user-configurable sort animation.

## Further Notes

- Zed is the architectural and interaction source of truth wherever it already
  supplies a convention. Go Chartr is a visual reference, not permission to embed
  fixed colors or bypass theme semantics. Chartr-rs is the behavioral precedent
  for Settings presentation and Herdr recovery.
- Importing Zed's complete workspace and settings UI crates is intentionally
  avoided because they carry unrelated editor, collaboration, language, remote,
  database, telemetry, audio, and agent-product dependencies. Focused Chartr
  implementations must still preserve the established Zed boundaries rather than
  inventing a different architecture.
- The current prototype is not an API compatibility constraint. It may be replaced
  wholesale when doing so produces the agreed model more directly.
- The test seams and acceptance coverage above were explicitly agreed during the
  grilling session and are the completion contract for implementation.
