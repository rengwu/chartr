//go:build webview

package main

import (
	"context"
	"errors"
	"flag"
	"fmt"
	"net"
	"os"
	"os/signal"
	"reflect"
	"syscall"
	"unsafe"

	webview "github.com/webview/webview_go"

	"github.com/rengwu/chartr/internal/env"
	"github.com/rengwu/chartr/internal/server"
)

// version, commit and date are stamped at build time via -ldflags -X, exactly as
// they are for the supported binary — the shell rides the same tag, so it must
// report the same stamp (spec story 60).
var (
	version = "dev"
	commit  = "none"
	date    = "unknown"
)

// appName is the window title and the macOS menu-bar name.
const appName = "chartr"

func main() {
	dataDir := flag.String("data-dir", "", "chartr-owned state root (defaults to the current directory)")
	showVersion := flag.Bool("version", false, "print version and exit")
	flag.Parse()

	if *showVersion {
		fmt.Printf("chartr shell %s (commit %s, built %s)\n", version, commit, date)
		return
	}

	if err := run(*dataDir); err != nil {
		fmt.Fprintf(os.Stderr, "chartr shell: %v\n", err)
		os.Exit(1)
	}
}

// run is the shell's whole life: bind loopback, claim the single-instance lock,
// serve the cockpit in-process, and point a native window at it.
//
// It is `cmd/chartr`'s run() with two differences — the address is always
// `127.0.0.1:0` so there is no fixed port to collide on, and the window's
// lifetime is joined to the server's: closing the window cancels the same
// context signal.NotifyContext cancels today, and the server dies with it.
func run(dataDir string) error {
	// Adopt the operator's login-shell PATH before anything can resolve a binary.
	// The shell needs this more than the supported binary does: launched from
	// Finder or the Dock it inherits launchd's PATH, which carries neither
	// /opt/homebrew/bin nor ~/.local/bin, so without this no agent installed the
	// ordinary way is findable at all.
	env.HydratePATH()

	srv, err := server.New(server.Options{DataDir: dataDir})
	if err != nil {
		return err
	}

	// Bind first so the lock can record the real port. The OS picks it.
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		return err
	}
	url := "http://" + ln.Addr().String()

	release, err := acquireLock(dataDir, url)
	if err != nil {
		ln.Close()
		var locked *errLocked
		if errors.As(err, &locked) {
			// One window is the invariant. Raise the running one where the
			// platform lets us; where it does not, say so with the URL rather
			// than pretend a raise worked (spec story 54).
			if raiseInstance(locked.info.PID) {
				return nil
			}
			return fmt.Errorf("a shell is already running at %s (pid %d) — open that window, or quit it first",
				locked.info.URL, locked.info.PID)
		}
		return err
	}
	defer release()

	// Must precede window creation: macOS reads the app name once, when the
	// NSApplication behind the window is created.
	setAppName(appName)

	w, err := newWindow()
	if err != nil {
		ln.Close()
		return err
	}
	defer w.Destroy()

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()
	ctx, cancel := context.WithCancel(ctx)
	defer cancel()

	served := make(chan error, 1)
	go func() { served <- srv.Serve(ctx, ln) }()

	// A signal closes the window, so the two exit paths converge on one teardown.
	go func() {
		<-ctx.Done()
		w.Dispatch(w.Terminate)
	}()

	w.SetTitle(appName)
	w.SetSize(1280, 840, webview.HintNone)
	installNativeMenu(appName)
	w.Navigate(url)

	// Blocks on the native main loop until the window closes or Terminate fires.
	w.Run()
	cancel()
	return <-served
}

// newWindow creates the native window, turning "the native runtime is not here"
// into a hard error that names what is missing and points at the supported
// binary — never a silent browser fallback (spec story 58, ADR 0013).
func newWindow() (webview.WebView, error) {
	w := webview.New(false)
	if nativeHandle(w) == nil {
		return nil, fmt.Errorf("could not create a native window: %s.\n"+
			"The shell needs it at runtime; the supported, cgo-free cockpit binary is `chartr`,\n"+
			"which serves the same cockpit in your browser", missingRuntime)
	}
	return w, nil
}

// nativeHandle reads the C handle out of the webview_go wrapper.
//
// webview_create returns NULL when the window cannot be created — a missing
// WebView2 runtime on Windows, no display on Linux — but webview_go wraps that
// NULL in a non-nil WebView whose every method then dereferences it. There is
// no exported way to ask, and calling Window() to find out is the crash we are
// trying to avoid, so we read the wrapper's single handle field directly. This
// is the one unsafe corner in the shell and it is confined to this function.
func nativeHandle(w webview.WebView) unsafe.Pointer {
	v := reflect.ValueOf(w)
	if v.Kind() != reflect.Pointer || v.IsNil() {
		return nil
	}
	elem := v.Elem()
	if elem.Kind() != reflect.Struct || elem.NumField() == 0 {
		return nil
	}
	return unsafe.Pointer(elem.Field(0).Pointer())
}
