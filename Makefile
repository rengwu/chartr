# wayfinder-harness — build and checks.
#
# The one supported artifact is the pure-Go binary with the Svelte build
# embedded (ADR 0010, 0011). `make build` produces it; `make check` and
# `make test` run everything a ticket must pass before commit.

BIN := bin/harness

.PHONY: build web go-build dev-backend dev-web check test vet clean

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

clean:
	rm -rf $(BIN)
