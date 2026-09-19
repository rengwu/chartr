#!/bin/bash
# Called inside an Arch Linux container as root; only repackages existing files.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
output="$root/target/packages"
version=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' "$root/Cargo.toml" | head -1)
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-rc\.[1-9][0-9]*)?$ ]] || {
    echo "expected a stable or release-candidate workspace version" >&2; exit 1;
}
pkgver=${version/-rc./rc}
archive="chartr-$version-linux-x86_64.tar.gz"
[[ -f "$output/$archive" ]]
pacman -Syu --noconfirm --needed base-devel gtk3 webkit2gtk-4.1 libxcb \
    libxkbcommon-x11 fontconfig vulkan-icd-loader xdg-utils
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cp "$output/$archive" "$work/"
useradd --no-create-home builder
chown -R builder:builder "$work"
runuser -u builder -- bash "$root/scripts/package-aur.sh" "$work/$archive" "$work/aur"
# makepkg uses this local copy before trying the public release URL. The draft
# release does not exist yet, but its published AUR recipe must use that URL.
cp "$work/$archive" "$work/aur/"
chown builder:builder "$work/aur/$archive"
# makepkg forbids running as root. The build user can only write its temp dir.
(cd "$work/aur" && runuser -u builder -- env PKGEXT=.pkg.tar.zst makepkg --noconfirm)
package="chartr-bin-$pkgver-1-x86_64.pkg.tar.zst"
recipe="chartr-$version-aur.tar.gz"
cp "$work/aur/$package" "$output/"
# Publish only the recipe and metadata, never binaries or makepkg output to AUR.
tar -C "$work/aur" -czf "$output/$recipe" PKGBUILD .SRCINFO
(cd "$output" && sha256sum "$package" "$recipe" >> SHA256SUMS-x86_64)
