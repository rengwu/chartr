#!/usr/bin/env bash
# Install a native Linux package on the distribution family it targets, then
# prove that the installed desktop entry starts a shell which renders chartr.
# This catches three different release failures in one place: bad package
# metadata, a binary linked against the wrong distro ABI, and a WebKit window
# which starts but renders its internal error page.
#
# Usage: scripts/smoke-native-package.sh <deb|rpm> <package> <version>
set -euo pipefail

FORMAT="${1:?usage: smoke-native-package.sh <deb|rpm> <package> <version>}"
PACKAGE="${2:?usage: smoke-native-package.sh <deb|rpm> <package> <version>}"
VERSION="${3:?usage: smoke-native-package.sh <deb|rpm> <package> <version>}"
[ -f "$PACKAGE" ] || { echo "smoke: no such package: $PACKAGE" >&2; exit 1; }

case "$FORMAT" in
  deb)
    IMAGE="chartr-deb-smoke"
    CONTAINER_PACKAGE="/chartr.deb"
    docker build -q -t "$IMAGE" - <<'DOCKERFILE'
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
      xvfb dbus dbus-x11 ca-certificates curl imagemagick \
      libwebkit2gtk-4.1-0 \
  && rm -rf /var/lib/apt/lists/*
DOCKERFILE
    ;;
  rpm)
    IMAGE="chartr-rpm-smoke"
    CONTAINER_PACKAGE="/chartr.rpm"
    docker build -q -t "$IMAGE" - <<'DOCKERFILE'
FROM fedora:44
RUN dnf install -y \
      xorg-x11-server-Xvfb dbus-daemon curl ImageMagick webkit2gtk4.1 \
  && dnf clean all
DOCKERFILE
    ;;
  *)
    echo "smoke: format must be deb or rpm (got $FORMAT)" >&2
    exit 2
    ;;
esac

docker run --rm \
  --security-opt seccomp=unconfined \
  -e PACKAGE_FORMAT="$FORMAT" \
  -e EXPECTED_VERSION="$VERSION" \
  -v "$(realpath "$PACKAGE"):$CONTAINER_PACKAGE:ro" \
  "$IMAGE" bash -euo pipefail -c '
	if [ "$PACKAGE_FORMAT" = deb ]; then
		PACKAGE=/chartr.deb
		test "$(dpkg-deb -f "$PACKAGE" Package)" = chartr
		test "$(dpkg-deb -f "$PACKAGE" Architecture)" = "$(dpkg --print-architecture)"
		dpkg-deb -f "$PACKAGE" Depends | grep -Eq "(^|, )libc6 \(>= 2\.39\)(,|$)"
		dpkg-deb -f "$PACKAGE" Depends | grep -Eq "(^|, )libwebkit2gtk-4.1-0(,|$)"
		dpkg -i "$PACKAGE"
	else
		PACKAGE=/chartr.rpm
		test "$(rpm -qp --qf "%{NAME}" "$PACKAGE")" = chartr
		test "$(rpm -qp --qf "%{ARCH}" "$PACKAGE")" = "$(rpm --eval "%{_arch}")"
		rpm -qpR "$PACKAGE" | grep -Eq "^glibc[[:space:]]+>=[[:space:]]+2\.39$"
		rpm -qpR "$PACKAGE" | grep -qx webkit2gtk4.1
		rpm -i "$PACKAGE"
	fi

	test -x /usr/bin/chartr
	test -f /usr/share/applications/io.github.rengwu.chartr.desktop
	test -f /usr/share/metainfo/io.github.rengwu.chartr.metainfo.xml
	test -f /usr/share/icons/hicolor/512x512/apps/chartr.png
	chartr --version | grep -F "chartr shell ${EXPECTED_VERSION} "

	export XDG_RUNTIME_DIR=/tmp/xdg
	mkdir -p "$XDG_RUNTIME_DIR" /tmp/dd
	chmod 700 "$XDG_RUNTIME_DIR"
	export DISPLAY=:99

	Xvfb :99 -screen 0 1280x840x24 >/dev/null 2>&1 &
	sleep 2

	chartr --data-dir /tmp/dd > /tmp/run.log 2>&1 &
	APP_PID=$!

	fail() {
		echo "SMOKE FAILED: $*" >&2
		echo "--- app output ---" >&2
		cat /tmp/run.log >&2
		exit 1
	}

	for _ in $(seq 1 30); do
		[ -f /tmp/dd/.chartr/shell.lock ] && break
		kill -0 "$APP_PID" 2>/dev/null || fail "the installed shell exited instead of staying up"
		sleep 0.5
	done
	sleep 10

	kill -0 "$APP_PID" 2>/dev/null || fail "the installed shell exited instead of staying up"
	LOCK=/tmp/dd/.chartr/shell.lock
	[ -f "$LOCK" ] || fail "no single-instance lock written; the server never bound"
	URL=$(grep -o "http://[^\"]*" "$LOCK")
	curl -fsS -o /tmp/page.html "$URL/" || fail "the cockpit did not serve over loopback"
	grep -qi "<title>chartr</title>" /tmp/page.html || fail "loopback served something other than chartr"

	if command -v magick >/dev/null 2>&1; then
		magick import -display :99 -window root /tmp/shot.png
		MEAN=$(magick /tmp/shot.png -colorspace Gray -format "%[fx:int(mean*255)]" info:)
		COLOURS=$(magick /tmp/shot.png -format %k info:)
	else
		import -display :99 -window root /tmp/shot.png
		MEAN=$(convert /tmp/shot.png -colorspace Gray -format "%[fx:int(mean*255)]" info:)
		COLOURS=$(convert /tmp/shot.png -format %k info:)
	fi
	echo "screenshot: mean brightness ${MEAN}/255 across ${COLOURS} colours"
	[ "$MEAN" -le 100 ] || fail "the window is bright (${MEAN}/255); WebKit likely rendered its error page"
	[ "$COLOURS" -ge 50 ] || fail "the window is nearly featureless (${COLOURS} colours)"
	! grep -q "WebKit encountered an internal error" /tmp/run.log || fail "WebKit reported an internal error"

	kill "$APP_PID" 2>/dev/null || true
	echo "SMOKE PASSED: the $PACKAGE_FORMAT package installed and rendered chartr $EXPECTED_VERSION"
'
