# Installation

The Rust rewrite is currently built from source. Production release packages
are still being prepared; the [v0.2.4 downloads](https://github.com/rengwu/chartr/releases/tag/v0.2.4)
are for the legacy Go/Svelte version. You can also build a
[development DMG](releasing.md#macos-development-dmg) locally on macOS.

| Platform | Current support                                                    |
| -------- | ------------------------------------------------------------------ |
| macOS    | Native desktop app; CI runs on Apple silicon.                      |
| Linux    | Native desktop app under X11 or XWayland; CI runs on Ubuntu 24.04. |
| Windows  | Deferred until further notice                                      |

## Build from source

Install Git, Rustup, a C/C++ build toolchain, CMake, and pkg-config. On macOS,
install the Xcode Command Line Tools. Rustup reads the pinned Rust version and
components from [rust-toolchain.toml](../rust-toolchain.toml).

Install **Zig 0.15.2** to build the pinned Herdr sidecar. Chartr itself does not
require Zig. A matching sidecar is reused on subsequent builds.

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

These are the same system packages used by [CI](../.github/workflows/ci.yml).
Rustup and Zig 0.15.2 must be installed separately. On Ubuntu 24.04, you can
also install the system dependencies with `sh scripts/install-linux-deps.sh`.

</details>

```sh
git clone --branch rewrite/rust https://github.com/rengwu/chartr.git
cd chartr
rustup show

# Replace this path with your Zig 0.15.2 executable.
ZIG=/absolute/path/to/zig-0.15.2/zig sh vendor/herdr/fetch.sh

# Chartr reuses the sidecar; no Zig compiler is needed for this step.
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

## Linux release packages

The [Linux release workflow](../.github/workflows/release-linux.yml) builds native
x86_64 and ARM64 binaries on Ubuntu 24.04. Each build produces a `.tar.gz` and
`.deb`; x86_64 also produces an Arch Linux `.pkg.tar.zst`, using the same binaries.
Run the workflow manually for downloadable artifacts. A `v<workspace-version>`
tag creates a **draft** GitHub release after both architectures finish.

Install a downloaded package with `sudo apt install ./chartr_*.deb` on Ubuntu
24.04 or newer, or `sudo pacman -U ./chartr-*.pkg.tar.zst` on current Arch Linux.
For the tarball, extract it and run `./usr/bin/chartr` from the extracted folder;
system GTK/WebKitGTK and graphics libraries are still required. Keep the
`chartr` and `herdr` executables together. Updating the package replaces the
binaries and leaves your user configuration and sessions on disk intact.

See [Release builds](releasing.md) for the local commands, cache behavior,
and timing reports.

Once chartr is running, follow [Getting started](getting-started.md).
