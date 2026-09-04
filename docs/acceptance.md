# Release acceptance

The durable behavior contract is the
[workspace specification](../.plan/maps/chartr-zeddy-workspace/spec.md). This
checklist is the release gate, not a second specification.

## Automated gate

Both macOS and Ubuntu must pass `.github/workflows/ci.yml`, including formatting,
the locked workspace suite, the native plugin contract build, and the Linux Wry
and GPUI X11 link. Before release, run the real-sidecar suite locally on each
shipping architecture:

```sh
cargo test -p zeddy --test live_session -- --ignored --nocapture --test-threads=1
```

That suite must handshake the exact sidecar, create a persistent terminal,
produce a namespace-safe native attach target, hard-kill Herdr, replace the
daemon, and reject the stale session identity.

## Visual matrix

Review at 700×900, 1100×720, and a maximized window in both Chartr Dark and
Chartr Light. Capture and compare:

- empty Free sessions startup, one folder, and several spaces;
- several variable-height space cards before, during, and after a reorder;
- Sidebar / All Spaces, Sidebar / Active Space, and Tabbed mode;
- empty space, one standalone tab, nested horizontal/vertical panes, resized dividers,
  and automatic split collapse after its last item moves or closes;
- terminal and plugin close buttons, active/hover/focus states, two standalone
  outer tabs beside one collapsed three-item pane group in both chromes, visible
  draggable tab bars in every selected group pane, and bulk confirmation scoped
  to only the selected group;
- live terminal titles changing from their Herdr tab number to `nano`, `htop`,
  or a detected agent and back when that foreground process exits; every
  collapsed pane groups can be renamed from their context menu, blank names
  restore the count title, and unnamed groups track their current item count,
  such as `5 tabs`;
- Zed-style transient pane-body drop highlights: full-content center and
  half-content left, right, top, and bottom targets, including nearest-edge
  corner resolution and no split target over a pane's tab bar;
- one native, application-wide Settings window with General, Appearance,
  Terminal, Hotkeys, Plugins, and a contributed plugin Settings view;
- command palette, unavailable-folder recovery, closed-attach recovery, rejected
  plugin, visible web permissions, and the top-right Problems menu—including
  timestamps plus retry and restart actions for a backend crash loop;
- the native Hello pane and real Clock web pane, including its persisted format.

Install a web plugin from both a local folder and a Git repository in Settings →
Plugins. Confirm that its declared permissions appear before installation, the
managed copy is not a live reference to the source, and **Later** leaves a
restart-required banner. Replace that plugin and confirm its data survives.
Attempt to install a separately compiled native GPUI plugin and confirm Chartr
rejects it without executing or loading the library. Confirm **Restart**
persists the workspace, relaunches Chartr, and exposes a newly installed web or
hosted contribution.

Install the separately packaged `com.chartr.browser` plugin and exercise it in
standalone, grouped, and split panes on macOS and Linux/X11. Confirm its
manifest-only package installs with no compiler or platform binary. Verify the
themed toolbar, URL/search interpretation, redirects, Back/Forward, Stop/Reload,
keyboard shortcuts, one page per pane, current-pane handling of new-window
links, native file uploads, system-browser download handoff, denied site
permissions, ephemeral web-engine storage, and per-pane last-URL restoration.
Network/TLS failures may retain the operating system web engine's error page.

Reject the build for clipping, overlapping hit targets, hard-coded feature
colors, inconsistent spacing, missing close controls, duplicated tabs, a webview
that survives its pane, or any placeholder standing in for an advertised plugin
tier.

## Interaction and accessibility

Run the matrix with pointer and keyboard. Confirm `Cmd/Ctrl+W`, command palette,
directional focus, move-to-existing-pane, join, Settings singleton focus, native
`Cmd/Ctrl+W` close, and `Ctrl+Tab` Settings-page cycling. Close the last workspace
and confirm Settings closes too.

