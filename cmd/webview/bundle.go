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

// isAppImage reports whether this launch came out of a Linux AppImage.
//
// It is the same question isBundled asks, but it cannot be asked the same way.
// A macOS bundle is a path shape the assembly step produces, so the executable's
// own path is proof. An AppImage's executable path is `/tmp/.mount_XXXXXX/usr/
// bin/chartr` — a mount point with a random suffix that the runtime chooses
// afresh every launch and that self-extraction (`APPIMAGE_EXTRACT_AND_RUN`, what
// a machine without FUSE gets) does not even use. Matching that shape would be
// matching a temp directory.
//
// So the runtime's own statement is what is trusted instead: it exports APPIMAGE
// as the path of the .AppImage file it is running, verified on both the FUSE
// mount and the self-extracting path. It is taken as a lookup rather than read
// from the process so this stays as testable as the predicate above.
func isAppImage(lookup func(string) (string, bool)) bool {
	path, ok := lookup("APPIMAGE")
	return ok && path != ""
}

// runtimeRoot picks the chartr-owned runtime root — sessions, payload archives
// and the single-instance lock — for this launch.
//
// A terminal launch is unchanged: an empty answer means "the working directory",
// which is where the shell has always written and what the operator `cd`'d to.
// A packaged launch cannot use that, because the working directory is no longer
// something the operator chose. Finder hands a macOS bundle `/`, which it cannot
// write to: the single-instance lock is the first thing to touch the root, so
// the app would exit before drawing a window, writing the reason to a stream
// nobody reads. An AppImage is the same problem wearing the opposite failure —
// the working directory is wherever the operator happened to be, so it succeeds,
// and quietly writes a different runtime root per launch directory. Two windows
// then both open holding two locks, neither seeing the other, and the spaces
// registered from one are missing from the next (#3).
//
// So a bundled launch anchors to configRoot — the same home-anchored path the
// config root already resolves to (server.ConfigRoot), not an Apple-conventional
// application-support directory. Two roots chosen by how the app was started
// would give one operator two session archives and two locks, and the split
// would be invisible until it confused them.
//
// An explicitly passed root always wins, packaged or not; and if there is no
// home to anchor to, configRoot is empty and a packaged launch degrades to the
// working directory exactly as the config root itself does.
func runtimeRoot(explicit, exePath, configRoot string, lookup func(string) (string, bool)) string {
	if explicit != "" {
		return explicit
	}
	if !isBundled(exePath) && !isAppImage(lookup) {
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
	dataDir := fs.String("data-dir", "", "chartr session/runtime root (defaults to the current directory, or to ~/.config/chartr when launched from an app bundle or an AppImage); user config lives under ~/.config/chartr")
	showVersion := fs.Bool("version", false, "print version and exit")

	err := fs.Parse(args)
	got := shellFlags{dataDir: *dataDir, showVersion: *showVersion}
	if err != nil && !bundled {
		return shellFlags{}, err
	}
	return got, nil
}
