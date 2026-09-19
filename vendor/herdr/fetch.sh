#!/bin/sh
# Vendors the herdr executable chartr ships as its backend.
#
# Run explicitly or by CI, never from `cargo build`. An already matching native
# executable is reused without requiring Zig. Set HERDR_REBUILD=1 to rebuild it.
# Source and compilation outputs survive under .build/ for subsequent builds.
#
#     sh vendor/herdr/fetch.sh              # this machine's target
#     sh vendor/herdr/fetch.sh <triple>…    # named targets (cross toolchain required)
#
# Each executable lands at `vendor/herdr/<triple>/herdr`, which is gitignored:
# they belong to herdr, not to this history. `crates/chartr/build.rs` copies the
# one for the target being built in beside the chartr binary.
#
# Herdr's latest release predates semantic mouse forwarding for direct attach.
# Until that change is tagged, this builds one immutable upstream revision and
# brands it with a chartr-specific version. The runtime handshake therefore
# rejects both an older release binary and an arbitrary build of the same
# upstream package version.

set -eu

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)

version=$(sed -n 's/^pub const SUPPORTED_HERDR_VERSION: &str = "\(.*\)";$/\1/p' \
    "$root/crates/chartr-herdr/src/lib.rs")
upstream_version=$(sed -n \
    's/^pub const SUPPORTED_HERDR_UPSTREAM_VERSION: &str = "\(.*\)";$/\1/p' \
    "$root/crates/chartr-herdr/src/lib.rs")
revision=$(sed -n \
    's/^pub const SUPPORTED_HERDR_REVISION: &str = "\([0-9a-f][0-9a-f]*\)";$/\1/p' \
    "$root/crates/chartr-herdr/src/lib.rs")
[ -n "$version" ] && [ -n "$upstream_version" ] && [ -n "$revision" ] || {
    echo "cannot read the Herdr pins from crates/chartr-herdr/src/lib.rs" >&2
    exit 1
}

# Herdr does not support chartr's Windows control plane; keep accepted targets
# explicit so a typo cannot silently produce a sidecar in the wrong directory.
supported_target() {
    case "$1" in
    aarch64-apple-darwin | x86_64-apple-darwin | \
        aarch64-unknown-linux-gnu | x86_64-unknown-linux-gnu) return 0 ;;
    *) return 1 ;;
    esac
}

host_target() {
    machine=$(uname -m)
    case "$(uname -s)" in
    Darwin) case "$machine" in arm64) echo "aarch64-apple-darwin" ;; *) echo "x86_64-apple-darwin" ;; esac ;;
    Linux) case "$machine" in aarch64) echo "aarch64-unknown-linux-gnu" ;; *) echo "x86_64-unknown-linux-gnu" ;; esac ;;
    *) echo "unsupported host: $(uname -s)" >&2; exit 1 ;;
    esac
}

# macOS may reject a restored/copied linker signature before --version runs.
sign_native_sidecar() {
    case "$host" in
    *-apple-darwin) codesign --force --sign - "$1" ;;
    esac
}

host=$(host_target)
targets=${*:-$host}
pending=""
for target in $targets; do
    supported_target "$target" || {
        echo "unsupported Herdr target: $target" >&2
        exit 1
    }
    binary="$here/$target/herdr"
    if [ "$target" = "$host" ] && [ -x "$binary" ]; then
        sign_native_sidecar "$binary"
    fi
    if [ "${HERDR_REBUILD:-0}" != 1 ] && [ "$target" = "$host" ] && [ -x "$binary" ] &&
        [ "$("$binary" --version 2>/dev/null || true)" = "herdr $version" ]; then
        echo "reusing Herdr $version for $target"
    else
        pending="$pending $target"
    fi
done
[ -n "$pending" ] || exit 0

# Do not accidentally build with the user's default Rust toolchain after cd.
toolchain=$(sed -n 's/^channel = "\([^" ]*\)"$/\1/p' "$root/rust-toolchain.toml")
[ -n "$toolchain" ] || { echo "cannot read pinned Rust toolchain" >&2; exit 1; }

zig_bin=${ZIG:-}
if [ -z "$zig_bin" ] && command -v brew >/dev/null 2>&1; then
    brew_zig=$(brew --prefix zig@0.15 2>/dev/null || true)
    if [ -x "$brew_zig/bin/zig" ]; then
        zig_bin="$brew_zig/bin/zig"
    fi
fi
if [ -z "$zig_bin" ] && command -v zig >/dev/null 2>&1; then
    zig_bin=$(command -v zig)
fi
[ -n "$zig_bin" ] || {
    echo "building Herdr requires Zig 0.15.2 (set ZIG to its executable)" >&2
    exit 1
}
case $("$zig_bin" version) in
0.15.2) ;;
*)
    echo "building Herdr requires Zig 0.15.2; $zig_bin is $("$zig_bin" version)" >&2
    exit 1
    ;;
esac

work="$here/.build"
source_dir="$work/source/$revision"
mkdir -p "$work/source"
if [ ! -d "$source_dir" ]; then
    download=$(mktemp -d "$work/source/download.XXXXXX")
    trap 'rm -rf "$download"' EXIT
    trap 'exit 1' HUP INT TERM
    echo "fetching Herdr source $revision"
    curl --retry 3 -fsSL "https://github.com/herdrdev/herdr/archive/$revision.tar.gz" \
        -o "$download/herdr.tar.gz"
    mkdir "$download/source"
    tar -xzf "$download/herdr.tar.gz" -C "$download/source" --strip-components=1
    mv "$download/source" "$source_dir"
    rm -rf "$download"
    trap - EXIT HUP INT TERM
fi

actual_upstream_version=$(sed -n \
    's/^version = "\(.*\)"$/\1/p' "$source_dir/Cargo.toml" | head -1)
[ "$actual_upstream_version" = "$upstream_version" ] || {
    echo "revision $revision is Herdr $actual_upstream_version, expected $upstream_version" >&2
    exit 1
}

build_id=$(printf '%s' "$revision" | cut -c1-12)
for target in $pending; do
    echo "building Herdr $version for $target"
    (
        cd "$source_dir"
        CARGO_TARGET_DIR="$work/target" \
            HERDR_BUILD_CHANNEL=chartr \
            HERDR_BUILD_ID="$build_id" \
            HERDR_BUILD_COMMIT="$revision" \
            ZIG="$zig_bin" \
            cargo "+$toolchain" build --release --locked --target "$target" --timings
    )

    dir="$here/$target"
    mkdir -p "$dir"
    # Publish only a complete executable; preserve the old one if a build fails.
    cp "$work/target/$target/release/herdr" "$dir/herdr.tmp"
    chmod +x "$dir/herdr.tmp"
    if [ "$target" = "$host" ]; then
        sign_native_sidecar "$dir/herdr.tmp"
        [ "$("$dir/herdr.tmp" --version)" = "herdr $version" ] || {
            echo "built Herdr does not match $version" >&2
            rm -f "$dir/herdr.tmp"
            exit 1
        }
    fi
    mv -f "$dir/herdr.tmp" "$dir/herdr"
done

cp "$source_dir/LICENSE" "$here/LICENSE"
echo "vendored Herdr $version from $revision"
