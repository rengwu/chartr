package server

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"time"

	"github.com/rengwu/chartr/internal/env"
)

// The folder picker is the native half of "add a space": the operator names a
// folder in their own OS chooser rather than typing an absolute path into a
// text field. It is raised server-side, exactly like the settings surface's open
// action — chartr always serves on loopback, so a dialog the
// server raises lands on the operator's own desktop whether they are in the
// native shell or a plain browser at :8787. That is what lets one code path
// cover both binaries.
//
// The picker only *names* a folder. Registration stays the existing POST
// /api/spaces action, so a success and every refusal keep the one response shape
// the register flow already has.

// pickTimeout bounds how long a raised chooser may stay open. It is generous —
// the operator may go hunting through a deep tree — but finite, so a dialog
// abandoned behind a window cannot hold the picker lock forever.
const pickTimeout = 10 * time.Minute

// defaultPickPrompt is what the chooser says when the caller names nothing
// more specific — "Add a space" is still the picker's original and most common
// caller. Any other caller (the sources form registering a dir source) sends
// its own prompt so the dialog reads as what it is actually for.
const defaultPickPrompt = "Choose a project folder to add as a space"

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
func nativePicker(startDir, prompt string) (pickerCommand, bool) {
	switch runtime.GOOS {
	case "darwin":
		// `choose folder` returns an HFS-style alias; `POSIX path of` converts it to
		// the ordinary slash path the registry wants. `default location` opens the
		// chooser where the operator most likely wants to be rather than wherever
		// the app happened to leave it.
		script := fmt.Sprintf(
			`POSIX path of (choose folder with prompt %s default location (POSIX file %s))`,
			appleScriptString(prompt),
			appleScriptString(startDir),
		)
		return pickerCommand{name: "osascript", args: []string{"-e", script}}, true
	case "linux":
		if path, err := exec.LookPath("zenity"); err == nil {
			return pickerCommand{name: path, args: []string{
				"--file-selection",
				"--directory",
				"--title=" + prompt,
				"--filename=" + ensureTrailingSep(startDir),
			}}, true
		}
		if path, err := exec.LookPath("kdialog"); err == nil {
			return pickerCommand{name: path, args: []string{
				"--getexistingdirectory", startDir,
				"--title", prompt,
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
// browse is strictly less power than the register action it feeds.
func pickStartDir() string {
	if home, err := os.UserHomeDir(); err == nil && home != "" {
		return home
	}
	return string(os.PathSeparator)
}

// dismissCode is the exit status every supported chooser uses for "the operator
// closed me without choosing" — zenity, kdialog and osascript's `choose folder`
// alike. It is the only non-zero exit that means the operator answered.
const dismissCode = 1

// stderrExcerpt bounds how much of a failing chooser's own words reach the
// operator. One line is what a loader error or a bad-argv complaint is; the cap
// is there so a chooser that decides to narrate cannot push a wall of text
// through an error surface sized for a sentence.
const stderrExcerpt = 300

// classifyPickError decides whether a chooser that exited non-zero was declined
// or broken, and is the whole reason this is a named function: the two are
// indistinguishable from the handler's side and were conflated for a release.
//
// Reporting a fault as a cancellation is not a cosmetic loss. When the AppImage
// pointed LD_LIBRARY_PATH at its own libraries, the operator's zenity died in
// the loader with exit 127 and a plain sentence on stderr saying exactly that —
// and the cockpit answered {"cancelled":true}, so the dialog that never opened
// looked like a dialog the operator had closed (#1, fixed in internal/env, and
// #2, this). Anything that is not the dismissal code or a timeout is therefore a
// fault now, and it carries the chooser's own stderr, which is where the
// actionable half of these failures has always been.
//
// timedOut is the caller's context deadline, passed rather than read so this
// stays a pure function a test can drive without raising a dialog.
func classifyPickError(err error, timedOut bool) error {
	// A deadline reads as a cancellation whatever the child's exit looked like:
	// nobody answered the dialog, and the kill that ends it is ours.
	if timedOut {
		return errPickCancelled
	}
	var exitErr *exec.ExitError
	if !errors.As(err, &exitErr) {
		// Not an exit at all — the chooser could not be started. nativePicker
		// resolved it on PATH moments ago, so this is a real fault.
		return fmt.Errorf("raising the folder chooser: %w", err)
	}
	if exitErr.ExitCode() == dismissCode {
		return errPickCancelled
	}
	// ExitCode is -1 when a signal ended it, which with no deadline of ours
	// means something else killed the chooser. That is worth saying plainly
	// rather than dressing up as an exit status.
	if exitErr.ExitCode() < 0 {
		return fmt.Errorf("the folder chooser was killed (%s)%s", exitErr.ProcessState, pickerStderr(exitErr))
	}
	return fmt.Errorf("the folder chooser exited %d%s", exitErr.ExitCode(), pickerStderr(exitErr))
}

// pickerStderr renders what the chooser printed as a suffix, or nothing when it
// printed nothing. cmd.Output captures stderr into the ExitError for exactly
// this; it was already being collected and only ever thrown away.
func pickerStderr(exitErr *exec.ExitError) string {
	msg := strings.TrimSpace(string(exitErr.Stderr))
	if msg == "" {
		return ""
	}
	if i := strings.IndexByte(msg, '\n'); i >= 0 {
		msg = msg[:i]
	}
	if len(msg) > stderrExcerpt {
		msg = msg[:stderrExcerpt] + "…"
	}
	return ": " + msg
}

// pickFolder raises the native chooser and returns the absolute folder the
// operator named. It returns errPickCancelled when they dismiss it, and
// errNoPicker when this machine has no chooser at all.
func pickFolder(ctx context.Context, prompt string) (string, error) {
	cmd, ok := nativePicker(pickStartDir(), prompt)
	if !ok {
		return "", errNoPicker
	}

	ctx, cancel := context.WithTimeout(ctx, pickTimeout)
	defer cancel()

	c := exec.CommandContext(ctx, cmd.name, cmd.args...)
	// The chooser is a host binary: under the AppImage it must get the
	// operator's environment back, not the bundle's loader paths, or it dies
	// in the loader before a window ever opens.
	c.Env = env.HostEnviron()
	out, err := c.Output()
	if err != nil {
		return "", classifyPickError(err, ctx.Err() != nil)
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
// created and keeps one response shape.
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

	// The caller may name what the dialog says it is choosing for — "Add a
	// space" and "register a skill source" want different words on the same
	// chooser. An absent or empty body keeps the original wording.
	var body struct {
		Prompt string `json:"prompt"`
	}
	if r.Body != nil {
		_ = json.NewDecoder(r.Body).Decode(&body)
	}
	prompt := body.Prompt
	if prompt == "" {
		prompt = defaultPickPrompt
	}

	path, err := pickFolder(r.Context(), prompt)
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
