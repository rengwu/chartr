# chartr — build and checks.
#
# The one supported artifact is the pure-Go binary with the Svelte build
# embedded (ADR 0010, 0011). `make build` produces it; `make check` and
# `make test` run everything a ticket must pass before commit.

BIN := bin/chartr

.PHONY: build web go-build dev-backend dev-web check test vet clean \
        webview bundle dmg snapshot release

## build: frontend then the self-contained binary with the SPA embedded.
build: web go-build

## web: install deps and produce web/dist (embedded by the web package).
web:
	cd web && npm install && npm run build

go-build:
	go build -o $(BIN) ./cmd/chartr

## dev-backend: run the chartr backend (serves :8787).
dev-backend:
	go run ./cmd/chartr

## dev-web: run Vite with HMR, proxying /api and /ws to the backend.
dev-web:
	cd web && npm run dev

## check: static checks — go vet and svelte-check.
check: vet
	cd web && npm run check

vet:
	go vet ./...

## test: the process-boundary suite. Runs standalone — the embedded dist needs
## only the committed .gitkeep to compile, and the tests drive the control
## socket and HTTP, not the built SPA.
test:
	go test ./...

## snapshot: build the supported binaries locally without publishing, exactly as
## a release would (goreleaser, cgo-free), into build/goreleaser. Useful for
## eyeballing the artifact set before tagging.
snapshot:
	go run github.com/goreleaser/goreleaser/v2@latest release --snapshot --clean

## release: cut the real release from the current tag (goreleaser). Runs in CI on
## a v* tag; needs GITHUB_TOKEN. Local use is for dry-runs — prefer `snapshot`.
release:
	go run github.com/goreleaser/goreleaser/v2@latest release --clean

# The shell rides the same tag as the supported binary and must report the same
# stamp (ADR 0013), but it is built outside goreleaser, so the stamp is derived
# here. Overridable so CI can pass the exact tag it released.
WEBVIEW_VERSION ?= $(shell git describe --tags --always --dirty 2>/dev/null || echo dev)
WEBVIEW_COMMIT  ?= $(shell git rev-parse HEAD 2>/dev/null || echo none)
WEBVIEW_DATE    ?= $(shell git show -s --format=%cI HEAD 2>/dev/null || echo unknown)

## webview: build the best-effort native webview shell for the host into
## build/shell, with a per-asset .sha256 sidecar.
##
## This is a best-effort tier (ADR 0011): it needs cgo + a system webview library
## and MAY fail without blocking the supported release. It builds natively — cgo
## does not cross-compile — so the release workflow runs this once per runner.
## The sidecar is deliberately per-asset: the supported release owns
## checksums.txt, and a best-effort artifact must never mutate that manifest.
webview:
	@set -e; \
	goos=$$(go env GOOS); goarch=$$(go env GOARCH); \
	if [ -n "$$GOOS" ] && [ "$$GOOS" != "$$goos" ]; then \
		echo "webview shell cannot cross-compile to $$GOOS from $$goos (cgo); nothing to attach"; \
		exit 0; \
	fi; \
	mkdir -p build/shell; \
	ext=""; [ "$$goos" = "windows" ] && ext=".exe"; \
	name="chartr-shell_$(WEBVIEW_VERSION)_$${goos}_$${goarch}$$ext"; \
	echo "building native webview shell for $${goos}/$${goarch}"; \
	CGO_ENABLED=1 go build -tags webview -trimpath \
		-ldflags "-s -w -X main.version=$(WEBVIEW_VERSION) -X main.commit=$(WEBVIEW_COMMIT) -X main.date=$(WEBVIEW_DATE)" \
		-o "build/shell/$$name" ./cmd/webview; \
	cd build/shell; \
	if command -v sha256sum >/dev/null 2>&1; then \
		sha256sum "$$name" > "$$name.sha256"; \
	else \
		shasum -a 256 "$$name" > "$$name.sha256"; \
	fi; \
	echo "built build/shell/$$name"

# The bundle's reverse-DNS identifier, read off the repo rather than invented.
MACAPP_ID    := io.github.rengwu.chartr
# The oldest macOS the bundle claims to run on, used only if the executable turns
# out not to declare one. LSMinimumSystemVersion is otherwise read off the linked
# binary's own LC_BUILD_VERSION: the floor is whatever the runner's toolchain
# targeted, not a number we picked, and claiming an older one would only promise
# an operator a launch the loader then refuses.
MACAPP_MACOS := 12.0
# The marks the cockpit already ships (Vite copies web/public to the dist root, so
# these are the same bytes the runtime Dock icon reads).
#
# These are the mac-specific masters, NOT the square icon-512.png the PWA uses,
# and the difference is load-bearing: macOS does not mask app icons, so the .icns
# has to carry Apple's own shape — art inset to 824/1024 with a continuous-corner
# squircle and the template's shadow (ADR 0016). Feeding it a full-bleed square
# draws a tile that is visibly oversized and the wrong silhouette beside every
# neighbour in the Dock.
#
# Three of them, because the artwork is redrawn per size band rather than scaled:
# at 16 the cursor is bigger and bleeds off the plate, at 32 it is contained, and
# 48-and-up is the full-detail drawing. Each entry below is rendered from the
# master drawn for its band, and every canvas is the largest entry that band
# feeds, so the iconset loop only ever downscales. Regenerate all three with
# scripts/mac-app-icon.py.
MACAPP_MARK_16 := web/public/icon-mac-16.png
MACAPP_MARK_32 := web/public/icon-mac-32.png
MACAPP_MARK    := web/public/icon-mac-1024.png

