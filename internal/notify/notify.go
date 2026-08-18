// Package notify fires the OS notification telling an operator a long run
// ended (session-notifications spec: fired server-side, per platform,
// best-effort). It's the loud half of RunFinished — the session card's dot
// is the quiet half — for when the operator has walked away and the
// cockpit tab is closed.
//
// Two things are load-bearing:
//
// **Best-effort.** A missing binary, non-zero exit, or headless box with no
// notification daemon is a fact about the machine, never the run. Notify
// reports it and the caller carries on — the feature degrades, the cockpit
// doesn't.
//
// **Arguments, never a shell string.** A space is named by an
// operator-chosen directory, a ticket by agent-written text; both can
// contain quotes, backticks, newlines, and all three platform paths are
// command lines. Every path here builds an argument vector and lets the OS
// do the quoting — see command in exec.go, and windowsCommand for the one
// platform whose payload can't ride argv.
package notify

import (
	"fmt"
	"time"

	"github.com/rengwu/chartr/internal/model"
)

// Notification is what the operator is shown: a title naming the space and
// a body naming the run. Nothing more — clicking it does nothing, since
// routing a click back into a cockpit view needs a deep-link scheme that
// doesn't exist and is out of scope.
type Notification struct {
	Title string
	Body  string
}

// Notifier delivers one notification. Deliberately one method wide, with the
// platform chosen once where the server is built, so a test substitutes a
// stub and the suite never shells out to a real notifier.
//
// An error means the machine couldn't be told — the caller's to log and
// swallow, never to retry or surface as a failure of the run it reported.
type Notifier interface {
	Notify(Notification) error
}

// Run is one ended run as the operator would describe it: which space,
// which ticket if the tab was a session, how it ended, how long it worked.
// The content half of terminal.RunFinished, converted where the server
// knows how to name a space.
type Run struct {
	// Space is the space's name — what the sidebar calls it, not its id or path.
	Space string
	// MapSlug and TicketNum name the ticket, empty on a tab that isn't a
	// session. An ad-hoc shell running a long build is a run like any
	// other; Compose says what it can rather than inventing a ticket.
	MapSlug   string
	TicketNum int
	// Reason is the provider outcome or attention state, translated into the
	// operator's words by Compose.
	Reason string
	// Duration is how long the run worked, with the settle wait already excluded.
	Duration time.Duration
}

// Compose turns an ended run into the notification the operator reads.
// Pure, and the one place the wording lives, so it's testable without a
// platform, a server or a PTY.
func Compose(r Run) Notification {
	title := r.Space
	if title == "" {
		// A space deregistered between the run ending and this call is
		// still worth reporting — it just reports as chartr itself.
		title = "chartr"
	}

	subject := "A terminal"
	if r.MapSlug != "" {
		// Same identity the tab wears in the cockpit, so notification and
		// tab read as the same thing.
		subject = fmt.Sprintf("%s #%02d", r.MapSlug, r.TicketNum)
	}

	return Notification{
		Title: title,
		Body:  fmt.Sprintf("%s %s · ran %s", subject, phrase(r.Reason), humanDuration(r.Duration)),
	}
}

// phrase says a provider outcome or published attention state in the operator's
// words. Anything unknown is reported without a claim about what it means.
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
	case "failed":
		return "failed"
	case "interrupted":
		return "was interrupted"
	default:
		return "stopped"
	}
}

// humanDuration writes a run's length the way someone glancing at a
// notification reads it: whole seconds under a minute, minutes and seconds
// under an hour, hours and minutes above. A zero part is dropped, so a run
// reads "4m" not "4m 0s".
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
