# Installation

The [v0.3.0 release](https://github.com/rengwu/chartr/releases/tag/v0.3.0)
provides stable packages for the Rust rewrite.
The [v0.2.4 downloads](https://github.com/rengwu/chartr/releases/tag/v0.2.4) remain
available for the legacy Go/Svelte app. You can also build a
[development DMG](releasing.md#macos-development-dmg) locally on macOS.

| Platform | Current support                                                    |
| -------- | ------------------------------------------------------------------ |
| macOS    | Native desktop app for Apple silicon.                             |
| Linux    | Native desktop app under X11 or XWayland; CI runs on Ubuntu 24.04. |
| Windows  | Deferred until further notice                                      |

## Arch Linux / Omarchy

Download the x86_64 `.pkg.tar.zst` from the
[release page](https://github.com/rengwu/chartr/releases) and install it with:

```sh
sudo pacman -U ./chartr-*.pkg.tar.zst
```

This installs runtime dependencies, the `chartr` command, the app launcher entry,
and its icon. New packages are named `chartr-bin`; they provide `chartr` and
conflict with the older `chartr` package, so pacman can replace it cleanly.
Select just the package version you want if the download directory contains
several versions.

Once the first stable Rust release has been published to the AUR, install it on
Omarchy with:

```sh
omarchy pkg aur add chartr-bin
```

On other Arch installations with an AUR helper, use `yay -S chartr-bin`.
The AUR package downloads the prebuilt release; it does not compile Rust or Zig.
Updates then come through your AUR helper, including Omarchy's update flow.
Release candidates remain explicit downloads and do not replace the stable AUR
package. AUR availability depends on the maintainer completing the
[publishing setup](releasing.md#aur-publishing).

## Build from source

Install Git, Rustup, a C/C++ build toolchain, CMake, Ninja, and pkg-config. On macOS,
install the Xcode Command Line Tools. Rustup reads the pinned Rust version and
components from [rust-toolchain.toml](../rust-toolchain.toml).

Install **Zig 0.15.2** to build the pinned Herdr sidecar. Chartr itself does not
require Zig. A matching sidecar is reused on subsequent builds.

<details>
<summary>Ubuntu 24.04 build dependencies</summary>

```sh
sudo apt-get update
sudo apt-get install -y \
  build-essential clang cmake ninja-build pkg-config \
  libasound2-dev libfontconfig-dev libglib2.0-dev libssl-dev \
  libva-dev libvulkan1 libwayland-dev libx11-xcb-dev \
  libxkbcommon-x11-dev libzstd-dev libwebkit2gtk-4.1-dev \
  libnss3 libcups2t64 libxcomposite1 libxdamage1 libxrandr2 libgbm1
```

These are the same system packages used by [CI](../.github/workflows/ci.yml).
Rustup and Zig 0.15.2 must be installed separately. On Ubuntu 24.04, you can
also install the system dependencies with `sh scripts/install-linux-deps.sh`.

</details>

```sh
git clone https://github.com/rengwu/chartr.git
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

## macOS release packages

Download the `macos-arm64.dmg` for Apple silicon from the release page,
open the disk image, and drag `chartr.app` to Applications. `v0.3.0-rc.1` also
includes a `macos-x86_64.dmg` for Intel; this is the final candidate with Intel
packages. Subsequent macOS releases target Apple silicon only.

These builds are ad-hoc signed and not notarized. If macOS blocks
opening the app, review the download source and use **System Settings → Privacy
& Security → Open Anyway** after attempting to open it.

## Linux release packages

The [release workflow](../.github/workflows/release.yml) builds native
x86_64 and ARM64 binaries on Ubuntu 24.04. Each build produces a `.tar.gz` and
`.deb`; x86_64 also produces an Arch Linux `.pkg.tar.zst` and an
`aur.tar.gz` containing its `PKGBUILD` and `.SRCINFO`, using the same binaries.
Run the workflow manually for downloadable artifacts. A `v<workspace-version>`
tag creates a **draft** GitHub release after all three platform builds finish.
Candidate tags such as `v0.3.0-rc.1` are marked as prereleases. Debian candidates
use a version such as `0.3.0~rc.1`; Arch uses `0.3.0rc1`, so stable `0.3.0` sorts
after its candidates.

Install a downloaded package with `sudo apt install ./chartr_*.deb` on Ubuntu
24.04 or newer, or `sudo pacman -U ./chartr-*.pkg.tar.zst` on current Arch Linux.
For the tarball, extract it and run `./usr/bin/chartr` from the extracted folder;
system GTK/WebKitGTK and graphics libraries are still required. Keep the
`chartr` and `herdr` executables together. Updating the package replaces the
binaries and leaves your user configuration and sessions on disk intact.

### Optional native plugins

Plugin installation only fetches and validates prebuilt packages; it never runs
a compiler or build script. Plugins with native code must publish a package for
the current OS and architecture. See [plugin installation](plugins.md).

### Blank Wayfinder webviews on NVIDIA

Chartr automatically uses WebKit's shared-memory buffers when the NVIDIA kernel
module is loaded. This avoids black Wayfinder views caused by failed
GBM buffer allocation, while keeping WebKit's compositor enabled. The default
applies to both `cargo run -p chartr` and installed builds; macOS is unaffected.

If another driver has the same problem, try:

```sh
WEBKIT_DMABUF_RENDERER_FORCE_SHM=1 chartr
```

An explicit `WEBKIT_DMABUF_RENDERER_FORCE_SHM` or
`WEBKIT_DISABLE_DMABUF_RENDERER` value is respected. Set
`WEBKIT_DMABUF_RENDERER_FORCE_SHM=0` to opt out of the automatic workaround.
Avoid setting `WEBKIT_DISABLE_DMABUF_RENDERER=1` in launchers: it disables the
renderer instead of selecting its shared-memory transport.

See [Release builds](releasing.md) for the local commands, cache behavior,
and timing reports.

Once chartr is running, follow [Getting started](getting-started.md).
