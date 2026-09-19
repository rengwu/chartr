#!/bin/bash
# Update a cloned AUR repository using a verified recipe. Never force-push.
set -euo pipefail
if (( $# != 2 )); then
    echo "usage: $0 <recipe-directory> <aur-checkout>" >&2
    exit 2
fi
recipe=$(realpath "$1")
checkout=$(realpath "$2")
field() { awk -v key="$1" '$1 == key && $2 == "=" { print $3 }' "$2"; }
[[ $(field pkgbase "$recipe/.SRCINFO") == chartr-bin ]]
[[ $(field pkgname "$recipe/.SRCINFO") == chartr-bin ]]
version=$(field pkgver "$recipe/.SRCINFO")
release=$(field pkgrel "$recipe/.SRCINFO")
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ && "$release" =~ ^[1-9][0-9]*$ ]] || {
    echo "only stable versions may be published to chartr-bin" >&2; exit 1;
}
cd "$checkout"
[[ -z $(git status --porcelain) ]] || {
    echo "AUR checkout must be clean" >&2; exit 1;
}
if [[ -f .SRCINFO ]]; then
    [[ $(field pkgbase .SRCINFO) == chartr-bin ]]
    if cmp -s "$recipe/PKGBUILD" PKGBUILD && cmp -s "$recipe/.SRCINFO" .SRCINFO; then
        echo "AUR already contains this recipe"
        exit 0
    fi
    previous="$(field pkgver .SRCINFO)-$(field pkgrel .SRCINFO)"
    [[ $(vercmp "$version-$release" "$previous") -gt 0 ]] || {
        echo "refusing to overwrite AUR $previous with $version-$release" >&2; exit 1;
    }
elif [[ -n $(git ls-files) ]]; then
    echo "existing AUR repository has no .SRCINFO" >&2; exit 1
fi
cp "$recipe/PKGBUILD" "$recipe/.SRCINFO" .
git add -- PKGBUILD .SRCINFO
git -c user.name='chartr release automation' \
    -c user.email='noreply@github.com' commit -m "Update chartr-bin to $version-$release"
git push origin HEAD:master
