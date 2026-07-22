package server

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"time"
)

// The folder picker is the native half of "add a space": the operator names a
// folder in their own OS chooser rather than typing an absolute path into a
// text field. It is raised server-side, exactly like the config surface's open
// action (ADR 0014) — chartr always serves on loopback, so a dialog the
// server raises lands on the operator's own desktop whether they are in the
// native shell or a plain browser at :8787. That is what lets one code path
// cover both binaries.
//
// The picker only *names* a folder. Registration stays the existing POST
// /api/spaces action, so the announced `git init` and every refusal keep the
// one response shape the register flow already has (story 2).

// pickTimeout bounds how long a raised chooser may stay open. It is generous —
// the operator may go hunting through a deep tree — but finite, so a dialog
// abandoned behind a window cannot hold the picker lock forever.
const pickTimeout = 10 * time.Minute

// errPickCancelled is the operator dismissing the chooser. It is an outcome,
// not a failure: the handler answers it as "no folder chosen" rather than an
// error the UI would show in red.
var errPickCancelled = errors.New("cancelled")

// errNoPicker is the absence of any native chooser on this machine — a Linux box
// with neither zenity nor kdialog, or Windows, which this build does not cover.
// The frontend never provokes it (the snapshot's nativePicker flag steers it to
// the typed-path form instead), but a hand-rolled request still gets a clear
// answer rather than a hang.
var errNoPicker = errors.New("no native folder chooser available")

// pickerCommand is the argv that raises this platform's folder chooser, and how
// to read a chosen path back out of it. Every supported chooser prints the
// absolute path on stdout and exits non-zero when dismissed, which is what makes
// one runner enough for all of them.
type pickerCommand struct {
	name string
	args []string
}

// nativePicker resolves this machine's folder chooser, or false when there is
// none. It is a lookup on $PATH and a GOOS switch — no dialog is raised — so it
// is safe to call at startup and cheap to call per request.
//
// Linux prefers zenity (GTK, the common default) and falls back to kdialog (KDE).
// macOS uses osascript, which is always present. Windows is deliberately
// unhandled: the shipping targets are macOS and Linux, and a Windows operator
// gets the typed-path form rather than a half-tested chooser.
func nativePicker(startDir string) (pickerCommand, bool) {
	switch runtime.GOOS {
	case "darwin":
		// `choose folder` returns an HFS-style alias; `POSIX path of` converts it to
		// the ordinary slash path the registry wants. `default location` opens the
		// chooser where the operator most likely wants to be rather than wherever
		// the app happened to leave it.
		script := fmt.Sprintf(
			`POSIX path of (choose folder with prompt "Choose a project folder to add as a space" default location (POSIX file %s))`,
			appleScriptString(startDir),
		)
		return pickerCommand{name: "osascript", args: []string{"-e", script}}, true
	case "linux":
		if path, err := exec.LookPath("zenity"); err == nil {
			return pickerCommand{name: path, args: []string{
				"--file-selection",
				"--directory",
				"--title=Add a space",
				"--filename=" + ensureTrailingSep(startDir),
			}}, true
		}
		if path, err := exec.LookPath("kdialog"); err == nil {
			return pickerCommand{name: path, args: []string{
				"--getexistingdirectory", startDir,
				"--title", "Add a space",
			}}, true
		}
	}
	return pickerCommand{}, false
}

// appleScriptString quotes a Go string as an AppleScript string literal. The
// start directory is server-derived (the operator's home), never client input,
// but it is interpolated into a script and so is quoted rather than trusted.
func appleScriptString(s string) string {
	return `"` + strings.NewReplacer(`\`, `\\`, `"`, `\"`).Replace(s) + `"`
}

// ensureTrailingSep makes a directory path read as a directory to zenity, which
// treats a --filename without a trailing separator as "select this entry in the
// parent" rather than "start inside it".
func ensureTrailingSep(dir string) string {
	if dir == "" || strings.HasSuffix(dir, string(os.PathSeparator)) {
		return dir
	}
	return dir + string(os.PathSeparator)
}

// pickStartDir is where a chooser opens. The operator's home is the honest
// default: it is where projects live, and it is the one directory guaranteed to
// exist. The chooser is free to navigate anywhere from there — a read-only
// browse is strictly less power than the register action it feeds, which already
// runs `git init` wherever it is pointed.
func pickStartDir() string {
	if home, err := os.UserHomeDir(); err == nil && home != "" {
		return home
	}
	return string(os.PathSeparator)
}

// pickFolder raises the native chooser and returns the absolute folder the
// operator named. It returns errPickCancelled when they dismiss it, and
// errNoPicker when this machine has no chooser at all.
func pickFolder(ctx context.Context) (string, error) {
	cmd, ok := nativePicker(pickStartDir())
	if !ok {
		return "", errNoPicker
	}

	ctx, cancel := context.WithTimeout(ctx, pickTimeout)
	defer cancel()

	out, err := exec.CommandContext(ctx, cmd.name, cmd.args...).Output()
	if err != nil {
		// Every supported chooser exits non-zero on dismissal and prints nothing
		// useful, so a non-zero exit with no path is a cancellation rather than a
		// fault. A context deadline reads the same way: nobody answered the dialog.
		var exitErr *exec.ExitError
		if errors.As(err, &exitErr) || ctx.Err() != nil {
			return "", errPickCancelled
		}
		return "", fmt.Errorf("raising the folder chooser: %w", err)
	}

	path := strings.TrimSpace(string(out))
	if path == "" {
		return "", errPickCancelled
	}
	// osascript's `POSIX path of` yields a trailing slash on directories; the
	// registry cleans paths itself, but a clean absolute path here keeps the
	// response and the notice reading the way the operator picked it.
	if abs, err := filepath.Abs(path); err == nil {
		path = abs
	}
	return path, nil
}

// handlePickFolder raises the operator's native folder chooser and answers with
// the folder they named. It registers nothing: the client posts the returned
// path to /api/spaces, so the register action stays the single place a space is
// created and the announced `git init` keeps one response shape.
//
// Only one chooser may be open at a time. A second request while one is up
// answers 409 rather than stacking dialogs the operator would have to dismiss in
// order.
func (s *Server) handlePickFolder(w http.ResponseWriter, r *http.Request) {
	if !s.pickLock.TryLock() {
		httpError(w, http.StatusConflict, "a folder chooser is already open")
		return
	}
	defer s.pickLock.Unlock()

	path, err := pickFolder(r.Context())
	switch {
	case errors.Is(err, errPickCancelled):
		// Dismissing the chooser is an ordinary outcome, so it is a 200 the client
		// can ignore rather than an error it would have to special-case out of its
		// error surface.
		writeJSON(w, http.StatusOK, map[string]any{"cancelled": true})
	case errors.Is(err, errNoPicker):
		httpError(w, http.StatusNotImplemented, errNoPicker.Error())
	case err != nil:
		httpError(w, http.StatusInternalServerError, err.Error())
	default:
		writeJSON(w, http.StatusOK, map[string]any{"path": path, "cancelled": false})
	}
}