## bundle: assemble the best-effort macOS app bundle — the shell executable, an
## Info.plist and a generated icon — into build/macapp, ad-hoc signed.
##
## This is `make webview` plus packaging, not a new kind of build: it packages the
## shell that target just built, with the same stamp, for the host's own
## architecture (cgo does not cross-compile) — which is in the staged directory's
## name, so an operator can see what they are getting and a second architecture
## can appear beside it later without renaming this one. Run `make web` first: the
## shell serves the cockpit out of the embedded dist, exactly as the loose one does.
##
## The signature is ad-hoc (`-`), which is the minimum that makes the app launch —
## Apple Silicon refuses to execute a binary carrying no signature at all. It is
## NOT a Developer ID signature and the app is not notarized, so Gatekeeper blocks
## the first launch of a downloaded copy; that is a stated cost (ADR 0016), not a
## bug. Signing is last, after the property list and the icon are in place: the Go
## linker signs the executable it produces, but nothing signs the bundle around it.
##
## Off macOS this prints a line and succeeds, like the webview target it builds
## on, which is what lets the shells job stay green on every runner.
bundle:
	@set -e; \
	goos=$$(go env GOOS); \
	if [ "$$goos" != "darwin" ]; then \
		echo "the macOS app bundle is only assembled on macOS (this is $$goos); nothing to package"; \
		exit 0; \
	fi; \
	$(MAKE) webview WEBVIEW_VERSION=$(WEBVIEW_VERSION) \
		WEBVIEW_COMMIT=$(WEBVIEW_COMMIT) WEBVIEW_DATE=$(WEBVIEW_DATE); \
	goarch=$$(go env GOARCH); \
	exe="build/shell/chartr-shell_$(WEBVIEW_VERSION)_darwin_$${goarch}"; \
	stage="build/macapp/chartr_$(WEBVIEW_VERSION)_darwin_$${goarch}"; \
	app="$$stage/chartr.app"; \
	echo "assembling $$app"; \
	rm -rf "$$stage"; \
	mkdir -p "$$app/Contents/MacOS" "$$app/Contents/Resources"; \
	cp "$$exe" "$$app/Contents/MacOS/chartr"; \
	iconset="$$stage/chartr.iconset"; \
	mkdir -p "$$iconset"; \
	for spec in "16 icon_16x16 $(MACAPP_MARK_16)" \
	            "32 icon_16x16@2x $(MACAPP_MARK_32)" \
	            "32 icon_32x32 $(MACAPP_MARK_32)" \
	            "64 icon_32x32@2x $(MACAPP_MARK)" \
	            "128 icon_128x128 $(MACAPP_MARK)" \
	            "256 icon_128x128@2x $(MACAPP_MARK)" \
	            "256 icon_256x256 $(MACAPP_MARK)" \
	            "512 icon_256x256@2x $(MACAPP_MARK)" \
	            "512 icon_512x512 $(MACAPP_MARK)"; do \
		set -- $$spec; \
		sips -z "$$1" "$$1" "$$3" --out "$$iconset/$$2.png" >/dev/null; \
	done; \
	iconutil -c icns "$$iconset" -o "$$app/Contents/Resources/chartr.icns"; \
	rm -rf "$$iconset"; \
	short=$$(printf '%s' '$(WEBVIEW_VERSION)' \
		| sed -n 's/^v\{0,1\}\([0-9][0-9]*\(\.[0-9][0-9]*\)\{0,2\}\)\([-+].*\)\{0,1\}$$/\1/p'); \
	[ -n "$$short" ] || short=0.0.0; \
	minos=$$(otool -l "$$app/Contents/MacOS/chartr" 2>/dev/null \
		| awk '/minos/ { print $$2; exit }'); \
	[ -n "$$minos" ] || minos=$(MACAPP_MACOS); \
	printf '%s\n' \
		'<?xml version="1.0" encoding="UTF-8"?>' \
		'<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">' \
		'<plist version="1.0">' \
		'<dict>' \
		'	<key>CFBundleInfoDictionaryVersion</key><string>6.0</string>' \
		'	<key>CFBundlePackageType</key><string>APPL</string>' \
		'	<key>CFBundleName</key><string>chartr</string>' \
		'	<key>CFBundleDisplayName</key><string>chartr</string>' \
		'	<key>CFBundleIdentifier</key><string>$(MACAPP_ID)</string>' \
		'	<key>CFBundleExecutable</key><string>chartr</string>' \
		'	<key>CFBundleIconFile</key><string>chartr</string>' \
		"	<key>CFBundleShortVersionString</key><string>$$short</string>" \
		'	<key>CFBundleVersion</key><string>$(WEBVIEW_VERSION)</string>' \
		"	<key>LSMinimumSystemVersion</key><string>$$minos</string>" \
		'	<key>LSApplicationCategoryType</key><string>public.app-category.developer-tools</string>' \
		'	<key>NSHighResolutionCapable</key><true/>' \
		'</dict>' \
		'</plist>' \
		> "$$app/Contents/Info.plist"; \
	codesign --force --sign - --identifier "$(MACAPP_ID)" "$$app"; \
	codesign --verify --strict "$$app"; \
	echo "built $$app (version $$short, build $(WEBVIEW_VERSION), darwin/$$goarch, macOS $$minos+, ad-hoc signed)"

