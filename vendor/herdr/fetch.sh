#!/bin/sh
# Vendors the herdr executable zeddy ships as its backend.
#
# A maintenance step, run by hand when the pin moves — never by a build. This is
# the only thing in zeddy that reaches the network, and it is not on the path of
# `cargo build`.
#
#     sh vendor/herdr/fetch.sh              # this machine's target
#     sh vendor/herdr/fetch.sh <triple>…    # named targets
#
# Each executable lands at `vendor/herdr/<triple>/herdr`, which is gitignored:
# they belong to herdr, not to this history. `crates/zeddy/build.rs` copies the
# one for the target being built in beside the zeddy binary.
#
# The version is not a flag. It is read from `SUPPORTED_HERDR_VERSION`, the one
# place zeddy pins herdr, so a vendored binary and the client that drives it
# cannot disagree about which release this is.

set -eu

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)

version=$(sed -n 's/^pub const SUPPORTED_HERDR_VERSION: &str = "\(.*\)";$/\1/p' \
    "$root/crates/zeddy-herdr/src/lib.rs")
[ -n "$version" ] || {
    echo "cannot read SUPPORTED_HERDR_VERSION from crates/zeddy-herdr/src/lib.rs" >&2
    exit 1
}

# herdr publishes no Windows build, and `zeddy-herdr` does not compile there
# either — its control plane is a Unix domain socket.
asset_for() {
    case "$1" in
    aarch64-apple-darwin) echo "herdr-macos-aarch64" ;;
    x86_64-apple-darwin) echo "herdr-macos-x86_64" ;;
    aarch64-unknown-linux-gnu) echo "herdr-linux-aarch64" ;;
    x86_64-unknown-linux-gnu) echo "herdr-linux-x86_64" ;;
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

for target in $targets; do
    asset=$(asset_for "$target") || {
        echo "no herdr release asset for $target" >&2
        exit 1
    }
    dir="$here/$target"
    mkdir -p "$dir"
    url="https://github.com/herdrdev/herdr/releases/download/v$version/$asset"
    echo "fetching herdr $version for $target"
    curl -fsSL "$url" -o "$dir/herdr"
    chmod +x "$dir/herdr"
done

curl -fsSL "https://raw.githubusercontent.com/herdrdev/herdr/v$version/LICENSE" \
    -o "$here/LICENSE"
echo "vendored herdr $version"
