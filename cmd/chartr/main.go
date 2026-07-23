// Command chartr is the chartr cockpit: one self-contained binary
// that serves the embedded Svelte SPA and drives wayfinder maps to completion.
//
// This entry point is deliberately thin — construct the server, listen, serve —
// so that the process-boundary tests exercise the exact same code path an
// operator runs.
package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"net"
	"os"
	"os/signal"
	"syscall"

	"github.com/rengwu/chartr/internal/env"
	"github.com/rengwu/chartr/internal/server"
)

// version, commit and date are stamped by goreleaser at release time via
// -ldflags -X (see .goreleaser.yaml). A plain `go build` leaves the defaults, so
// a from-source binary honestly reports itself as such rather than claiming a tag
// it was not cut from.
var (
	version = "dev"
	commit  = "none"
	date    = "unknown"
)

func main() {
	addr := flag.String("addr", "127.0.0.1:8787", "address to serve the cockpit on")
	dataDir := flag.String("data-dir", "", "chartr-owned state root (defaults to the current directory)")
	showVersion := flag.Bool("version", false, "print version and exit")
	flag.Parse()

	if *showVersion {
		fmt.Printf("chartr %s (commit %s, built %s)\n", version, commit, date)
		return
	}

	if err := run(*addr, *dataDir); err != nil {
		log.Fatalf("chartr: %v", err)
	}
}

func run(addr, dataDir string) error {
	// Adopt the operator's login-shell PATH before anything can resolve a binary,
	// so an agent they can run in their terminal is one chartr can find. It must
	// happen here rather than inside server.New: it mutates process-global state,
	// which is a main's business and not a constructor's.
	env.HydratePATH()

	srv, err := server.New(server.Options{DataDir: dataDir})
	if err != nil {
		return err
	}

	ln, err := net.Listen("tcp", addr)
	if err != nil {
		return err
	}

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	fmt.Printf("chartr %s cockpit on http://%s\n", version, ln.Addr())
	return srv.Serve(ctx, ln)
}
