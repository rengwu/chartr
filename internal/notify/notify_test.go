package notify

import (
	"encoding/base64"
	"strings"
	"testing"
	"time"

	"github.com/rengwu/chartr/internal/model"
)

// Compose is the whole of the wording, so it is table-tested directly: the four
// reasons in the operator's words, the ticket where the tab is a session, and what
// a tab without one says instead.
func TestComposeSaysTheSpaceTheTicketTheReasonAndTheDuration(t *testing.T) {
	for _, tc := range []struct {
		name      string
		run       Run
		wantTitle string
		wantBody  string
	}{{
		name: "a session that landed idle",
		run: Run{Space: "chartr", MapSlug: "session-notifications-impl", TicketNum: 3,
			Reason: model.TerminalIdle, Duration: 4*time.Minute + 12*time.Second},
		wantTitle: "chartr",
		wantBody:  "session-notifications-impl #03 finished · ran 4m 12s",
	}, {
		name: "a session waiting on a permission prompt",
		run: Run{Space: "chartr", MapSlug: "widget", TicketNum: 11,
			Reason: model.TerminalBlocked, Duration: 90 * time.Second},
		wantTitle: "chartr",
		wantBody:  "widget #11 needs you · ran 1m 30s",
	}, {
		name: "a session that died mid-run",
		run: Run{Space: "chartr", MapSlug: "widget", TicketNum: 1,
			Reason: model.TerminalDead, Duration: 45 * time.Second},
		wantTitle: "chartr",
		wantBody:  "widget #01 crashed · ran 45s",
	}, {
		name: "a tab that is not a session names no ticket",
		run: Run{Space: "harness", Reason: model.TerminalExited,
			Duration: time.Hour + 3*time.Minute},
		wantTitle: "harness",
		wantBody:  "A terminal exited · ran 1h 3m",
	}, {
		name:      "a space chartr can no longer name still reports",
		run:       Run{Reason: model.TerminalIdle, Duration: 2 * time.Minute},
		wantTitle: "chartr",
		wantBody:  "A terminal finished · ran 2m",
	}} {
		t.Run(tc.name, func(t *testing.T) {
			got := Compose(tc.run)
			if got.Title != tc.wantTitle || got.Body != tc.wantBody {
				t.Errorf("Compose(%+v) = %q / %q, want %q / %q",
					tc.run, got.Title, got.Body, tc.wantTitle, tc.wantBody)
			}
		})
	}
}

// hostile is a space name shaped like the things that break a command line: both
// quotes, a backtick, a dollar, and a newline. A space is named by a directory the
// operator chose, so every one of these is legal input.
const hostile = "my \"repo\"; rm -rf $HOME `echo x`\nsecond line"

// The one part of this a stub notifier cannot cover: each platform builds an
// argument vector, so hostile text is data rather than syntax. The assertion is
// the same on all three whatever OS the suite runs on — the builders take no
// build tag, precisely so a Linux CI still proves the macOS and Windows vectors.
func TestPlatformCommandsCarryHostileTextAsArguments(t *testing.T) {
	n := Compose(Run{Space: hostile, MapSlug: "widget", TicketNum: 3,
		Reason: model.TerminalIdle, Duration: time.Minute})
	if !strings.Contains(n.Title, "\n") {
		t.Fatalf("the fixture stopped being hostile: %q", n.Title)
	}

	t.Run("darwin passes the text as osascript argv", func(t *testing.T) {
		c := macCommand(n)
		if c.Name != "osascript" {
			t.Errorf("name = %q, want osascript", c.Name)
		}
		// The title and body are their own argv entries, whole and unescaped, and the
		// script that reads them is a constant that names neither.
		if got := c.Args[len(c.Args)-2:]; got[0] != n.Body || got[1] != n.Title {
			t.Errorf("trailing argv = %q, want [body title] = %q", got, []string{n.Body, n.Title})
		}
		for _, a := range c.Args[:len(c.Args)-2] {
			if strings.Contains(a, "repo") {
				t.Errorf("the script argument %q interpolated the operator's text", a)
			}
		}
	})

	t.Run("linux passes the text as notify-send argv", func(t *testing.T) {
		c := linuxCommand(n)
		if c.Name != "notify-send" {
			t.Errorf("name = %q, want notify-send", c.Name)
		}
		if got := c.Args[len(c.Args)-2:]; got[0] != n.Title || got[1] != n.Body {
			t.Errorf("trailing argv = %q, want [title body] = %q", got, []string{n.Title, n.Body})
		}
		// `--` closes option parsing, so a title that begins with a dash is text.
		if c.Args[len(c.Args)-3] != "--" {
			t.Errorf("argv = %q, want -- before the positional title and body", c.Args)
		}
	})

	t.Run("windows carries the text as base64 inside a constant script", func(t *testing.T) {
		c := windowsCommand(n)
		if c.Name != "powershell" {
			t.Errorf("name = %q, want powershell", c.Name)
		}
		script := c.Args[len(c.Args)-1]
		if c.Args[len(c.Args)-2] != "-Command" {
			t.Errorf("argv = %q, want the script last, after -Command", c.Args)
		}
		// Nothing operator-shaped reaches PowerShell's parser: the script names the
		// text only as base64, and both blobs decode back to exactly what went in.
		if strings.Contains(script, "repo") || strings.Contains(script, "\n") {
			t.Errorf("the script interpolated the operator's text: %q", script)
		}
		for _, want := range []string{n.Title, n.Body} {
			blob := base64.StdEncoding.EncodeToString([]byte(want))
			if !strings.Contains(script, "'"+blob+"'") {
				t.Errorf("the script does not carry %q as base64", want)
			}
		}
	})
}

// An OS chartr has no path for degrades exactly as a missing binary does: an error
// the caller logs once and swallows, never a panic and never a silent success that
// would let a broken machine look like a working one.
func TestUnsupportedPlatformReportsRatherThanPretends(t *testing.T) {
	if _, ok := commandFor("plan9", Notification{Title: "t", Body: "b"}); ok {
		t.Fatal("commandFor claimed a notification path on plan9")
	}
	err := execNotifier{goos: "plan9"}.Notify(Notification{Title: "t", Body: "b"})
	if err == nil || !strings.Contains(err.Error(), "plan9") {
		t.Errorf("Notify on an unsupported platform = %v, want an error naming it", err)
	}
}
