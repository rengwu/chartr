#!/usr/bin/env bash
# Prove a built AppImage actually runs the cockpit on a machine that has no
# WebKit and no GTK on it.
#
# The bar this has to clear is set by the failure it exists to catch. When the
# bundled WebKit cannot spawn its helper processes, the shell does *not* exit
# non-zero and does not log a fatal error: the process stays up, the Go server
# binds its port and serves the SPA over loopback perfectly happily, and the
# window renders "WebKit encountered an internal error" on a white page. Every
# cheap check passes. So this script does not ask whether the app started, it
# asks what is on the screen -- an Xvfb screenshot, tested for the cockpit's
# near-black olive background rather than the error page's white one. That is
# the assertion; the rest are here to fail earlier with a clearer message.
#
# Usage: scripts/smoke-appimage.sh <path-to-.AppImage>
set -euo pipefail

APPIMAGE="${1:?usage: smoke-appimage.sh <path-to-.AppImage>}"
[ -f "$APPIMAGE" ] || { echo "smoke: no such AppImage: $APPIMAGE" >&2; exit 1; }

# The container is deliberately a *minimal desktop*, not an empty box. The three
# libraries installed here are the ones AppImage expects every host to provide
# because they are bound to things we must not bundle: the GPU driver
# (libEGL/libGLESv2), the operator's own font configuration (libfontconfig) and
# their compositor's protocol (libwayland-client). Everything else the app needs
# -- all of WebKitGTK, all of GTK -- must come out of the AppImage, and the
# absence of any webkit or gtk package here is what makes that a real test.
IMAGE="chartr-appimage-smoke"
docker build -q -t "$IMAGE" - <<'DOCKERFILE'
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
      xvfb dbus dbus-x11 ca-certificates curl \
      libegl1 libgles2 libfontconfig1 libwayland-client0 \
      imagemagick xdotool x11-utils \
  && rm -rf /var/lib/apt/lists/* \
  && ! dpkg -l | grep -qiE "webkit|libgtk-3"
DOCKERFILE

docker run --rm -v "$(realpath "$APPIMAGE")":/chartr.AppImage:ro "$IMAGE" bash -euo pipefail -c '
	cp /chartr.AppImage /tmp/chartr.AppImage && chmod +x /tmp/chartr.AppImage

	# No FUSE in a container, so the AppImage self-extracts instead of mounting.
	# This is the runtime saying so, not us reaching inside the artifact.
	export APPIMAGE_EXTRACT_AND_RUN=1
	export XDG_RUNTIME_DIR=/tmp/xdg && mkdir -p "$XDG_RUNTIME_DIR" && chmod 700 "$XDG_RUNTIME_DIR"
	mkdir -p /tmp/dd
	export DISPLAY=:99

	Xvfb :99 -screen 0 1280x840x24 >/dev/null 2>&1 &
	sleep 2

	/tmp/chartr.AppImage --data-dir /tmp/dd > /tmp/run.log 2>&1 &
	APP_PID=$!
	sleep 14

	fail() { echo "SMOKE FAILED: $*" >&2; echo "--- app output ---" >&2; cat /tmp/run.log >&2; exit 1; }

	kill -0 "$APP_PID" 2>/dev/null || fail "the shell exited instead of staying up"

	LOCK=/tmp/dd/.chartr/shell.lock
	[ -f "$LOCK" ] || fail "no single-instance lock written; the server never bound"
	URL=$(grep -o "http://[^\"]*" "$LOCK")

	curl -fsS -o /tmp/page.html "$URL/" || fail "the cockpit did not serve over loopback"
	grep -qi "<title>chartr</title>" /tmp/page.html || fail "loopback served something that is not the cockpit"

	# The real assertion. A working cockpit fills the window with its near-black
	# olive background; the WebKit error page is white text-on-white. Mean
	# brightness separates them by a mile, so the threshold does not need to be
	# delicate.
	import -display :99 -window root /tmp/shot.png
	MEAN=$(convert /tmp/shot.png -colorspace Gray -format "%[fx:int(mean*255)]" info:)
	COLOURS=$(convert /tmp/shot.png -format %k info:)
	echo "screenshot: mean brightness ${MEAN}/255 across ${COLOURS} colours"

	if [ "$MEAN" -gt 100 ]; then
		fail "the window is bright (mean ${MEAN}/255) -- WebKit is almost certainly showing its error page rather than the cockpit"
	fi
	if [ "$COLOURS" -lt 50 ]; then
		fail "the window is nearly featureless (${COLOURS} colours) -- nothing rendered"
	fi

	if grep -q "WebKit encountered an internal error" /tmp/run.log; then
		fail "WebKit reported an internal error"
	fi

	# The X11 identity is the runtime half of desktop integration. The GTK resource
	# name is lowercase and its resource class is conventionally title-cased;
	# StartupWMClass uses the latter so a real desktop resolves the bundled icon.
	# Bare Xvfb has no window manager, so _NET_WM_ICON is diagnostic only here.
	WINDOW_ID=$(xdotool search --onlyvisible --name "chartr" 2>/dev/null | head -n 1)
	[ -n "$WINDOW_ID" ] || fail "could not find the chartr window"
	PROPS=$(xprop -id "$WINDOW_ID" WM_CLASS _NET_WM_ICON)
	echo "$PROPS"
	echo "$PROPS" | grep -q "WM_CLASS(STRING) = \"chartr\", \"Chartr\"" || \
		fail "the window WM_CLASS does not match chartr.desktop"

	echo "SMOKE PASSED: the cockpit rendered and its window matches chartr.desktop"
'
