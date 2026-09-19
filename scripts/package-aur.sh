#!/bin/bash
# Generate an AUR recipe from the exact release archive; requires Arch makepkg.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
if (( $# != 2 )); then
    echo "usage: $0 <release-archive> <recipe-directory>" >&2
    exit 2
fi
archive=$(realpath "$1")
name=$(basename "$archive")
[[ -f "$archive" && "$name" =~ ^chartr-([0-9]+\.[0-9]+\.[0-9]+(-rc\.[1-9][0-9]*)?)-linux-x86_64\.tar\.gz$ ]] || {
    echo "expected an x86_64 stable or release-candidate archive" >&2; exit 1;
}
version=${BASH_REMATCH[1]}
pkgver=${version/-rc./rc}
if [[ "$version" == *-rc.* ]]; then
    [[ $(vercmp "$pkgver" "${version%-rc.*}") -lt 0 ]]
fi
checksum=$(sha256sum "$archive" | cut -d ' ' -f1)
mkdir -p "$2"
output=$(realpath "$2")
sed -e "s/^pkgver=.*/pkgver=$pkgver/" \
    -e "s/^_upstream_version=.*/_upstream_version=$version/" \
    -e "s/^sha256sums=.*/sha256sums=('$checksum')/" \
    "$root/packaging/arch/PKGBUILD" > "$output/PKGBUILD"
(cd "$output" && makepkg --printsrcinfo > .SRCINFO)
