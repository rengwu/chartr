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

That suite must paint a real shell, load ANSI host scrollback, hard-kill Herdr,
observe the broken stream, replace the daemon, and reject the stale session
identity.

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
- command palette, unavailable-folder recovery, broken-stream recovery, backend
  crash-loop banner, rejected plugin, and visible web permissions;
- the native Hello pane and real Clock web pane, including its persisted format.

Reject the build for clipping, overlapping hit targets, hard-coded feature
colors, inconsistent spacing, missing close controls, duplicated tabs, a webview
that survives its pane, or any placeholder standing in for an advertised plugin
tier.

## Interaction and accessibility

Run the matrix with pointer and keyboard. Confirm `Cmd/Ctrl+W`, command palette,
directional focus, move-to-existing-pane, join, Settings singleton focus, native
`Cmd/Ctrl+W` close, and `Ctrl+Tab` Settings-page cycling. Close the last workspace
and confirm Settings closes too.

Print more than two viewports of styled output in a terminal. With both a mouse
wheel and a trackpad, move to the oldest row and back to the live prompt. Confirm
small pixel deltas accumulate smoothly, colors survive in history, new output
does not pull a historical viewport to the bottom, and scrolling up again after
returning to the prompt refreshes the host history.

Open OpenCode, Claude Code, and Codex and scroll in both directions with a wheel
and a trackpad. Confirm each application viewport moves instead of host history.
Then hold Shift while scrolling and confirm the gesture reaches host scrollback
instead.

Confirm the active-space picker sits in the macOS title bar immediately after
the traffic lights, and the chevron menu sits at the far-right corner in both
chrome modes. Sidebar mode offers `Switch to Tabbed mode`, a
trailing-checkmarked `Show only active space` toggle, a separator, and
`Settings`; tabbed mode offers `Switch to Sidebar mode`, a separator, and
`Settings`. Switching presentation or sidebar scope updates immediately and
survives relaunch. On platforms with a native system title bar, confirm both
controls retain their in-app chrome positions.

In tabbed mode, the `+` control follows the last outer tab while they fit. When
the tabs overflow, only the tabs scroll: `+` pins beside their right edge, while
the title-bar chevron remains pinned at the window's far right. The padded `+`
cell retains a left divider against the scrolling tabs. Every tab retains its
minimum clickable width, including pane-local tabs within grouped workspaces.
Switch selection across both tab strips and confirm tab edges, following tabs,
and trailing controls remain stationary without a one-pixel shift.
Each pane-local `+` likewise follows its last tab on the left and stays pinned
beside the scrolling pane tabs. Outer and pane-local `+` controls use the same
icon size and padded divider cell; an empty outer strip keeps the `+` vertically
centered at the normal tab-bar height. The muted tab-bar background continues
after its compact cell and remains the append drop target without reserving width;
when pane tabs overflow, `+` reaches the pane's right edge.

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
their specific tabs, and its buttons retain their own actions.

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
