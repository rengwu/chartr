# chartr — build and checks.
#
# The one supported artifact is the pure-Go binary with the Svelte build
# embedded (ADR 0010, 0011). `make build` produces it; `make check` and
# `make test` run everything a ticket must pass before commit.

BIN := bin/chartr

.PHONY: build web go-build dev-backend dev-web check test vet clean \
        webview snapshot release

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

clean:
	rm -rf $(BIN) build/
