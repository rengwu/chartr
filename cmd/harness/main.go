// Command harness is the wayfinder-harness cockpit: one self-contained binary
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

	"github.com/rengwu/wayfinder-harness/internal/server"
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
	dataDir := flag.String("data-dir", "", "harness-owned state root (defaults to the current directory)")
	showVersion := flag.Bool("version", false, "print version and exit")
	flag.Parse()

	if *showVersion {
		fmt.Printf("wayfinder-harness %s (commit %s, built %s)\n", version, commit, date)
		return
	}

	if err := run(*addr, *dataDir); err != nil {
		log.Fatalf("harness: %v", err)
	}
}

func run(addr, dataDir string) error {
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

	fmt.Printf("wayfinder-harness %s cockpit on http://%s\n", version, ln.Addr())
	return srv.Serve(ctx, ln)
}
