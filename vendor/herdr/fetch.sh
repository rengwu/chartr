#!/bin/sh
# Vendors the herdr executable zeddy ships as its backend.
#
# A maintenance step, run by hand when the pin moves — never by a build. This is
# the only thing in zeddy that reaches the network, and it is not on the path of
# `cargo build`.
#
#     sh vendor/herdr/fetch.sh              # this machine's target
#     sh vendor/herdr/fetch.sh <triple>…    # named targets (cross toolchain required)
#
# Each executable lands at `vendor/herdr/<triple>/herdr`, which is gitignored:
# they belong to herdr, not to this history. `crates/zeddy/build.rs` copies the
# one for the target being built in beside the zeddy binary.
#
# Herdr's latest release predates semantic mouse forwarding for direct attach.
# Until that change is tagged, this builds one immutable upstream revision and
# brands it with a Zeddy-specific version. The runtime handshake therefore
# rejects both an older release binary and an arbitrary build of the same
# upstream package version.

set -eu

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)

version=$(sed -n 's/^pub const SUPPORTED_HERDR_VERSION: &str = "\(.*\)";$/\1/p' \
    "$root/crates/zeddy-herdr/src/lib.rs")
upstream_version=$(sed -n \
    's/^pub const SUPPORTED_HERDR_UPSTREAM_VERSION: &str = "\(.*\)";$/\1/p' \
    "$root/crates/zeddy-herdr/src/lib.rs")
revision=$(sed -n \
    's/^pub const SUPPORTED_HERDR_REVISION: &str = "\([0-9a-f][0-9a-f]*\)";$/\1/p' \
    "$root/crates/zeddy-herdr/src/lib.rs")
[ -n "$version" ] && [ -n "$upstream_version" ] && [ -n "$revision" ] || {
    echo "cannot read the Herdr pins from crates/zeddy-herdr/src/lib.rs" >&2
    exit 1
}

# Herdr does not support Zeddy's Windows control plane; keep accepted targets
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

targets=${*:-$(host_target)}

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

work=$(mktemp -d "${TMPDIR:-/tmp}/zeddy-herdr.XXXXXX")
trap 'rm -rf "$work"' EXIT HUP INT TERM
source_dir="$work/source"
mkdir -p "$source_dir"

echo "fetching Herdr source $revision"
curl -fsSL "https://github.com/herdrdev/herdr/archive/$revision.tar.gz" \
    -o "$work/herdr.tar.gz"
tar -xzf "$work/herdr.tar.gz" -C "$source_dir" --strip-components=1

actual_upstream_version=$(sed -n \
    's/^version = "\(.*\)"$/\1/p' "$source_dir/Cargo.toml" | head -1)
[ "$actual_upstream_version" = "$upstream_version" ] || {
    echo "revision $revision is Herdr $actual_upstream_version, expected $upstream_version" >&2
    exit 1
}

build_id=$(printf '%s' "$revision" | cut -c1-12)
for target in $targets; do
    supported_target "$target" || {
        echo "unsupported Herdr target: $target" >&2
        exit 1
    }

    echo "building Herdr $version for $target"
    (
        cd "$source_dir"
        CARGO_TARGET_DIR="$work/target" \
            HERDR_BUILD_CHANNEL=zeddy \
            HERDR_BUILD_ID="$build_id" \
            HERDR_BUILD_COMMIT="$revision" \
            ZIG="$zig_bin" \
            cargo build --release --locked --target "$target"
    )

    dir="$here/$target"
    mkdir -p "$dir"
    cp "$work/target/$target/release/herdr" "$dir/herdr"
    chmod +x "$dir/herdr"
done

curl -fsSL "https://raw.githubusercontent.com/herdrdev/herdr/$revision/LICENSE" \
    -o "$here/LICENSE"
echo "vendored Herdr $version from $revision"
