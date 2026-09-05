# Code map

Chartr's workspace owns spaces; each space owns its live items and a serializable
pane layout. Keep behavior with its owner and reusable visual controls in
`crates/zeddy/src/components/`.

| Area | Entry point | Supporting modules |
| --- | --- | --- |
| Window and space coordination | `app.rs` | `app/view.rs` renders the window and dispatches actions. |
| Backend lifecycle | `app/backend.rs` | Connection, supervision, recovery, and exit policy. |
| Workspace persistence | `app/persistence.rs`, `persistence.rs` | Coalesced snapshots, ordered background writes, incremental SQLite updates, and shutdown flush. |
| Terminal search | `app/terminal_search.rs` | Search state updates, navigation, and overlay. |
| Pane layout UI | `app/panes.rs` | Pane headers, layout rendering, drag/drop, and resizing. |
| Plugin instances | `app/plugins.rs` | Launcher, shared view construction, restoration, and cloning. |
| Bundled plugins | `app/bundled_plugins.rs` | Catalog setup and packaged asset materialization. |
| Native plugin services | `crates/zeddy-plugin/src/services.rs` | Catalog-scoped Agent and Skills exports, with live provider availability. |
| Wayfinder | `plugins/wayfinder/app.js` | Web canvas in `starmap.js`; the permission-gated bridge in `src/lib.rs` owns source-aware prompts and file-derived claims. |
| Window controls | `app/window_chrome.rs` | Space switcher, problems menu, title bar, and action buttons. |
| Dialogs and commands | `app/rename.rs`, `app/command_palette.rs` | Rename lifecycle and command palette. |
| Settings integration | `app/settings_bridge.rs` | Workspace operations exposed to Settings. |
| Settings window | `settings_window.rs` | Shared state, setting updates, window frame, and field controls. |
| Settings pages | `settings_window/` | Appearance, general/terminal/hotkeys, and plugin pages. |
| Shared controls | `components.rs` | Reexports native modal hosting, popup menus, and selection controls. |
| Live content and model | `space.rs`, `item.rs`, `workspace.rs` | Runtime ownership and serializable pane/tab state. |

The `app/` and `settings_window/` modules implement their parent's view type.
The parent retains ownership of state; feature modules access it without adding
forwarding layers. Their helpers stay private unless another feature needs them.

Workspace changes coalesce for 250 ms before taking a snapshot; serialization
and SQLite writes run in the background, with only changed rows updated. Failed
saves retry after two seconds. Close/quit flushes synchronously to preserve the
latest layout, and revision checks prevent older queued work from overwriting it.

Run `cargo fmt --check` and `cargo test --workspace` after a refactor. The two
live-session tests require a real Herdr daemon and are ignored by default.
