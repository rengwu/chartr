# chartr

<img alt="chartr icon" src="./docs/assets/v4/icon-mac-1024.png" width="34%" align="right">

**An extensible AI-native workspace, built with Rust.**

- [Build from source](#installation)
- [Getting started](#getting-started)
- [Documentation](#documentation)
- [Legacy v0.2.4 release](https://github.com/rengwu/chartr/releases/tag/v0.2.4)


chartr brings your CLI agents, skills, and tools into one workspace. Organize
projects into spaces, arrange terminals and tools side by side in split panes,
and switch between sidebar and tabbed views. Customize themes and fonts to
make it your own.

Extend your workflow with plugins, from agent launching and skill management
to planning and browsing. Use the included plugins, install more, or build your
own tools that live alongside your agents.

> chartr has been rewritten in Rust. The legacy Go/Svelte version is preserved
> on [`legacy/v0.2.4`](https://github.com/rengwu/chartr/tree/legacy/v0.2.4);
> v0.2.4 downloads are for that version. chartr is still in active development,
> and features and file formats may change before 1.0.


<img width="1169" height="857" alt="Screenshot 2026-09-08 at 4 11 24 AM" src="https://github.com/user-attachments/assets/8af894fe-4a0f-4247-b733-e9d521fd44a1" />

<br clear="right">

## Key features

- **Folders as spaces** — Keep each project's terminals and tools together,
  with a permanent Free sessions space for work outside a project.
- **Tabs and split panes** — Drag terminals and tools into groups, split them
  horizontally or vertically, and switch between sidebar and tabbed layouts.
- **Persistent sessions** — By default, your shells survive closing and reopening
  the app. Closing a terminal tab ends that session.
- **Persistent activity status** — A hideable status bar shows running sessions,
  terminal service health, and Companion sharing/connections even after its pane closes.
  Click a service for controls; restore the bar in Settings → General or with
  “Workspace: Toggle status bar” in the command palette.
- **Inbox** — Browse detected agent sessions in a searchable history sidebar,
  with the selected session’s original terminal on the right. Rename, archive,
  and launch agents without leaving the view. [Inbox details](docs/conversations.md).
- **Live terminal titles** — See the detected agent or foreground command in
  each tab, with the session label as its fallback.
- **Tools beside your terminals** — Agent, Skills, Saved Prompts, Markdown Prompt, and Wayfinder are
  bundled. Install additional web and hosted plugins, including the Browser surface.
- **Make it yours** — Choose themes and fonts, follow the system appearance,
  rebind shortcuts, and configure plugins in native Settings.
- **Bring your CLI agents** — Register the agents you already use and launch
  them with your own arguments, environment, and prompts.
- **Use your existing skills** — Register skills from local folders or remote
  Git repositories, and choose which sources take precedence.
- **Live star-map** — Explore your plan, ticket dependencies, and progress in
  the bundled Wayfinder surface.
- **Ticket-ready sessions** — Review the map, ticket, resolved blockers, and
  selected skills before launching an agent with that context.

## Installation

The rewrite is currently built from source. A development DMG can also be built
locally on macOS. Production release packages are still being prepared.

| Platform | Current support                                                    |
| -------- | ------------------------------------------------------------------ |
| macOS    | Native desktop app; CI runs on Apple silicon.                      |
| Linux    | Native desktop app under X11 or XWayland; CI runs on Ubuntu 24.04. |
| Windows  | Deferred until further notice                                      |

### Build from source

Install Git, Rustup, a C/C++ build toolchain, CMake, and pkg-config. On macOS,
install the Xcode Command Line Tools. Rustup reads the pinned Rust version and
components from [rust-toolchain.toml](rust-toolchain.toml).

Keep two Zig versions available: **0.15.2** for building the pinned Herdr sidecar,
and **0.16.0** on your `PATH` for the workspace build. The `ZIG` assignment below
selects the older compiler for the sidecar command only.

<details>
<summary>Ubuntu 24.04 build dependencies</summary>

```sh
sudo apt-get update
sudo apt-get install -y \
  build-essential clang cmake pkg-config \
  libasound2-dev libfontconfig-dev libglib2.0-dev libssl-dev \
  libva-dev libvulkan1 libwayland-dev libx11-xcb-dev \
  libxkbcommon-x11-dev libzstd-dev libwebkit2gtk-4.1-dev
```

These are the same system packages used by [CI](.github/workflows/ci.yml).
Rustup and the two Zig versions must be installed separately.

</details>

```sh
git clone --branch rewrite/rust https://github.com/rengwu/chartr.git
cd chartr
rustup show

# Replace this path with your Zig 0.15.2 executable.
ZIG=/absolute/path/to/zig-0.15.2/zig sh vendor/herdr/fetch.sh

# The workspace uses Zig 0.16.0 from PATH.
cargo run -p chartr --locked
```

The sidecar fetch builds an immutable Herdr revision with the direct-attach
mouse handling the rewrite needs. chartr uses its own private daemon and data
directory; a standalone Herdr installation is not required.

Pass a folder to register and open it as a space at launch:

```sh
cargo run -p chartr --locked -- /path/to/project
```

No frontend build is needed. The bundled web surfaces ship with their assets.

### macOS development DMG

After fetching the sidecar, build a disk image with:

```sh
scripts/build-dev-dmg.sh
```

The script builds in release mode and produces a `chartr.app` DMG and SHA-256
checksum under `target/`. The bundle uses the macOS app artwork in
`docs/assets/v4/`, including the dedicated small-size variants. The app is
ad-hoc signed and unnotarized. Pass an output path as the script's only argument
to put the image elsewhere.

## Getting started

1. **Open a space.** Add a project folder, or use Free sessions for a shell
   outside a project. The `+` button opens a terminal.
2. **Register an agent.** Open **Settings → Plugins → Agent → Configure** and
   add an installed CLI agent and its launch settings.
3. **Register your skills.** Open **Settings → Plugins → Skills → Configure**
   and add local folders or Git repositories containing your skills.
4. **Chart your work.** Open the Agent surface using the **New surface** button
   beside `+`. Work with your agent to write a plan under `.plan/maps/`, following
   the [tracker convention](plugins/wayfinder/TRACKER-CONVENTION.md).
5. **Drive the map.** Open Wayfinder, choose a map and a ready ticket, then use
   **Review & launch** to inspect the prompt and start its agent session.

Wayfinder can browse existing maps before agents or skills are configured. It
allows one claimed ticket per space at a time; ordinary agent sessions and
terminals remain independent.

To add the optional Browser surface from this checkout, choose `plugins/browser`
under **Settings → Plugins → Install from Folder…**, then restart when prompted.
See [Browser](plugins/browser/README.md) for its capabilities and limits.

## Your data

chartr runs locally and does not require a chartr account. Maps and tickets are
Markdown files in your project. The application keeps its other data under the
`chartr` namespace:

| Data                                      | Default location                    |
| ----------------------------------------- | ----------------------------------- |
| Settings, shortcuts, and registered spaces | `~/.config/chartr/`                  |
| Window and workspace state                | `~/.local/state/chartr/state.sqlite` |
| Installed plugins and plugin data         | `~/.local/share/chartr/`             |
| Private Herdr runtime                     | `~/.config/chartr/herdr/`            |

`XDG_CONFIG_HOME`, `XDG_STATE_HOME`, and `XDG_DATA_HOME` override the corresponding
base directories. These defaults apply on both macOS and Linux. Configuration
and data from previous development namespaces, Go chartr, or chartr-rs are not
imported automatically.

## Documentation

- [Workspace reference](docs/workspace.md) — spaces, panes, terminals, settings,
  persistence, and plugin behavior
- [Plugin packages](docs/plugins.md) — installation, permissions, and host APIs
- [Skills](plugins/skills/README.md) — source registration and discovery
- [Saved Prompts](plugins/prompts/README.md) — saved prompts and reusable prompt data
- [Markdown Prompt](plugins/markdown-prompt/README.md) — compose and maintain Markdown from text and plugin templates
- [Wayfinder](plugins/wayfinder/README.md) — maps, prompts, and ticket launches
- [Tracker convention](plugins/wayfinder/TRACKER-CONVENTION.md) — the map and
  ticket file format
- [Code map](docs/code-map.md) — where each part of the app lives
- [ADRs](docs/adr/) — why it is shaped the way it is
- [Release acceptance](docs/acceptance.md) — the automated and hands-on release
  checklist

## Project status

Development continues here on `rewrite/rust`, with the complete `chartr`
commit history preserved. The Go implementation at v0.2.4 remains on
[`legacy/v0.2.4`](https://github.com/rengwu/chartr/tree/legacy/v0.2.4).

The Rust workspace, persistent terminals, native Settings, plugin host, and
bundled Agent, Skills, Saved Prompts, Markdown Prompt, and Wayfinder surfaces are implemented. CI builds
and tests the workspace and native plugin example on macOS and Ubuntu.

Before the first Rust release:

- Complete hands-on terminal, plugin, persistence, and recovery acceptance on
  each shipping architecture.
- Prepare distributable macOS and Linux packages and a release workflow.
- Document installation and upgrade behavior, then publish a release candidate.

Native Wayland web panes and Windows support remain outside the current target.
See [open issues](https://github.com/rengwu/chartr/issues) for additional reports.

### Development checks

```sh
cargo fmt --all --check
cargo test --workspace --locked --no-fail-fast
cargo check --manifest-path examples/plugins/hello/Cargo.toml --locked
node --test crates/chartr/tests/plugin_bridge.cjs
node --test plugins/wayfinder/tests/layout.test.mjs
```

The JavaScript checks require Node.js. The real-sidecar tests are ignored by the
default suite; after fetching Herdr, run them explicitly before release:

```sh
cargo test -p chartr --test live_session --locked -- --ignored --nocapture --test-threads=1
```

That suite deliberately kills and restarts its private test daemon to verify
recovery. Interactive terminal and webview behavior is covered by the
[release acceptance checklist](docs/acceptance.md).

## Related projects

- [Zed](https://github.com/zed-industries/zed) — the GPUI framework, UI components,
  themes, and terminal stack used by the rewrite
- [Herdr](https://github.com/herdrdev/herdr) — the persistent terminal backend
  that powers chartr's sessions
- [wayfinder-maps](https://github.com/rengwu/wayfinder-maps) — the read-only map CLI
  and viewer where the star-map started
- [mattpocock/skills](https://github.com/mattpocock/skills) — the original
  `/wayfinder` skill and the method that inspired the maps workflow

## Acknowledgements

- [@brownoxford](https://github.com/brownoxford) for privately reporting
  vulnerabilities that helped harden the original implementation's localhost
  trust boundaries.
- [@bradymwilliams](https://github.com/bradymwilliams) for
  [reporting an issue](https://github.com/rengwu/chartr/pull/5) that led to
  improvements when opening chartr from monorepo subdirectories.

## Licence

[GPL-3.0-or-later](LICENSE-GPL). chartr links Zed's `ui` and `theme` crates
directly; see [ADR 0002](docs/adr/0002-the-zed-layer.md).
