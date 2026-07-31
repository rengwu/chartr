// Package notify fires the operating-system notification that tells an operator a
// long run ended (session-notifications spec, "The notification is fired
// server-side, per platform, best-effort"). It is the loud half of the one
// RunFinished event — the dot on the session's card is the quiet half — and it is
// what makes the case the whole effort exists for work: the operator has walked
// away and the cockpit's browser tab is closed, so the only surface left is the
// machine's own notification centre.
//
// Two things are load-bearing here.
//
// **Best-effort.** A missing binary, a non-zero exit, a headless box with no
// notification daemon: every one of them is a fact about the machine, never about
// the run. Notify reports it and the caller carries on. The feature degrades; the
// cockpit does not.
//
// **Arguments, never a shell string.** A space is named by a directory the
// operator chose and a ticket by text an agent wrote; both can contain quotes,
// backticks and newlines, and all three platform paths are command lines. Every
// path here builds an argument vector and lets the OS do the quoting — see
// command in exec.go, and windowsCommand for the one platform whose payload
// cannot ride argv.
package notify

import (
	"fmt"
	"time"

	"github.com/rengwu/chartr/internal/model"
)

// Notification is what the operator is shown: a title naming the space and a body
// naming the run. Nothing more — clicking it does nothing, because routing a click
// back into a specific cockpit view needs a deep-link scheme that does not exist
// and is out of scope.
type Notification struct {
	Title string
	Body  string
}

// Notifier delivers one notification. It is deliberately one method wide, with
// the platform chosen once where the server is built, so a test substitutes a stub
// and the suite never shells out to a real notifier on any OS.
//
// An error means the machine could not be told, and is the caller's to log and
// swallow — never to retry, and never to surface to the operator as a failure of
// the run it was reporting.
type Notifier interface {
	Notify(Notification) error
}

// Run is one ended run as the operator would describe it: which space, which
// ticket if the tab was a session, how it ended, and how long it worked. It is the
// content half of terminal.RunFinished, converted where the server knows how to
// name a space; nothing here re-derives any part of the clock's rule.
type Run struct {
	// Space is the space's name — what the sidebar calls it, not its id or path.
	Space string
	// MapSlug and TicketNum name the ticket, and are empty on a tab that is not a
	// session. An ad-hoc shell running a long build is a run like any other; it
	// simply has no ticket to name, and Compose says what it can rather than
	// inventing one.
	MapSlug   string
	TicketNum int
	// Reason is the state the tab settled into (model.TerminalIdle and friends),
	// which Compose translates into the operator's words.
	Reason string
	// Duration is how long the run worked, with the settle wait already excluded.
	Duration time.Duration
}

// Compose turns an ended run into the notification the operator reads. It is pure
// and is the one place the wording lives, so the content is testable without a
// platform, a server or a PTY.
func Compose(r Run) Notification {
	title := r.Space
	if title == "" {
		// A space chartr can no longer name — deregistered between the run ending and
		// this call — is still worth reporting; it just reports as chartr itself.
		title = "chartr"
	}

	subject := "A terminal"
	if r.MapSlug != "" {
		// The same identity the tab wears in the cockpit, so the notification and the
		// tab the operator comes back to read as the same thing.
		subject = fmt.Sprintf("%s #%02d", r.MapSlug, r.TicketNum)
	}

	return Notification{
		Title: title,
		Body:  fmt.Sprintf("%s %s · ran %s", subject, phrase(r.Reason), humanDuration(r.Duration)),
	}
}

// phrase says a published state in the operator's words. The four are the states
// a run can settle into; anything else is reported without a claim about what it
// means, since a state this does not know is the state grammar's news to break,
// not this package's to guess at.
func phrase(reason string) string {
	switch reason {
	case model.TerminalIdle:
		return "finished"
	case model.TerminalBlocked:
		return "needs you"
	case model.TerminalDead:
		return "crashed"
	case model.TerminalExited:
		return "exited"
	default:
		return "stopped"
	}
}

// humanDuration writes a run's length the way someone glancing at a notification
// reads it: whole seconds under a minute, minutes and seconds under an hour, hours
// and minutes above it. A zero part is dropped rather than written out, so a run
// reads "4m" and not "4m 0s".
func humanDuration(d time.Duration) string {
	d = d.Round(time.Second)
	if d < time.Minute {
		return fmt.Sprintf("%ds", int(d.Seconds()))
	}
	if d < time.Hour {
		m, s := int(d.Minutes()), int(d.Seconds())%60
		if s == 0 {
			return fmt.Sprintf("%dm", m)
		}
		return fmt.Sprintf("%dm %ds", m, s)
	}
	h, m := int(d.Hours()), int(d.Minutes())%60
	if m == 0 {
		return fmt.Sprintf("%dh", h)
	}
	return fmt.Sprintf("%dh %dm", h, m)
}
