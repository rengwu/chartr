#!/bin/bash
# Called inside an Arch Linux container as root; only repackages existing files.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
output="$root/target/packages"
version=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' "$root/Cargo.toml" | head -1)
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-rc\.[1-9][0-9]*)?$ ]] || {
    echo "expected a stable or release-candidate workspace version" >&2; exit 1;
}
# Keep rc immediately after the numeric version for pacman's prerelease ordering.
pkgver=${version/-rc./rc}
archive="chartr-$version-linux-x86_64.tar.gz"
[[ -f "$output/$archive" ]]
pacman -Syu --noconfirm --needed base-devel gtk3 webkit2gtk-4.1 libxcb \
    libxkbcommon-x11 fontconfig vulkan-icd-loader xdg-utils
if [[ "$version" == *-rc.* ]]; then
    [[ $(vercmp "$pkgver" "${version%-rc.*}") -lt 0 ]]
fi
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cp "$output/$archive" "$work/"
checksum=$(sha256sum "$work/$archive" | cut -d ' ' -f1)
sed -e "s/^pkgver=.*/pkgver=$pkgver/" \
    -e "s/^_upstream_version=.*/_upstream_version=$version/" \
    -e "s/^sha256sums=.*/sha256sums=('$checksum')/" \
    "$root/packaging/arch/PKGBUILD" > "$work/PKGBUILD"
useradd --no-create-home builder
chown -R builder:builder "$work"
# makepkg forbids running as root. The build user can only write its temp dir.
(cd "$work" && runuser -u builder -- env PKGEXT=.pkg.tar.zst makepkg --noconfirm)
cp "$work/chartr-$pkgver-1-x86_64.pkg.tar.zst" "$output/"
(cd "$output" && sha256sum "chartr-$pkgver-1-x86_64.pkg.tar.zst" >> SHA256SUMS-x86_64)
