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

That suite must paint a real shell, hard-kill Herdr, observe the broken stream,
replace the daemon, and reject the stale session identity.

## Visual matrix

Review at 700×900, 1100×720, and a maximized window in both Chartr Dark and
Chartr Light. Capture and compare:

- empty Ad-hoc startup, one folder, and several spaces;
- Sidebar / All Spaces, Sidebar / Active Space, and Tabbed mode;
- one pane, nested horizontal/vertical panes, resized dividers, zoom, and an
  intentionally empty pane;
- terminal and plugin close buttons, active/hover/focus states, edge drop target,
  grouped sidebar tabs, and the bulk-termination confirmation;
- General, Appearance, Terminal, Hotkeys, Plugins, and a contributed plugin
  Settings view;
- command palette, unavailable-folder recovery, broken-stream recovery, backend
  crash-loop banner, rejected plugin, and visible web permissions;
- the native Hello pane and real Clock web pane, including its persisted format.

Reject the build for clipping, overlapping hit targets, hard-coded feature
colors, inconsistent spacing, missing close controls, duplicated tabs, a webview
that survives its pane, or any placeholder standing in for an advertised plugin
tier.

## Interaction and accessibility

Run the matrix with pointer and keyboard. Confirm `Cmd/Ctrl+W`, command palette,
directional focus, split-and-move, move-to-existing-pane, join, zoom, Settings
close/focus restoration, and `Ctrl+Tab` Settings-page cycling. Every drag outcome
must have a semantic action alternative.

Inspect the GPUI accessibility tree on macOS and Linux. Tabs and Settings
navigation must expose roles, labels, and selection; Zed buttons and menus must
retain their labels and focus rings; contrast must remain readable in both
themes. Chartr introduces no animation, so reduced-motion mode requires no
alternate transition path.

## Persistence and lifecycle

Relaunch after changing window bounds, sidebar width/scope, mode, space names,
split ratios, active panes/items, plugin Settings, and a missing folder. Confirm
normal exit adopts detached terminals; item close kills exactly one session;
closing a populated pane or folder space confirms and kills all descendants;
session-bound plugins cascade; disabling or revoking a plugin closes every live
instance; and stale backend/plugin records are summarized without corrupting the
surviving pane tree.
