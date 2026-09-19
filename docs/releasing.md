# Release builds

Linux releases have one native compilation per architecture and reuse those
binaries across package formats. The workflow is intentionally separate from
test CI: running tests must not force a second dev-profile build in a release
job. Existing CI and hands-on [acceptance](acceptance.md) remain release gates.

## GitHub Actions

`Release packages` runs on default-branch pushes (warming the release cache),
manual dispatches, and `v*` tags. A tag must exactly match `v` plus the
workspace version in `Cargo.toml`, for example `v0.3.0` or `v0.3.0-rc.1`.
The [workflow](../.github/workflows/release.yml) builds Linux x86_64/ARM64 and
macOS Apple silicon before assembling a draft release. Intel macOS packages
are no longer built. Candidate tags
create prereleases; publishing them must not update the latest stable release.

1. Install Ubuntu system dependencies and the pinned Rust toolchain.
2. Restore the release dependency cache and the exact pinned Herdr executable.
   Only a Herdr cache miss installs Zig 0.15.2 and compiles the sidecar.
3. Run `cargo build --release --locked -p chartr --bin chartr --timings` once.
4. Package the same binaries as `.tar.gz` and `.deb`. For x86_64, an Arch Linux
   container checks dynamic-library resolution and makes a `.pkg.tar.zst`
   without compiling anything.
5. Upload packages, per-architecture SHA-256 checksums, and timing reports.
   Tag builds also create/update a **draft** release after the macOS jobs finish.
   Reruns refuse to replace
   assets on an already published release.

Ubuntu 24.04 is the build and minimum Ubuntu runtime baseline. These are glibc
builds with system GTK 3 and WebKitGTK 4.1 dependencies, not static executables
or universal Linux binaries. Arch Linux packaging targets x86_64; no Arch ARM
package is claimed. X11/XWayland and a working Vulkan driver are required.
The package places both executables in `/usr/lib/chartr` and exposes `chartr`
through `/usr/bin/chartr`. The tarball carries the same layout with a relative
symlink, so it can also run directly from an extracted directory.

No frontend build, Zig 0.16 installation, cross-compilation setup, whole-program
LTO, or per-format recompilation is involved. Release compiler settings retain
Cargo's defaults; the optimization comes from eliminating repeated work without
changing application behavior or taking on a fork of Zed's dependency graph.

Candidate versions keep their `-rc.N` suffix in Cargo and tarball/DMG names.
Debian maps that suffix to `~rc.N`, and Arch maps it to `rcN`; both package
managers must sort the candidate before the matching stable release. The Debian
fixture test and the real Arch packaging job verify that ordering.

## Local Linux build

On Ubuntu 24.04:

```sh
sh scripts/install-linux-deps.sh
rustup show
ZIG=/path/to/zig-0.15.2/zig sh vendor/herdr/fetch.sh
cargo build --release --locked -p chartr --bin chartr --timings
scripts/package-linux.sh
```

Packages land in `target/packages`. `package-linux.sh` also accepts a binary
directory and output directory, so a custom Cargo target directory can be used.
It checks native ELF architecture, shared-library resolution, and the exact
Herdr version before packaging. Debian dependencies are derived by
`dpkg-shlibdeps`, with dynamically loaded libraries declared explicitly. It
strips only the packaged copies and uses fast gzip compression.

For the Arch x86_64 package, after the tarball is built, run:

```sh
docker run --rm -v "$PWD:/work" -w /work \
  archlinux:base-devel bash scripts/package-arch.sh
```

The container installs Arch runtime dependencies, then runs `makepkg` as an
unprivileged user. The PKGBUILD copies the existing installation tree. Docker
must be available to your user. This does not install anything on the host.

## macOS development DMG

After installing the [build prerequisites](installation.md#build-from-source)
and fetching the sidecar, run this from the repository root:

```sh
scripts/build-dev-dmg.sh
```

The script builds in release mode and produces a `chartr.app` DMG and SHA-256
checksum under `target/`. The bundle uses the macOS app artwork in
`docs/assets/v4/`, including the dedicated small-size variants. The app is
ad-hoc signed and unnotarized. Pass an output path as the script's only argument
to put the image elsewhere.

Release automation uses the same verified bundle path with
`CHARTR_BUNDLE_ID=io.github.rengwu.chartr`, retaining the production identifier
from earlier releases. Local builds default to `dev.chartr.dev`. The full
candidate version is recorded in `chartrReleaseVersion`; Apple's numeric
`CFBundleShortVersionString` uses the corresponding base version. Release DMGs
remain ad-hoc signed and unnotarized, and the release notes must say so.

## Caches and build time

The shared `setup-herdr` action caches the finished sidecar by runner image,
CPU architecture, Herdr pins, Rust pin, fetch script, and action definition.
There is no fallback to a different sidecar cache key. The binary is checked
with `--version` and saved immediately, so a later application build failure
does not discard a successful Herdr build. CI and release jobs share this cache.

Locally, `fetch.sh` also reuses an already matching native sidecar. On a miss,
it retains source under `vendor/herdr/.build/source/<revision>` and Cargo output
under `vendor/herdr/.build/target`. That directory is excluded from Chartr's
Cargo workspace so upstream builds remain independent. Rebuilding explicitly is:

```sh
HERDR_REBUILD=1 ZIG=/path/to/zig-0.15.2/zig sh vendor/herdr/fetch.sh
```

Use this after changing sidecar compiler flags, or when intentionally rebuilding
with a different compiler/environment. The Rust toolchain is selected explicitly
from this repository even after entering the upstream source directory. Failed
builds leave the previous sidecar intact. Do not run simultaneous fetches into
the same checkout.

Test and release dependency caches have separate keys. Rust-cache handles Rust
version, lockfile, manifest and compiler-environment invalidation; unchanged
registry and Git dependencies are reusable. Workspace code is rebuilt normally.
GitHub allows a tag to restore default-branch caches, but not a different tag's
caches. Keeping default-branch builds enabled prevents every release tag from
starting cold. Caches may still be evicted; every job supports a cold build.

A first build still compiles the large Zed editor/terminal dependency graph.
The local `.cargo/config.toml` is untracked and does not constrain CI jobs.
Test CI disables debug symbols to reduce disk and memory use. Release jobs use
Cargo's normal parallelism and do not inherit the local two-job limit.

Both builds use `--timings`; reports are uploaded even after later steps fail:

- Chartr: `target/cargo-timings/`
- Herdr: `vendor/herdr/.build/target/cargo-timings/`

Compare a cold run and a subsequent run with unchanged dependency pins. Record
Herdr cache hits, Cargo compile time, and packaging time separately. A cache hit
eliminates sidecar compilation entirely; exact wall-clock improvements depend
on the runner and dependency cache contents.

References: [Cargo timings](https://doc.rust-lang.org/cargo/reference/timings.html),
[Rust cache](https://github.com/Swatinem/rust-cache), and
[GitHub cache scope and retention](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching).
