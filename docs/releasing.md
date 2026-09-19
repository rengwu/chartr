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
   container checks dynamic-library resolution and makes a `chartr-bin`
   `.pkg.tar.zst` without compiling anything. It also produces
   `chartr-<version>-aur.tar.gz` with the exact tested `PKGBUILD` and `.SRCINFO`.
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
Debian maps that suffix to `~rc.N` inside the package while keeping `-rc.N` in
the filename (GitHub normalizes tildes in asset names). Arch maps it to `rcN`; both package
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

To generate just the AUR recipe on Arch, as a regular user:

```sh
bash scripts/package-aur.sh \
  target/packages/chartr-0.3.0-linux-x86_64.tar.gz target/aur/chartr-bin
```

Use the actual archive version, including `-rc.N` for a candidate. The generator
uses that archive's SHA-256 checksum and a versioned GitHub release URL.
`makepkg --printsrcinfo` generates the metadata. The full packaging job tests
that same recipe with a local copy of the archive before it is publicly
available. Only `PKGBUILD` and `.SRCINFO` go into the AUR bundle.

## AUR publishing

The [Publish AUR workflow](../.github/workflows/aur.yml) updates `chartr-bin`
when a **stable GitHub release is published**. Creating a draft, pushing a tag,
or publishing an RC does not update AUR. Every x86_64 build still includes an
AUR recipe bundle for inspection or manual testing.

One-time setup:

1. Create an account at [aur.archlinux.org](https://aur.archlinux.org/) and
   check whether `chartr-bin` already exists. If another maintainer owns it,
   arrange co-maintainer access before enabling publication.
2. Create a dedicated, passphrase-free SSH key for release automation and add
   its **public** key to the AUR account. Keep its private key out of Git.
3. Store the private key as the GitHub Actions secret `AUR_SSH_PRIVATE_KEY` in
   `rengwu/chartr`, for example:

   ```sh
   gh secret set AUR_SSH_PRIVATE_KEY --repo rengwu/chartr < /path/to/aur-private-key
   ```

4. Store the verified SSH known-hosts entry for `aur.archlinux.org` in the
   Actions **variable** `AUR_SSH_KNOWN_HOSTS`. Compare the host-key fingerprint
   against the [AUR authentication documentation](https://wiki.archlinux.org/title/AUR_submission_guidelines#Authentication)
   before accepting it. The workflow requires strict host-key checking.
5. Merge the workflow into `main` before tagging the first release that should
   include AUR artifacts. Publish that release from its draft in the GitHub UI.

Missing publishing configuration fails with an explicit setup error; it does
not report a successful AUR publication. AUR receives packaging files only.
GitHub continues to host the binaries and checksums. The first successful push
creates the AUR package if it does not already exist.

Before pushing, the job verifies the AUR bundle's release checksum, checks that
its metadata names the exact published binary and checksum, downloads that
binary through `makepkg --verifysource`, and confirms `.SRCINFO` matches the
recipe. It never builds Chartr again. Publication is serialized, identical
retries do nothing, and older or changed same-version recipes cannot overwrite
the current AUR version. Updating to a new upstream version resets `pkgrel=1`.

To retry after correcting credentials or an interrupted run:

```sh
gh workflow run aur.yml --ref main -f release_tag=v0.3.0
```

Use a published stable tag containing the new AUR asset. Existing releases
without that asset cannot be published by this workflow. The dispatch validates
the release again, so it cannot publish a draft or RC. If another workflow
publishes a release using `GITHUB_TOKEN`, GitHub does not emit a new workflow run
for that event; explicitly dispatch `aur.yml` afterwards. See
[GitHub's event documentation](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows).

Arch packaging CI tests stable and RC versions using real `makepkg`, validates
checksums and installed files, and exercises first publication, updates,
retries, and downgrade rejection against a temporary local Git repository.

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

## External native plugins

Application builds contain the generic native surface host, without external
plugin libraries, engines or assets. Plugin authors publish prebuilt packages
from their own repositories. Application installation never builds plugins.