In General settings, confirm the sidebar middle-click switch is hidden until
"Middle click to close tab" is checked. With only the parent enabled, middle-click
standalone and grouped tabs in tabbed mode and pane-local tabs; each target closes,
while sidebar rows do not. Enable the sidebar checkbox and confirm its standalone
and grouped rows close as well. Disable the parent and confirm none of these
surfaces close from a middle click.

Confirm "Show space picker in sidebar mode" appears in General settings. Turning
it off hides the title-bar picker in sidebar mode without hiding any space cards;
turning it on restores the picker, and the choice survives relaunch.

Exercise the terminal as a terminal, not only as a shell prompt:

- paste single-line and multiline text with the platform shortcut and context
  menu, then copy a pointer and keyboard selection back out;
- press Shift+Enter in a multiline-capable prompt and confirm it inserts LF
  without submitting, while Enter submits normally;
- use Option/Alt+Left and Option/Alt+Right to move by words;
- print more than two viewports of styled Unicode output, then use both a mouse
  wheel and a trackpad to reach the oldest row and return to the live prompt;
- run alternate-screen TUIs such as `less`, `nano`, and `htop`; confirm wheel,
  trackpad, mouse clicks, dragging, arrow keys, function keys, and resize reports
  reach the application instead of moving host scrollback;
- verify double/triple-click selection, select all, clear, scroll-to-top/bottom,
  URL and filesystem hyperlinks, wide glyphs, combining marks, and IME
  composition;
- open terminal search (`Cmd+F` on macOS, `Ctrl+Shift+F` elsewhere), confirm
  literal punctuation is matched, cycle in both directions, and dismiss back
  to the terminal without sending the search keystrokes to the shell;
- drag one or more files from the desktop into a terminal and confirm their
  shell-quoted paths are pasted exactly once;
- change terminal font family and size while a terminal is visible and confirm
  the grid reflows immediately without restart, clipping, or stale alignment;
- emit BEL and confirm the tab indicator appears, then type in that terminal
  and confirm the indicator clears.

Repeat the input and scrolling checks in standalone, grouped, and split panes,
including after detach/reattach and window resize. Reject any duplicate input,
stale viewport, focus loss, or interaction that works only in one pane shape.

Confirm the space picker sits in the macOS title bar immediately after the
traffic lights. At the far-right corner in both chrome modes, confirm the
`Sidebar` / `Tabbed` segmented control reflects and changes the presentation,
and the adjacent gear button opens Settings without the workspace reclaiming
window focus. Switching presentation or picker visibility updates immediately
and survives relaunch.
On platforms with a native system title bar, confirm both controls retain
their in-app chrome positions.

In tabbed mode, the `+` and adjacent plugin-pane controls follow the last outer tab
while they fit. When the tabs overflow, only the tabs scroll: both controls pin
beside their right edge, while the title-bar controls remain pinned at the
window's far right. The padded control cell retains a left divider against the
scrolling tabs. Every tab retains its minimum clickable width, including
pane-local tabs within grouped workspaces. Confirm every `+` immediately opens
a terminal session without presenting a context menu.
Switch selection across both tab strips and confirm tab edges, following tabs,
and trailing controls remain stationary without a one-pixel shift.
Confirm Browser, Clock, and Hello show their manifest-selected Hugeicons in
sidebar rows, standalone outer tabs, and pane-local tabs. Grouped outer tabs
continue to show the split indicator instead of one representative plugin icon.
Confirm a temporary **New Plugin Pane** tab shows the Full Screen icon in
each of those tab surfaces until a plugin replaces it in place.
Open that picker and confirm every plugin card shows the same manifest-selected
Hugeicon beside its plugin name.
Each pane-local `+` and plugin-pane pair likewise follows its last tab on the left
and stays pinned beside the scrolling pane tabs. Outer and pane-local controls
use the same icon size and padded divider cell; an empty outer strip keeps the
pair vertically centered at the normal tab-bar height. The muted tab-bar
background continues after its compact cell and remains the append drop target
without reserving width; when pane tabs overflow, the controls reach the pane's
right edge. Confirm each plugin-pane control opens the plugin picker at its indicated
location, including within the selected pane and within an inactive sidebar
space card.

