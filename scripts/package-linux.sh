#!/bin/bash
# Package an existing native Linux build. Never invokes Cargo or Zig.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
if (( $# > 2 )); then
    echo "usage: $0 [binary-directory] [output-directory]" >&2
    exit 2
fi
binaries=$(realpath "${1:-target/release}")
mkdir -p "${2:-target/packages}"
output=$(realpath "${2:-target/packages}")
version=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' Cargo.toml | head -1)
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-rc\.[1-9][0-9]*)?$ ]] || {
    echo "expected a stable or release-candidate workspace version" >&2; exit 1;
}
# Debian's tilde makes the candidate sort before the eventual stable release.
deb_version=${version/-rc./~rc.}
case "$(uname -m)" in
    x86_64) arch=x86_64; deb_arch=amd64; elf_machine='Advanced Micro Devices X86-64' ;;
    aarch64) arch=aarch64; deb_arch=arm64; elf_machine='AArch64' ;;
    *) echo "unsupported Linux architecture" >&2; exit 1 ;;
esac
expected=$(sed -n 's/^pub const SUPPORTED_HERDR_VERSION: &str = "\(.*\)";$/\1/p' crates/chartr-herdr/src/lib.rs)
[[ -x "$binaries/chartr" && -x "$binaries/herdr" ]] || {
    echo "missing chartr or herdr in $binaries" >&2; exit 1;
}
for binary in chartr herdr; do
    LC_ALL=C readelf -h "$binaries/$binary" | grep -F "$elf_machine" >/dev/null
    # ldd also reports missing symbol versions; reject either kind of ABI error.
    if ! linkage=$(LC_ALL=C ldd "$binaries/$binary" 2>&1); then
        echo "$linkage" >&2; exit 1
    fi
    if [[ "$linkage" == *'not found'* ]]; then
        echo "$linkage" >&2; exit 1
    fi
done
[[ "$("$binaries/herdr" --version)" == "herdr $expected" ]] || {
    echo "Herdr does not match the pinned version $expected" >&2; exit 1;
}
name="chartr-$version-linux-$arch"
stage=$(mktemp -d "$output/stage.XXXXXX")
trap 'rm -rf "$stage"' EXIT
bundle="$stage/$name"
install -Dm755 "$binaries/chartr" "$bundle/usr/lib/chartr/chartr"
install -Dm755 "$binaries/herdr" "$bundle/usr/lib/chartr/herdr"
# Strip only packaged copies. Keep Cargo's artifacts intact for reuse.
strip --strip-unneeded "$bundle/usr/lib/chartr/chartr" "$bundle/usr/lib/chartr/herdr"
mkdir -p "$bundle/usr/bin"
ln -s ../lib/chartr/chartr "$bundle/usr/bin/chartr"
install -Dm644 packaging/linux/chartr.desktop "$bundle/usr/share/applications/chartr.desktop"
install -Dm644 docs/assets/v4/icon-mac-1024.png "$bundle/usr/share/icons/hicolor/1024x1024/apps/chartr.png"
install -Dm644 LICENSE-GPL "$bundle/usr/share/licenses/chartr/LICENSE-GPL"
install -Dm644 vendor/herdr/LICENSE "$bundle/usr/share/licenses/chartr/LICENSE-Herdr"
cp packaging/linux/INSTALL.txt "$bundle/INSTALL.txt"
{
    echo "chartr: $version"
    echo "commit: $(git rev-parse HEAD)"
    echo "Herdr: $expected"
    echo "architecture: $arch"
    echo "Rust: $(rustc --version)"
    cat /etc/os-release
} > "$bundle/BUILD-INFO.txt"
# Use fast compression; packaging should not dominate a warm release build.
tar -C "$stage" -cf - "$name" | gzip -1 > "$output/$name.tar.gz"

# Let Debian derive versioned ELF dependencies from the binaries. Also list
# runtime libraries loaded dynamically, which dpkg-shlibdeps cannot discover.
mkdir -p "$stage/debian" "$bundle/DEBIAN"
cat > "$stage/debian/control" <<CONTROL
Source: chartr
Section: utils
Priority: optional
Maintainer: chartr maintainers <noreply@github.com>

Package: chartr
Architecture: any
Description: Agent multiplexer
CONTROL
deps=$(cd "$stage" && dpkg-shlibdeps -O \
    -e"$bundle/usr/lib/chartr/chartr" -e"$bundle/usr/lib/chartr/herdr")
deps=${deps#shlibs:Depends=}
cat > "$bundle/DEBIAN/control" <<CONTROL
Package: chartr
Version: $deb_version
Architecture: $deb_arch
Maintainer: chartr maintainers <noreply@github.com>
Section: utils
Priority: optional
Depends: $deps, libvulkan1, libxkbcommon-x11-0, libfontconfig1, xdg-utils
Recommends: xwayland
Homepage: https://github.com/rengwu/chartr
Description: Agent multiplexer with persistent terminals
 Native desktop application with private Herdr runtime.
CONTROL
# Package only the installation tree; archive instructions remain in the tarball.
rm "$bundle/INSTALL.txt"
install -Dm644 "$bundle/BUILD-INFO.txt" "$bundle/usr/share/doc/chartr/BUILD-INFO.txt"
rm "$bundle/BUILD-INFO.txt"
# GitHub normalizes '~' in asset names, so use the Cargo version in filenames.
# The package's internal Debian version above retains '~rc.N' for ordering.
dpkg-deb --root-owner-group -Zgzip -z1 --build "$bundle" "$output/chartr_${version}_${deb_arch}.deb"
(
    cd "$output"
    sha256sum "$name.tar.gz" "chartr_${version}_${deb_arch}.deb" > "SHA256SUMS-$arch"
)
echo "Packages written to $output"
