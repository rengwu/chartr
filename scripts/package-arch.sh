#!/bin/bash
# Called inside an Arch Linux container as root; only repackages existing files.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
output="$root/target/packages"
version=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' "$root/Cargo.toml" | head -1)
archive="chartr-$version-linux-x86_64.tar.gz"
[[ -f "$output/$archive" ]]
pacman -Syu --noconfirm --needed base-devel gtk3 webkit2gtk-4.1 libxcb \
    libxkbcommon-x11 fontconfig vulkan-icd-loader xdg-utils
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cp "$output/$archive" "$work/"
checksum=$(sha256sum "$work/$archive" | cut -d ' ' -f1)
sed -e "s/^pkgver=.*/pkgver=$version/" \
    -e "s/^sha256sums=.*/sha256sums=('$checksum')/" \
    "$root/packaging/arch/PKGBUILD" > "$work/PKGBUILD"
useradd --no-create-home builder
chown -R builder:builder "$work"
# makepkg forbids running as root. The build user can only write its temp dir.
(cd "$work" && runuser -u builder -- env PKGEXT=.pkg.tar.zst makepkg --noconfirm)
cp "$work/chartr-$version-1-x86_64.pkg.tar.zst" "$output/"
(cd "$output" && sha256sum "chartr-$version-1-x86_64.pkg.tar.zst" >> SHA256SUMS-x86_64)
