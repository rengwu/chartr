#!/bin/sh
# Build an ad-hoc-signed macOS disk image for local testing.
#
# The application uses Cargo's release profile so the GPUI terminal is
# representative of a distributable build. "Dev" describes the package:
# it has a separate bundle identifier, an ad-hoc signature, and no notarization.

set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
root=$(cd "$script_dir/.." && pwd)

if [ "$#" -gt 1 ]; then
    echo "usage: $0 [output.dmg]" >&2
    exit 2
fi

version=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' "$root/Cargo.toml" | head -1)
[ -n "$version" ] || {
    echo "cannot read the workspace version from Cargo.toml" >&2
    exit 1
}

revision=$(git -C "$root" rev-parse --short=12 HEAD)
build_number=$(git -C "$root" rev-list --count HEAD)
architecture=$(uname -m)
case "$architecture" in
arm64 | x86_64) ;;
*)
    echo "unsupported macOS architecture: $architecture" >&2
    exit 1
    ;;
esac

default_output="$root/target/Chartr-dev-$version-$revision-$architecture.dmg"
output=${1:-$default_output}
case "$output" in
/*) ;;
*) output="$root/$output" ;;
esac

echo "building Chartr $version ($revision)"
cargo build --manifest-path "$root/Cargo.toml" --release -p zeddy --locked

binary="$root/target/release/zeddy"
sidecar="$root/target/release/herdr"
[ -x "$binary" ] && [ -x "$sidecar" ] || {
    echo "release build did not produce zeddy and herdr" >&2
    exit 1
}

expected_herdr=$(sed -n \
    's/^pub const SUPPORTED_HERDR_VERSION: &str = "\(.*\)";$/\1/p' \
    "$root/crates/zeddy-herdr/src/lib.rs")
actual_herdr=$("$sidecar" --version | sed 's/^herdr //')
[ "$actual_herdr" = "$expected_herdr" ] || {
    echo "Herdr sidecar is $actual_herdr; expected $expected_herdr" >&2
    exit 1
}

mkdir -p "$root/target"
work=$(mktemp -d "$root/target/dev-dmg-stage.XXXXXX")
trap 'rm -rf "$work"' EXIT
trap 'exit 1' HUP INT TERM

image_root="$work/image"
app="$image_root/Chartr Dev.app"
macos="$app/Contents/MacOS"
resources="$app/Contents/Resources"
mkdir -p "$macos" "$resources"
ditto "$binary" "$macos/Chartr"
ditto "$sidecar" "$macos/herdr"

plist="$app/Contents/Info.plist"
plutil -create xml1 "$plist"
plutil -insert CFBundleDevelopmentRegion -string en "$plist"
plutil -insert CFBundleDisplayName -string "Chartr Dev" "$plist"
plutil -insert CFBundleExecutable -string Chartr "$plist"
plutil -insert CFBundleIdentifier -string dev.chartr.zeddy.dev "$plist"
plutil -insert CFBundleInfoDictionaryVersion -string 6.0 "$plist"
plutil -insert CFBundleName -string "Chartr Dev" "$plist"
plutil -insert CFBundlePackageType -string APPL "$plist"
plutil -insert CFBundleShortVersionString -string "$version" "$plist"
plutil -insert CFBundleVersion -string "$build_number" "$plist"
plutil -insert ChartrGitRevision -string "$revision" "$plist"
plutil -insert LSApplicationCategoryType -string public.app-category.developer-tools "$plist"
plutil -insert NSHighResolutionCapable -bool YES "$plist"

minimum_macos=$(otool -l "$binary" | awk '$1 == "minos" { print $2; exit }')
[ -n "$minimum_macos" ] || minimum_macos=11.0
plutil -insert LSMinimumSystemVersion -string "$minimum_macos" "$plist"
plutil -lint "$plist"

codesign --force --sign - --timestamp=none \
    --identifier dev.chartr.zeddy.dev.herdr "$macos/herdr"
codesign --force --sign - --timestamp=none \
    --identifier dev.chartr.zeddy.dev "$macos/Chartr"
codesign --force --sign - --timestamp=none \
    --identifier dev.chartr.zeddy.dev "$app"
codesign --verify --deep --strict --verbose=2 "$app"

ln -s /Applications "$image_root/Applications"
temporary_dmg="$work/Chartr-dev.dmg"
hdiutil create -quiet -volname "Chartr Dev $version" -srcfolder "$image_root" \
    -fs HFS+ -format UDZO "$temporary_dmg"
hdiutil verify "$temporary_dmg"

mkdir -p "$(dirname "$output")"
mv -f "$temporary_dmg" "$output"
(
    cd "$(dirname "$output")"
    shasum -a 256 "$(basename "$output")"
) > "$output.sha256"

echo "built $output"
echo "checksum: $output.sha256"
