package notify

import (
	"context"
	"encoding/base64"
	"fmt"
	"os/exec"
	"runtime"
	"strings"
	"time"
)

// notifyTimeout caps one delivery. Nothing here should take a tenth of it; the
// cap exists so a notification daemon that has wedged costs one abandoned process
// rather than a goroutine that never returns.
const notifyTimeout = 10 * time.Second

// command is the argument vector one notification is fired as: a binary and its
// arguments, never a shell string. It is a value rather than an *exec.Cmd so the
// three platform builders below are pure and are unit-testable on whichever OS the
// suite happens to be running — which matters, because the quoting they exist to
// avoid is the one part of this that a stub notifier cannot cover.
type command struct {
	Name string
	Args []string
}

// Platform returns the notifier for the machine chartr is running on, chosen once
// here so every consumer downstream holds the interface. An OS chartr has no
// notification path for yields a notifier that reports that on every call: the
// server logs the first one and carries on, which is the same degradation a
// missing binary gets.
func Platform() Notifier { return execNotifier{goos: runtime.GOOS} }

// execNotifier shells out to the platform's own notification tool. All three
// paths are exec, never a linked library, so the cgo-free single supported
// artifact (ADR 0011) is unaffected and nothing changes about how chartr builds or
// cross-compiles.
type execNotifier struct{ goos string }

func (e execNotifier) Notify(n Notification) error {
	c, ok := commandFor(e.goos, n)
	if !ok {
		return fmt.Errorf("notify: no notification path on %s", e.goos)
	}
	ctx, cancel := context.WithTimeout(context.Background(), notifyTimeout)
	defer cancel()
	if out, err := exec.CommandContext(ctx, c.Name, c.Args...).CombinedOutput(); err != nil {
		return fmt.Errorf("notify: %s: %w: %s", c.Name, err, strings.TrimSpace(string(out)))
	}
	return nil
}

// commandFor picks the platform builder. goos is a parameter rather than read
// from runtime here so the builders are reachable from a test on any machine.
func commandFor(goos string, n Notification) (command, bool) {
	switch goos {
	case "darwin":
		return macCommand(n), true
	case "linux":
		return linuxCommand(n), true
	case "windows":
		return windowsCommand(n), true
	default:
		return command{}, false
	}
}

// macCommand fires a notification through osascript.
//
// The obvious spelling — one -e holding `display notification "…"` with the text
// pasted in — interpolates operator text into AppleScript source, where a single
// quote ends the argument and a double quote ends the string literal. AppleScript
// has a first-class way not to: a run handler takes the trailing command-line
// arguments as `argv`, so the title and body travel as their own argv entries and
// the script that reads them is a constant.
func macCommand(n Notification) command {
	return command{Name: "osascript", Args: []string{
		"-e", "on run argv",
		"-e", "display notification (item 1 of argv) with title (item 2 of argv)",
		"-e", "end run",
		n.Body, n.Title,
	}}
}

// linuxCommand fires a notification through notify-send, the freedesktop
// notification client every desktop's daemon answers. The title and body are
// positional arguments, and `--` closes option parsing so a title that begins with
// a dash is text rather than a flag chartr does not know it is passing.
func linuxCommand(n Notification) command {
	return command{Name: "notify-send", Args: []string{
		"--app-name=chartr", "--", n.Title, n.Body,
	}}
}

// windowsCommand fires a toast through PowerShell.
//
// It is the one platform whose payload cannot ride argv: `powershell -Command`
// takes a *script*, and any argument after it is concatenated into that script
// rather than passed to it, so there is no argument vector to hide operator text
// in. Base64 is the way out. The script is a constant except for two base64 blobs,
// whose alphabet cannot contain a quote, a backtick, a newline or a `$`, so the
// text can neither end the string literal it sits in nor be expanded by the shell
// that parses it; PowerShell decodes them back to the exact bytes at the far end.
func windowsCommand(n Notification) command {
	script := strings.Join([]string{
		"$ErrorActionPreference='Stop'",
		"$title=" + decodeExpr(n.Title),
		"$body=" + decodeExpr(n.Body),
		"$null=[Windows.UI.Notifications.ToastNotificationManager,Windows.UI.Notifications,ContentType=WindowsRuntime]",
		"$xml=[Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02)",
		"$text=$xml.GetElementsByTagName('text')",
		"$null=$text.Item(0).AppendChild($xml.CreateTextNode($title))",
		"$null=$text.Item(1).AppendChild($xml.CreateTextNode($body))",
		"$toast=[Windows.UI.Notifications.ToastNotification]::new($xml)",
		// The toast needs an application id that is registered with the shell, and
		// chartr registers none — PowerShell's own is the standard borrow, and the
		// worst case is a toast attributed to it rather than no toast at all.
		"[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier(" +
			"'Microsoft.WindowsPowerShell_8wekyb3d8bbwe!powershell').Show($toast)",
	}, ";")

	return command{Name: "powershell", Args: []string{
		"-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", script,
	}}
}

// decodeExpr is the PowerShell expression that reproduces s exactly, whatever is
// in it. The literal in the script is always base64, so nothing operator-shaped
// ever reaches PowerShell's parser.
func decodeExpr(s string) string {
	return "[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('" +
		base64.StdEncoding.EncodeToString([]byte(s)) + "'))"
}