## dmg: stage the assembled bundle into the one file an operator downloads — a
## read-only disk image in build/macapp, with a per-asset .sha256 sidecar.
##
## The staged layout is the customary one: the app, a symlink to /Applications as
## the drag target, and a plain-text file carrying the Gatekeeper instructions.
## There is deliberately NO styled window — background art with positioned icons
## means scripting the Finder and committing a window-state file, which is
## cosmetics on a tier that ships with a "your Mac will block this" note in the
## box (ADR 0016).
##
## The architecture in the image's name is load-bearing, not decoration: the
## bundle is one slice (cgo does not cross-compile), so the name is what tells an
## operator whether the image is theirs, and what lets a second architecture
## appear beside it later without renaming this one.
##
## The sidecar is per-asset for the same reason `make webview` writes one: the
## supported release owns checksums.txt, and a best-effort artifact never mutates
## that manifest.
##
## Off macOS this prints a line and succeeds, like the bundle target it builds on.
dmg:
	@set -e; \
	goos=$$(go env GOOS); \
	if [ "$$goos" != "darwin" ]; then \
		echo "the macOS disk image is only built on macOS (this is $$goos); nothing to package"; \
		exit 0; \
	fi; \
	$(MAKE) bundle WEBVIEW_VERSION=$(WEBVIEW_VERSION) \
		WEBVIEW_COMMIT=$(WEBVIEW_COMMIT) WEBVIEW_DATE=$(WEBVIEW_DATE); \
	goarch=$$(go env GOARCH); \
	name="chartr_$(WEBVIEW_VERSION)_darwin_$${goarch}"; \
	app="build/macapp/$$name/chartr.app"; \
	out="build/macapp/$$name.dmg"; \
	stage="build/macapp/.dmg-root"; \
	short=$$(plutil -extract CFBundleShortVersionString raw "$$app/Contents/Info.plist"); \
	echo "staging $$out"; \
	rm -rf "$$stage" "$$out"; \
	mkdir -p "$$stage"; \
	ditto "$$app" "$$stage/chartr.app"; \
	ln -s /Applications "$$stage/Applications"; \
	printf '%s\n' \
		'chartr for macOS' \
		'================' \
		'' \
		'chartr is signed ad-hoc and is NOT notarized: this project has no Apple' \
		'Developer account. So macOS blocks the first launch. That is expected, and' \
		'the steps below are the whole of it.' \
		'' \
		'Install' \
		'-------' \
		'1. Drag chartr onto the Applications folder in this window.' \
		'2. Eject this disk image.' \
		'' \
		'First launch (the one macOS blocks)' \
		'-----------------------------------' \
		'3. Open chartr from Launchpad or the Applications folder. macOS puts up a' \
		'   dialog: "chartr" Not Opened - Apple could not verify "chartr" is free of' \
		'   malware that may harm your Mac or compromise your privacy.' \
		'   Click Done. Do NOT click Move to Trash, which is the highlighted button.' \
		'4. Open System Settings > Privacy & Security and scroll down to Security.' \
		'   Under the line "chartr" was blocked to protect your Mac, click' \
		'   Open Anyway. Authenticate with Touch ID or your password, then click' \
		'   Open Anyway once more in the dialog that follows.' \
		'' \
		'chartr opens, and every later launch opens with no prompt at all.' \
		'' \
		'These steps were verified on macOS 27.0. Right-clicking the app and choosing' \
		'Open does NOT clear this any more, whatever older advice says - use step 4.' \
		'' \
		'From a terminal instead, if you prefer:' \
		'    xattr -d com.apple.quarantine /Applications/chartr.app' \
		'That removes the quarantine attribute macOS attaches to downloads. Run it' \
		'only if you trust this download.' \
		'' \
		'What is in this image' \
		'---------------------' \
		"chartr $$short (build $(WEBVIEW_VERSION)), for $$goarch Macs only." \
		'Verify the download against the sidecar published beside it:' \
		"    shasum -a 256 -c $$name.dmg.sha256" \
		> "$$stage/READ ME FIRST.txt"; \
	hdiutil create -quiet -volname "chartr $$short" -srcfolder "$$stage" \
		-fs HFS+ -format UDZO -ov "$$out"; \
	rm -rf "$$stage"; \
	( cd build/macapp && shasum -a 256 "$$name.dmg" > "$$name.dmg.sha256" ); \
	echo "built $$out (+ .sha256) — chartr $$short, darwin/$$goarch, unsigned and un-notarized"

clean:
	rm -rf $(BIN) build/