For tab dragging, exercise each pane-body center and edge target, both corner
choices, before and after insertion on existing tabs, trailing-strip append,
movement between panes, and movement of the last source tab. Confirm the source
pane collapses only when it becomes empty, pane focus follows pointer selection,
`Escape` cancels without moving or cloning, and the platform clone modifier
(Option on macOS, Control elsewhere) clones only opt-in plugin items while all
other items move normally. Repeat with terminal, native-plugin, and web-plugin
items; tab headers must remain visible and draggable throughout. Clicking inside
a web plugin must activate its pane, and its native child view must yield during
a drag so neither the tab preview nor drop highlight is obscured.

In the All Spaces sidebar, clicking anywhere on a space card activates that
space and returns keyboard focus to its workspace. Its session rows still select
their specific tabs, and its `+` and plugin-pane buttons retain their own actions.

Create five standalone tabs in one space. Move tabs 4 and 5 into tab 3, split
tab 4 to the right, and leave tabs 1 and 2 standalone. Both sidebar and tabbed
chrome must show exactly three outer entries: tab 1, tab 2, and one three-item
group. While that group is selected, drag either standalone outer entry into
the center and each of the four edges of every pane. Confirm the source outer
entry disappears, the target group remains selected, and no item is duplicated.

Inspect the GPUI accessibility tree on macOS and Linux. Tabs and Settings
navigation must expose roles, labels, and selection; Zed buttons and menus must
retain their labels and focus rings; contrast must remain readable in both
themes.

In Sidebar / All Spaces, drag the top, middle, and bottom cards by their headings.
The complete card must track the pointer vertically without horizontal drift;
neighbouring variable-height cards must change places only after their midpoints
are crossed. Move the pointer beyond both vertical scroll edges and confirm
Zed-style autoscroll. Move it horizontally into the workspace, continue upward
or downward, and release there: the card must settle in the closest legal slot
for that final Y. `Escape` and a simulated `spaces.toml` write failure must snap
back to the model order. Repeat rapid direction reversals to confirm the 150 ms
FLIP settle remains continuous. Enable Appearance / Reduce Motion and repeat:
direct pointer carrying and sorting remain, while displaced-card and release
settle animations are absent.

Open the bundled native Agent plugin in a folder space and in Free sessions. Confirm
its plugin card and tabs show the Chip icon. With an empty registry, confirm the
composer, picker, and launch action are disabled;
the space and Git branch sit above rather than inside the composer; and the prompt
placeholder uses muted text. Register agents named for Claude, Codex, Grok,
OpenCode, and Pi, then confirm both the picker trigger and menu infer the matching
Hugeicons glyph (with the generic AI-programming glyph for OpenCode).
**Register your first agent** must navigate to **Agent management** and open the
registration dialog immediately. Register an adapter with arguments and each
prompt-delivery mode, edit it through the same dialog, and confirm deletion is
guarded by a confirmation. Return through both Back and the pane chevron's
**Manage agents** item, launch a non-empty prompt, and confirm the resulting
Chartr-owned terminal opens in the pane's owning space with the registered
environment, arguments, and prompt delivery. Relaunch Chartr and confirm the
agent registry remains available in every space.

## Persistence and lifecycle

Relaunch after changing window bounds, sidebar width/scope, mode, full space
order (including Free sessions and a recovered missing folder), space names,
outer-tab order, split ratios, active groups/panes/items, plugin Settings, and a
missing folder. Confirm the sidebar and `spaces.toml` retain the committed order.
Confirm normal exit adopts detached terminals; item close kills exactly one session;
closing a populated pane or folder space confirms and kills all descendants;
session-bound plugins cascade; disabling or revoking a plugin closes every live
instance; and stale backend/plugin records are summarized without corrupting the
surviving pane tree.
