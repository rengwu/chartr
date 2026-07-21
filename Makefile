# wayfinder-harness — build and checks.
#
# The one supported artifact is the pure-Go binary with the Svelte build
# embedded (ADR 0010, 0011). `make build` produces it; `make check` and
# `make test` run everything a ticket must pass before commit.

BIN := bin/harness

.PHONY: build web go-build dev-backend dev-web check test vet clean \
        webview snapshot release

## build: frontend then the self-contained binary with the SPA embedded.
build: web go-build

## web: install deps and produce web/dist (embedded by the web package).
web:
	cd web && npm install && npm run build

go-build:
	go build -o $(BIN) ./cmd/harness

## dev-backend: run the harness backend (serves :8787).
dev-backend:
	go run ./cmd/harness

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

## webview: build the best-effort native webview shell for GOOS into build/shell.
## This is a best-effort tier (ADR 0011): it needs cgo + a system webview library
## and MAY fail without blocking the supported release. The shell source is a
## separate future artifact; until cmd/webview exists this target is a no-op that
## exits 0, so the release job's shell lane stays green and simply attaches
## nothing for the platform. When the source lands, this is where it builds.
webview:
	@if [ -d ./cmd/webview ]; then \
		mkdir -p build/shell; \
		echo "building native webview shell for $${GOOS:-$$(go env GOOS)}"; \
		CGO_ENABLED=1 go build -tags webview -o build/shell/ ./cmd/webview; \
	else \
		echo "cmd/webview not present — best-effort webview shell tier not built (ADR 0011); nothing to attach"; \
	fi

clean:
	rm -rf $(BIN) build/
