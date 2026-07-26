package main

// This file is the shell's bundled-launch awareness, and like the lock beside it
// it is deliberately tag-free: pure Go, compiling and testing at CGO_ENABLED=0.
// That is the point — a launch with no terminal attached is the launch nobody
// watches, so the part that decides where the app writes and what it does with a
// stray argument is the part a unit test has to be able to reach.
//
// Everything here takes its inputs as arguments. A test drives it with
// constructed paths and never needs a real bundle, a real display, or a real
// home directory.

import (
	"flag"
	"io"
	"path/filepath"
)

// isBundled reports whether exePath — the shell's own executable path — sits
// inside a macOS application bundle: `…/Something.app/Contents/MacOS/chartr`.
//
// A path predicate, not a probe: no stat, no environment, no platform call. The
// shape is the whole test, because the shape is what the bundle format
// guarantees and what the assembly step produces.
func isBundled(exePath string) bool {
	macOS := filepath.Dir(exePath)
	if filepath.Base(macOS) != "MacOS" {
		return false
	}
	contents := filepath.Dir(macOS)
	if filepath.Base(contents) != "Contents" {
		return false
	}
	return filepath.Ext(filepath.Dir(contents)) == ".app"
}

// runtimeRoot picks the chartr-owned runtime root — sessions, payload archives
// and the single-instance lock — for this launch.
//
// A terminal launch is unchanged: an empty answer means "the working directory",
// which is where the shell has always written and what the operator `cd`'d to.
// A bundled launch cannot use that, because Finder hands it `/`, which it cannot
// write to: the single-instance lock is the first thing to touch the root, so
// the app would exit before drawing a window, writing the reason to a stream
// nobody reads.
//
// So a bundled launch anchors to configRoot — the same home-anchored path the
// config root already resolves to (server.ConfigRoot), not an Apple-conventional
// application-support directory. Two roots chosen by how the app was started
// would give one operator two session archives and two locks, and the split
// would be invisible until it confused them.
//
// An explicitly passed root always wins, bundled or not; and if there is no home
// to anchor to, configRoot is empty and a bundled launch degrades to the working
// directory exactly as the config root itself does.
func runtimeRoot(explicit, exePath, configRoot string) string {
	if explicit != "" {
		return explicit
	}
	if !isBundled(exePath) {
		return ""
	}
	return configRoot
}

// shellFlags is the shell's whole command line.
type shellFlags struct {
	dataDir     string
	showVersion bool
}

// parseFlags parses args — argv without the program name — into the shell's
// flags.
//
// From a terminal this is ordinary flag parsing: an unrecognised argument prints
// usage and is an error the caller exits on. Bundled it is not, because the
// arguments are no longer the operator's: the window server injects its own
// (`-psn_0_…` for a Finder launch, `-NSDocumentRevisionsDebugMode` under Xcode),
// and exiting over one would be a bounce in the Dock explained only on a stream
// Finder discards. So a bundled launch keeps whatever parsed before the
// unrecognised argument and ignores the rest.
func parseFlags(args []string, bundled bool) (shellFlags, error) {
	fs := flag.NewFlagSet("chartr", flag.ContinueOnError)
	if bundled {
		// There is no terminal to print usage to, and the argument being
		// complained about is not one an operator typed.
		fs.SetOutput(io.Discard)
	}
	dataDir := fs.String("data-dir", "", "chartr session/runtime root (defaults to the current directory, or to ~/.config/chartr when launched from an app bundle); user config lives under ~/.config/chartr")
	showVersion := fs.Bool("version", false, "print version and exit")

	err := fs.Parse(args)
	got := shellFlags{dataDir: *dataDir, showVersion: *showVersion}
	if err != nil && !bundled {
		return shellFlags{}, err
	}
	return got, nil
}
