# Chartr

Chartr is a multi-space terminal and plugin workspace built on Zed's GPUI,
component, theme, action, and pane conventions. This rewrite keeps its data
isolated under the `chartr-zeddy` namespace.

```sh
sh vendor/herdr/fetch.sh
cargo run -p zeddy
```

The supported desktop targets are macOS and Linux under X11 or XWayland.
Windows is deferred because Herdr currently uses Unix-domain sockets. Wry's
in-window Linux child webviews require X11, so Chartr selects the same GPUI
backend instead of exposing web panes that fail only on Wayland.

## Spaces, panes, and items

One window owns ordered spaces and one active space, following Zed's
`MultiWorkspace` responsibility. The permanent **Ad-hoc sessions** space is
folderless and starts sessions in the home directory (or its configured
replacement). Folder spaces are canonical-path identities with independent
recursive pane trees.

Every terminal or plugin instance is one item owned by exactly one pane in one
space. Tabs never appear in several spaces. New sessions enter the active pane
of the selected space. A terminal item is non-cloneable; closing it terminates
its Herdr session. A plugin may opt into multiple instances, modifier cloning,
restoration, and explicit binding to one terminal session.

Panes support nested horizontal and vertical splits, divider resizing,
directional focus, joining, zooming, tab reordering, movement, and edge-drop
splitting. The command palette provides keyboard alternatives for pane
operations. `Cmd+W` on macOS and `Ctrl+W` on Linux closes the active item;
operations that terminate multiple live sessions confirm with an exact count.

Sidebar and tabbed modes are projections over that same model. Sidebar mode can
show all spaces or only the active space and groups each pane's items. Tabbed
mode shows one space and uses Zed tabs, including close controls for plugin
items. Switching presentation never reparents or recreates an item.

## Settings and persistence

Settings is presented inside the main window, retaining the spaces sidebar.
The implemented pages are General, Appearance, Terminal, Hotkeys, and Plugins.
Changes are written atomically; hotkeys are semantic GPUI actions with conflict
detection. Chartr Dark is the fixed default, with Chartr Light and system theme
pairs available. IBM Plex Sans and the bundled IBM Plex Mono are configurable
defaults.

User-editable data remains text:

- `$XDG_CONFIG_HOME/chartr-zeddy/settings.toml`
- `$XDG_CONFIG_HOME/chartr-zeddy/keymap.toml`
- `$XDG_CONFIG_HOME/chartr-zeddy/spaces.toml`

Application-owned window, chrome, pane, selection, and restorable-item state is
versioned SQLite under `$XDG_STATE_HOME/chartr-zeddy/state.sqlite`. No existing
Go Chartr or Chartr-rs configuration is imported automatically.

Normal app exit detaches sessions. An optional setting terminates them instead.
The private Herdr runtime uses an exact socket under
`$XDG_CONFIG_HOME/chartr-zeddy/herdr`; inherited Herdr selectors are cleared so
Chartr cannot attach to a user's standalone daemon. Broken streams become
item-local recovery states, and unexpected daemon death receives one clean
restart before entering a stable crash-loop state with Retry.

## Plugins

Plugins are directories under
`$XDG_DATA_HOME/chartr-zeddy/plugins/<reverse-dns-id>/` containing
`zeddy-plugin.toml`.

Native plugins are fully trusted Rust dynamic libraries. They receive a stable
`InstanceContext` and return ordinary GPUI views:

```rust
fn view(
    &mut self,
    pane: &PaneKey,
    context: &InstanceContext,
    window: &mut gpui::Window,
    cx: &mut gpui::App,
) -> gpui::AnyView;
```

Native plugins may advertise one lazy Settings contribution through their
registrar. Libraries remain mapped until process exit so disabling one cannot
invalidate a live Rust vtable.

Web plugins are real Wry panes with local assets and a restrictive CSP. Their
manifest declares project-file, domain-scoped network, process, and optional
bound-session host actions. Safe filesystem paths are canonicalized beneath the
owning project, while folderless plugins receive only plugin data. Unrestricted
filesystem access is an explicit per-plugin grant; there is no global unsafe
switch. Revoking a grant or disabling a plugin destroys its live brokers and
views immediately. A web plugin may name a lazy `settings_entry` document.

`plugins/hello` and `plugins/clock` are complete native and web examples. They
are development references and are not installed automatically.

## Repository boundaries

```text
crates/zeddy/              window, spaces, panes, settings, persistence, UI
crates/zeddy-herdr/        private Herdr protocol and lifecycle
crates/zeddy-vt/           terminal parser boundary
crates/zeddy-plugin/       native and manifest authoring contract
crates/zeddy-plugin-host/  discovery, loading, and web filesystem broker
plugins/                   one complete example per plugin tier
vendor/herdr/              pinned sidecar fetch and licence
docs/adr/                  architectural decisions
.plan/maps/                durable product specification
```

## Verification

```sh
cargo fmt --all --check
cargo test --workspace --locked --no-fail-fast
cargo check --manifest-path plugins/hello/Cargo.toml --locked
cargo test -p zeddy --test live_session -- --ignored --nocapture --test-threads=1
```

The last command launches and hard-crashes the real pinned private Herdr. The
macOS/Linux build matrix and release acceptance checklist live in
`.github/workflows/ci.yml` and `docs/acceptance.md`.

## Licence

GPL-3.0-or-later. Chartr links Zed's `ui` and `theme` crates directly; see
[ADR 0002](docs/adr/0002-the-zed-layer.md).
