#!/bin/sh
# Build an ad-hoc-signed macOS disk image for local testing.
#
# The application uses Cargo's release profile so the GPUI terminal is
# representative of a distributable build. Local packages use the production
# app name, with a separate bundle identifier, ad-hoc signing, and no notarization.

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

default_output="$root/target/chartr-$version-$revision-$architecture.dmg"
output=${1:-$default_output}
case "$output" in
/*) ;;
*) output="$root/$output" ;;
esac

echo "building chartr $version ($revision)"
cargo build --manifest-path "$root/Cargo.toml" --release -p chartr --locked

binary="$root/target/release/chartr"
sidecar="$root/target/release/herdr"
[ -x "$binary" ] && [ -x "$sidecar" ] || {
    echo "release build did not produce chartr and herdr" >&2
    exit 1
}

expected_herdr=$(sed -n \
    's/^pub const SUPPORTED_HERDR_VERSION: &str = "\(.*\)";$/\1/p' \
    "$root/crates/chartr-herdr/src/lib.rs")
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
app="$image_root/chartr.app"
macos="$app/Contents/MacOS"
resources="$app/Contents/Resources"
mkdir -p "$macos" "$resources"
ditto "$binary" "$macos/chartr"
ditto "$sidecar" "$macos/herdr"

# Match main's macOS artwork at each size. The small marks are drawn for
# their size bands; downscaling the large master loses those details.
iconset="$work/chartr.iconset"
mkdir -p "$iconset"
for spec in \
    "16 icon_16x16 16" \
    "32 icon_16x16@2x 32" \
    "32 icon_32x32 32" \
    "64 icon_32x32@2x 1024" \
    "128 icon_128x128 1024" \
    "256 icon_128x128@2x 1024" \
    "256 icon_256x256 1024" \
    "512 icon_256x256@2x 1024" \
    "512 icon_512x512 1024" \
    "1024 icon_512x512@2x 1024"
do
    set -- $spec
    sips -z "$1" "$1" "$root/docs/assets/v4/icon-mac-$3.png" \
        --out "$iconset/$2.png" >/dev/null
done
iconutil -c icns "$iconset" -o "$resources/chartr.icns"

plist="$app/Contents/Info.plist"
plutil -create xml1 "$plist"
plutil -insert CFBundleDevelopmentRegion -string en "$plist"
plutil -insert CFBundleDisplayName -string chartr "$plist"
plutil -insert CFBundleExecutable -string chartr "$plist"
plutil -insert CFBundleIconFile -string chartr.icns "$plist"
plutil -insert CFBundleIdentifier -string dev.chartr.dev "$plist"
plutil -insert CFBundleInfoDictionaryVersion -string 6.0 "$plist"
plutil -insert CFBundleName -string chartr "$plist"
plutil -insert CFBundlePackageType -string APPL "$plist"
plutil -insert CFBundleShortVersionString -string "$version" "$plist"
plutil -insert CFBundleVersion -string "$build_number" "$plist"
plutil -insert chartrGitRevision -string "$revision" "$plist"
plutil -insert LSApplicationCategoryType -string public.app-category.developer-tools "$plist"
plutil -insert NSHighResolutionCapable -bool YES "$plist"

minimum_macos=$(otool -l "$binary" | awk '$1 == "minos" { print $2; exit }')
[ -n "$minimum_macos" ] || minimum_macos=11.0
plutil -insert LSMinimumSystemVersion -string "$minimum_macos" "$plist"
plutil -lint "$plist"

codesign --force --sign - --timestamp=none \
    --identifier dev.chartr.dev.herdr "$macos/herdr"
codesign --force --sign - --timestamp=none \
    --identifier dev.chartr.dev "$macos/chartr"
codesign --force --sign - --timestamp=none \
    --identifier dev.chartr.dev "$app"
codesign --verify --deep --strict --verbose=2 "$app"

ln -s /Applications "$image_root/Applications"
temporary_dmg="$work/chartr.dmg"
hdiutil create -quiet -volname chartr -srcfolder "$image_root" \
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
