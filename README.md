# zeddy

A simple agent multiplexer. Sessions live in a backend that outlives the
window; the window shows them in a sidebar or in a tab strip, and plugins add
panes beside them.

```sh
sh vendor/herdr/fetch.sh   # once per checkout, and whenever the pin moves
cargo run -p zeddy
```

## What it is

Open zeddy in a directory and it shows the sessions already running there,
adopting them rather than restarting them. `+` starts another. Quitting leaves
them running; the next launch picks them up where they were.

Sessions are **agents**, not just shells — the backend already knows what a
pane is running, so a session carries its agent's name and status without zeddy
inspecting a process tree.

## Two modes

The same list, in the two places a list of sessions wants to be:

- **Sidebar** — a vertical list down the left. Room for a title, the agent
  under it, and a close button that is not fighting the title for space. The
  mode for many long-lived sessions.
- **Tabs** — a horizontal strip across the top. Denser per session, familiar,
  and no room for a second line. The mode for a handful you are switching
  between quickly.

Both are one enum and one branch in `render`. Toggling never touches a session,
because nothing below the chrome knows which mode is showing.

## Layout

```text
crates/zeddy/              the window, and nothing a lower crate could own
crates/zeddy-herdr/        the only code that knows herdr exists
crates/zeddy-vt/           the only code that knows a VT parser exists
crates/zeddy-plugin/       the contract a plugin is written against
crates/zeddy-plugin-host/  the only code that loads foreign code
plugins/                   one example per tier; never installed automatically
vendor/herdr/              the pinned backend executable and its fetch script
docs/adr/                  the decisions that would otherwise be re-litigated
```

Each crate is a boundary rather than a bag of helpers. Swapping the VT parser
is a change to one file; so is swapping the backend.

## The backend is invisible

zeddy runs a **private** herdr: its own socket, its own XDG directories, its own
session name, under `~/.local/state/zeddy/herdr`. It does not discover, attach
to, stop, upgrade, or write the herdr you run yourself, and a `HERDR_SOCKET_PATH`
inherited from your shell cannot reach it — every herdr process zeddy launches is
placed in that namespace explicitly.

There is no backend administration surface. Starting and adopting are one call,
because the question zeddy acts on is not "is it running" but "can I talk to
it", and that is a `ping`.

The executable is resolved by path, beside zeddy's own, never through `PATH`: a
herdr you installed for yourself is yours, and picking it up would make zeddy's
backend version depend on the machine. `crates/zeddy/build.rs` copies the
vendored one into place and fails the build if it is not there.

## Plugins

Both tiers contribute the same thing — a pane zeddy can show in the sidebar or
as a tab — and nothing above the plugin host asks which tier a pane came from.
A plugin is a directory with a `zeddy-plugin.toml` in it under
`~/.local/share/zeddy/plugins/<id>/`.

**Native** (`kind = "native"`) is a `cdylib`. Its view is an ordinary GPUI
`AnyView` mounted directly in zeddy's element tree, so scrolling, resizing,
focus, input, and painting use exactly the same frame path as a built-in.
There is no webview, Wasm runtime, synthetic window, display-list replay, or UI
RPC layer. The whole authoring contract is one trait, one macro, and a manifest:

```rust
use zeddy_plugin::{Host, PaneKey, Plugin, Registrar, gpui, register};

struct StarMap;

impl Plugin for StarMap {
    const ID: &'static str = "com.example.starmap";
    fn new(_: Host, _: &mut gpui::App) -> Self { Self }
    fn activate(&mut self, r: &mut Registrar, _: &mut gpui::App) { r.add_pane("map", "Star map"); }
    fn view(&mut self, _: &PaneKey, _: &mut gpui::Window, cx: &mut gpui::App) -> gpui::AnyView {
        cx.new(|_| MapView::default()).into()
    }
}

register!(StarMap);
```

That openness is also the trust model. A native plugin may use raw GPUI, any
compatible crate, the filesystem, processes, and the network; installing one is
installing native code, and no sandbox is claimed.

**Web** (`kind = "web"`) is a manifest and an entry document. No Rust, no
toolchain, no ABI to match — anyone who has written a web page can write one,
and it is sandboxed. The trade is that its pane is composited rather than
painted on zeddy's frame path, so it is a frame behind the terminal beside it.

The two tiers exist because "anyone can author one" and "fast enough to paint a
star map at 120fps" are different requirements, and one runtime cannot honestly
be both. See [ADR 0003](docs/adr/0003-two-plugin-tiers.md).

`plugins/hello` is a complete native plugin; `plugins/clock` is a complete web
one. Neither is seeded or installed automatically.

## Testing

```sh
cargo test --workspace
```

Hermetic: no test contacts a herdr daemon. The pieces that can only be checked
against a real one are ignored by default and use the vendored executable:

```sh
cargo test -p zeddy --test live_session -- --ignored --nocapture
```

Run that one when the herdr pin moves. The frame stream rides herdr's command
line, which carries no compatibility promise, and it is the one coupling no
unit test can see break.

## Licence

GPL-3.0-or-later. zeddy links Zed's `ui` and `theme` crates directly, and those
are GPL-3.0-or-later; see [ADR 0002](docs/adr/0002-the-zed-layer.md).
