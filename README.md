# Chartr

Chartr is a multi-space terminal and plugin workspace built on Zed's GPUI,
component, theme, action, and pane conventions. This rewrite keeps its data
isolated under the `chartr-zeddy` namespace.

```sh
sh vendor/herdr/fetch.sh
cargo run -p zeddy
```

The workspace build requires Zig 0.16.0 for its pinned libghostty terminal
input encoder. The sidecar fetch currently builds an immutable post-0.8.2
Herdr revision, because the latest tagged release drops non-wheel mouse input
during direct attachment. That one maintenance step requires Rustup and Zig
0.15.2; set `ZIG` when needed. The source pin can return to a release asset once
Herdr tags its semantic direct-attach mouse forwarding.

The supported desktop targets are macOS and Linux under X11 or XWayland.
Windows is deferred because Herdr currently uses Unix-domain sockets. Wry's
in-window Linux child webviews require X11, so Chartr selects the same GPUI
backend instead of exposing web panes that fail only on Wayland.

For a macOS development disk image, run:

```sh
scripts/build-dev-dmg.sh
```

It produces an ad-hoc-signed, unnotarized `Chartr Dev.app` disk image and SHA-256
sidecar under `target/`. Pass an output path as the script's only argument to
place the image elsewhere.

## Spaces, panes, and items

One window owns ordered spaces and one active space, following Zed's
`MultiWorkspace` responsibility. The permanent **Free sessions** space is
folderless and starts sessions in the home directory (or its configured
replacement). Folder spaces are canonical-path identities with independent
outer tab collections and recursive pane groups.

Every terminal or plugin instance is one item owned by exactly one pane in one
outer tab and space. Tabs never appear in several places. New sessions and
plugins start as standalone outer tabs in the selected space. Dragging a
standalone onto a pane moves it into that group; dropping in the center joins
the pane and dropping at an edge creates a split. A terminal item is
non-cloneable; closing it terminates its Herdr session. A plugin may opt into
multiple instances, modifier cloning, restoration, and explicit binding to one
terminal session.

Panes support nested horizontal and vertical splits, divider resizing,
directional focus, joining, and Zed-style tab dragging. Tab and
trailing-strip drops reorder or move items; pane-body center drops move into a
pane; the four edge targets split it, with Zed's transient full/half-pane
highlight. Escape cancels a drag. The command palette provides keyboard
alternatives for pane navigation and moving or joining items. `Cmd+W` on macOS
and `Ctrl+W` on Linux closes the active item; operations that terminate multiple
live sessions confirm with an exact count. As in Zed, a non-root pane disappears
when its last item leaves; an outer tab disappears when its final item closes.
An empty space remains usable through its New action.

Sidebar and tabbed modes are projections over that same model. Both list every
standalone item and every pane group as one outer entry. Sidebar mode can show
all spaces or only the active space; tabbed mode keeps the active space's outer
entries beside its name. Selecting a group reveals its Zed-style draggable
pane-local tab bars, while a standalone has no duplicate inner bar. Switching
presentation never reparents or recreates an item. In All Spaces, drag a space
heading to reorder its whole card. The card stays locked to the sidebar's X axis,
continues tracking vertically outside the sidebar, autoscrolls at the list edges,
and settles into the closest legal slot at release.

Terminal titles follow Herdr's live view of the PTY, as in Chartr-rs: a detected
agent wins, otherwise the non-shell foreground process is shown, and an idle
shell falls back to Herdr's persistent tab label or number. The same two-second
backend refresh that discovers sessions updates and clears these inferred
titles. Collapsed pane groups can be renamed from their context menu and otherwise
use their item count as the title, such as **5 tabs**.

Every terminal is Zed's pinned `terminal` model and `TerminalView`, used as one
stack. Zed owns emulation, rendering, scrollback, resizing, keyboard
encoding, selection, clipboard, IME, links, and mouse reporting. Its local PTY
runs Herdr's native `terminal attach <id> --takeover` client; the persistent PTY
and shell remain owned by the private Herdr daemon. Chartr owns only attachment
lifecycle, pane placement, settings/theme inputs, and platform terminal bindings.
The pinned view has one documented host extension: Chartr can top-align the grid
instead of moving it by the spare sub-row pixels during pane resize. Zed's
bottom-alignment policy remains the default inside the vendored crate.
The pinned Zed terminal keymap supplies copy/paste, word navigation, scrollback,
vi mode, and character-palette behavior; Chartr adds terminal-buffer search,
desktop file drops, filesystem-link opening, and tab bell state at the host
boundary. Terminal font changes reflow live through the shared theme provider.

## Settings and persistence

Settings uses one application-wide native window, following Zed: every chrome
view menu, the command palette, and `Cmd/Ctrl+,` opens it or focuses the existing
instance.
It closes with the native window controls or `Cmd/Ctrl+W`, and closes when the
last workspace window closes. The implemented pages are General, Appearance,
Terminal, Hotkeys, and Plugins. Changes update every workspace live and are
written atomically; hotkeys are semantic GPUI actions with conflict detection.
Chartr Dark is the fixed default. Appearance exposes the same Ayu, Catppuccin,
Gruvbox, One, and VS Code catalog as Chartr-rs, plus Chartr Dark and Chartr
Light. Fixed mode chooses one theme; Match System keeps independent light and
dark selections. IBM Plex Sans and the bundled IBM Plex Mono are configurable
defaults. Reduce Motion disables the short space-sort settle animation while
retaining direct pointer tracking.

User-editable data remains text:

- `$XDG_CONFIG_HOME/chartr-zeddy/settings.toml`
- `$XDG_CONFIG_HOME/chartr-zeddy/keymap.toml`
- `$XDG_CONFIG_HOME/chartr-zeddy/spaces.toml`

Application-owned window, chrome, full space order, pane, selection, and
restorable-item state is versioned SQLite under
`$XDG_STATE_HOME/chartr-zeddy/state.sqlite`. Registered folder order is also the
file order in `spaces.toml`, so a registry write failure rejects and rolls back
the drop. No existing Go Chartr or Chartr-rs configuration is imported
automatically.

Normal app exit detaches sessions. An optional setting terminates them instead.
The private Herdr runtime uses an exact socket under
`$XDG_CONFIG_HOME/chartr-zeddy/herdr`; inherited Herdr selectors are cleared so
Chartr cannot attach to a user's standalone daemon. Closed attach clients become
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

The last command validates native attach targets and hard-crashes the real
pinned private Herdr. Interactive terminal behavior is covered by the release
acceptance checklist. The macOS/Linux build matrix and full checklist live in
`.github/workflows/ci.yml` and `docs/acceptance.md`.

## Licence

GPL-3.0-or-later. Chartr links Zed's `ui` and `theme` crates directly; see
[ADR 0002](docs/adr/0002-the-zed-layer.md).
